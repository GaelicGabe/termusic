/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use super::Model;
use anyhow::{bail, Result};
use id3::TagLike;
use id3::Version::Id3v24;
use regex::Regex;
use shell_words;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;
use termusiclib::invidious::Instance;
use termusiclib::track::Track;
use termusiclib::types::{DLMsg, Id, Msg};
use termusiclib::types::{YSMsg, YoutubeOptions};
use termusiclib::utils::get_parent_folder;
use tokio::runtime::Handle;
use tuirealm::props::{Alignment, AttrValue, Attribute, TableBuilder, TextSpan};
use tuirealm::{State, StateValue};
use ytd_rs::{Arg, YoutubeDL};

#[expect(dead_code)]
static RE_FILENAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[ffmpeg\] Destination: (?P<name>.*)\.mp3").unwrap());

static RE_FILENAME_YTDLP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[ExtractAudio\] Destination: (?P<name>.*)\.mp3").unwrap());

impl Model {
    pub fn youtube_options_download(&mut self, index: usize) -> Result<()> {
        // download from search result here
        if let Ok(item) = self.youtube_options.get_by_index(index) {
            let url = format!("https://www.youtube.com/watch?v={}", item.video_id);
            if let Err(e) = self.youtube_dl(url.as_ref()) {
                bail!("Download error: {e}");
            }
        }
        Ok(())
    }

    /// This function requires to be run in a tokio Runtime context
    pub fn youtube_options_search(&mut self, keyword: String) {
        let tx = self.tx_to_main.clone();
        tokio::spawn(async move {
            match Instance::new(&keyword).await {
                Ok((instance, result)) => {
                    let youtube_options = YoutubeOptions {
                        items: result,
                        page: 1,
                        invidious_instance: instance,
                    };
                    tx.send(Msg::YoutubeSearch(YSMsg::YoutubeSearchSuccess(
                        youtube_options,
                    )))
                    .ok();
                }
                Err(e) => {
                    tx.send(Msg::YoutubeSearch(YSMsg::YoutubeSearchFail(e.to_string())))
                        .ok();
                }
            }
        });
    }

    /// This function requires to be run in a tokio Runtime context
    pub fn youtube_options_prev_page(&mut self) {
        // this needs to be wrapped as this is not running another thread but some main-runtime thread and so needs to inform the runtime to hand-off other tasks
        // though i am not fully sure if that is 100% the case, this avoid the panic though
        tokio::task::block_in_place(move || {
            Handle::current().block_on(async {
                match self.youtube_options.prev_page().await {
                    Ok(()) => self.sync_youtube_options(),
                    Err(e) => self.mount_error_popup(e.context("youtube-dl search")),
                }
            });
        });
    }

    /// This function requires to be run in a tokio Runtime context
    pub fn youtube_options_next_page(&mut self) {
        // this needs to be wrapped as this is not running another thread but some main-runtime thread and so needs to inform the runtime to hand-off other tasks
        // though i am not fully sure if that is 100% the case, this avoid the panic though
        tokio::task::block_in_place(move || {
            Handle::current().block_on(async {
                match self.youtube_options.next_page().await {
                    Ok(()) => self.sync_youtube_options(),
                    Err(e) => self.mount_error_popup(e.context("youtube-dl search")),
                }
            });
        });
    }

    pub fn sync_youtube_options(&mut self) {
        if self.youtube_options.items.is_empty() {
            let table = TableBuilder::default()
                .add_col(TextSpan::from("No results."))
                .add_col(TextSpan::from(
                    "Nothing was found in 10 seconds, connection issue encountered.",
                ))
                .build();
            self.app
                .attr(
                    &Id::YoutubeSearchTablePopup,
                    Attribute::Content,
                    AttrValue::Table(table),
                )
                .ok();
            return;
        }

        let mut table: TableBuilder = TableBuilder::default();
        for (idx, record) in self.youtube_options.items.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }
            let duration =
                Track::duration_formatted_short(&Duration::from_secs(record.length_seconds));
            let duration_string = format!("[{duration:^10.10}]");

            let title = record.title.as_str();

            table
                .add_col(TextSpan::new(duration_string))
                .add_col(TextSpan::new(title).bold());
        }
        let table = table.build();
        self.app
            .attr(
                &Id::YoutubeSearchTablePopup,
                Attribute::Content,
                AttrValue::Table(table),
            )
            .ok();

        if let Some(domain) = &self.youtube_options.invidious_instance.domain {
            let title = format!(
                    "\u{2500}\u{2500}\u{2500} Page {} \u{2500}\u{2500}\u{2500}\u{2524} {} \u{251c}\u{2500}\u{2500} {} \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                    self.youtube_options.page(),
                    "Tab/Shift+Tab switch pages",
                    domain,
                );
            self.app
                .attr(
                    &Id::YoutubeSearchTablePopup,
                    Attribute::Title,
                    AttrValue::Title((title, Alignment::Left)),
                )
                .ok();
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn youtube_dl(&mut self, url: &str) -> Result<()> {
        let mut path: PathBuf = std::env::temp_dir();
        if let Ok(State::One(StateValue::String(node_id))) = self.app.state(&Id::Library) {
            path = get_parent_folder(Path::new(&node_id)).to_path_buf();
        }
        let config_tui = self.config_tui.read();
        let mut args = vec![
            Arg::new("--extract-audio"),
            // Arg::new_with_arg("--audio-format", "vorbis"),
            Arg::new_with_arg("--audio-format", "mp3"),
            Arg::new("--add-metadata"),
            Arg::new("--embed-thumbnail"),
            Arg::new_with_arg("--metadata-from-title", "%(artist) - %(title)s"),
            #[cfg(target_os = "windows")]
            Arg::new("--restrict-filenames"),
            Arg::new("--write-sub"),
            Arg::new("--all-subs"),
            Arg::new_with_arg("--convert-subs", "lrc"),
            Arg::new_with_arg("--output", "%(title).90s.%(ext)s"),
        ];
        let extra_args = parse_args(&config_tui.settings.extra_ytdlp_args);
        let mut extra_args_parsed = parse_extra_args(extra_args);
        if extra_args_parsed.len() > 0 {
            args.append(&mut extra_args_parsed);
        }

        let ytd = YoutubeDL::new(&path, args, url)?;
        let tx = self.tx_to_main.clone();

        // avoid full string clones when sending via a channel
        let url: Arc<str> = Arc::from(url);

        thread::spawn(move || -> Result<()> {
            tx.send(Msg::Download(DLMsg::DownloadRunning(
                url.clone(),
                "youtube music".to_string(),
            )))
            .ok();
            // start download
            let download = ytd.download();

            // check what the result is and print out the path to the download or the error
            match download {
                Ok(result) => {
                    tx.send(Msg::Download(DLMsg::DownloadSuccess(url.clone())))
                        .ok();
                    // here we extract the full file name from download output
                    if let Some(file_fullname) =
                        extract_filepath(result.output(), &path.to_string_lossy())
                    {
                        tx.send(Msg::Download(DLMsg::DownloadCompleted(
                            url,
                            Some(file_fullname.clone()),
                        )))
                        .ok();

                        // here we remove downloaded live_chat.json file
                        remove_downloaded_json(&path, &file_fullname);

                        embed_downloaded_lrc(&path, &file_fullname);
                    } else {
                        tx.send(Msg::Download(DLMsg::DownloadCompleted(url, None)))
                            .ok();
                    }
                }
                Err(e) => {
                    tx.send(Msg::Download(DLMsg::DownloadErrDownload(
                        url.clone(),
                        "youtube music".to_string(),
                        e.to_string(),
                    )))
                    .ok();
                    tx.send(Msg::Download(DLMsg::DownloadCompleted(url, None)))
                        .ok();
                }
            }
            Ok(())
        });
        Ok(())
    }
}

// This just parsing the output from youtubedl to get the audio path
// This is used because we need to get the song name
// example ~/path/to/song/song.mp3
fn extract_filepath(output: &str, dir: &str) -> Option<String> {
    // #[cfg(not(feature = "yt-dlp"))]
    // if let Some(cap) = RE_FILENAME.captures(output) {
    //     if let Some(c) = cap.name("name") {
    //         let filename = format!("{}/{}.mp3", dir, c.as_str());
    //         return Ok(filename);
    //     }
    // }
    // #[cfg(feature = "yt-dlp")]
    if let Some(cap) = RE_FILENAME_YTDLP.captures(output) {
        if let Some(c) = cap.name("name") {
            let filename = format!("{dir}/{}.mp3", c.as_str());
            return Some(filename);
        }
    }
    None
}

fn remove_downloaded_json(path: &Path, file_fullname: &str) {
    let files = walkdir::WalkDir::new(path).follow_links(true);
    for f in files
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|f| {
            let p = Path::new(f.file_name());
            p.extension().is_some_and(|ext| ext == "json")
        })
        .filter(|f| {
            let path_json = Path::new(f.file_name());
            let p1: &Path = Path::new(file_fullname);
            path_json.file_stem().is_some_and(|stem_lrc| {
                p1.file_stem().is_some_and(|p_base| {
                    stem_lrc
                        .to_string_lossy()
                        .contains(p_base.to_string_lossy().as_ref())
                })
            })
        })
    {
        std::fs::remove_file(f.path()).ok();
    }
}

fn embed_downloaded_lrc(path: &Path, file_fullname: &str) {
    let mut id3_tag = if let Ok(tag) = id3::Tag::read_from_path(file_fullname) {
        tag
    } else {
        let mut tags = id3::Tag::new();
        let file_path = Path::new(file_fullname);
        if let Some(p_base) = file_path.file_stem() {
            tags.set_title(p_base.to_string_lossy());
        }
        tags.write_to_path(file_path, Id3v24).ok();
        tags
    };

    // here we add all downloaded lrc file
    let files = walkdir::WalkDir::new(path).follow_links(true);

    for entry in files
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|f| f.file_type().is_file())
        .filter(|f| {
            let name = f.file_name();
            let p = Path::new(&name);
            p.extension().is_some_and(|ext| ext == "lrc")
        })
        .filter(|f| {
            let path_lrc = Path::new(f.file_name());
            let p1: &Path = Path::new(file_fullname);
            path_lrc.file_stem().is_some_and(|stem_lrc| {
                p1.file_stem().is_some_and(|p_base| {
                    stem_lrc
                        .to_string_lossy()
                        .contains(p_base.to_string_lossy().as_ref())
                })
            })
        })
    {
        let path_lrc = Path::new(entry.file_name());
        let mut lang_ext = "eng".to_string();
        if let Some(p_short) = path_lrc.file_stem() {
            let p2 = Path::new(p_short);
            if let Some(ext2) = p2.extension() {
                lang_ext = ext2.to_string_lossy().to_string();
            }
        }
        let lyric_string = std::fs::read_to_string(entry.path());
        id3_tag.add_frame(id3::frame::Lyrics {
            lang: "eng".to_string(),
            description: lang_ext,
            text: lyric_string.unwrap_or_else(|_| String::from("[00:00:01] No lyric")),
        });
        std::fs::remove_file(entry.path()).ok();
    }

    id3_tag.write_to_path(file_fullname, Id3v24).ok();
}

fn parse_args(input: &str) -> Result<Vec<(String, Option<String>)>, shell_words::ParseError> {
    let result = shell_words::split(input)?
        .into_iter()
        .map(|token| {
            if token.starts_with("--") {
                let parts: Vec<&str> = token.splitn(2, '=').collect();
                (parts[0].to_string(), parts.get(1).map(|s| s.to_string()))
            } else if token.starts_with('-') {
                (token.to_string(), None)
            } else {
                ("_positional".to_string(), Some(token.to_string()))
            }
        })
        .collect();
    Ok(result)
}

fn parse_extra_args(
    extra_args: Result<Vec<(String, Option<String>)>, shell_words::ParseError>,
) -> Vec<Arg> {
    let mut extra_args_parsed = vec![];
    if let Ok(extra_args_vec) = extra_args {
        if extra_args_vec.len() > 0 {
            for (name, opt_arg) in extra_args_vec {
                let arg = match opt_arg {
                    Some(value) => Arg::new_with_arg(&name, &value),
                    None => Arg::new(&name),
                };
                extra_args_parsed.push(arg);
            }
        }
    }
    extra_args_parsed
}

#[cfg(test)]
mod tests {

    use crate::ui::model::youtube_options::extract_filepath;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_youtube_output_parsing() {
        // #[cfg(not(feature = "yt-dlp"))]
        // assert_eq!(
        //     extract_filepath(
        //         r"sdflsdf [ffmpeg] Destination: 观众说“小哥哥，到饭点了”《干饭人之歌》走，端起饭盆干饭去.mp3 sldflsdfj",
        //         "/tmp"
        //     )
        //     .unwrap(),
        //     "/tmp/观众说“小哥哥，到饭点了”《干饭人之歌》走，端起饭盆干饭去.mp3".to_string()
        // );
        assert_eq!(
            extract_filepath(
                r"sdflsdf [ExtractAudio] Destination: 观众说“小哥哥，到饭点了”《干饭人之歌》走，端起饭盆干饭去.mp3 sldflsdfj",
                "/tmp"
            )
            .unwrap(),
            "/tmp/观众说“小哥哥，到饭点了”《干饭人之歌》走，端起饭盆干饭去.mp3".to_string()
        );
    }
}
