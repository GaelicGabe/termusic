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
use crate::invidious::{Instance, YoutubeVideo};
use crate::track::Track;
use crate::ui::{DLMsg, Id, Msg};
use crate::utils::get_parent_folder;
use anyhow::{anyhow, bail, Result};
use id3::TagLike;
use id3::Version::Id3v24;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::{Path, PathBuf};
// use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, sleep};
use std::time::Duration;
use tuirealm::props::{Alignment, AttrValue, Attribute, TableBuilder, TextSpan};
use tuirealm::{State, StateValue};
use ytd_rs::{Arg, YoutubeDL};

lazy_static! {
    static ref RE_FILENAME: Regex =
        Regex::new(r"\[ffmpeg\] Destination: (?P<name>.*)\.mp3").unwrap();
    static ref RE_FILENAME_YTDLP: Regex =
        Regex::new(r"\[ExtractAudio\] Destination: (?P<name>.*)\.mp3").unwrap();
}

impl Model {
    pub fn youtube_options_download(&mut self, index: usize) -> Result<()> {
        // download from search result here
        if let Ok(item) = self.youtube_options.get_by_index(index) {
            let url = format!("https://www.youtube.com/watch?v={}", item.video_id);
            if let Err(e) = self.youtube_dl(url.as_ref()) {
                bail!("Error download: {e}");
            }
        }
        Ok(())
    }

    pub fn youtube_options_search(&mut self, keyword: &str) {
        let search_word = keyword.to_string();
        let tx = self.tx_to_main.clone();
        thread::spawn(
            move || match crate::invidious::Instance::new(&search_word) {
                Ok((instance, result)) => {
                    let youtube_options = YoutubeOptions {
                        items: result,
                        page: 1,
                        invidious_instance: instance,
                    };
                    tx.send(Msg::Download(DLMsg::YoutubeSearchSuccess(youtube_options)))
                        .ok();
                }
                Err(e) => {
                    tx.send(Msg::Download(DLMsg::YoutubeSearchFail(e.to_string())))
                        .ok();
                }
            },
        );
    }

    pub fn youtube_options_prev_page(&mut self) {
        match self.youtube_options.prev_page() {
            Ok(_) => self.sync_youtube_options(),
            Err(e) => self.mount_error_popup(format!("search error: {e}")),
        }
    }
    pub fn youtube_options_next_page(&mut self) {
        match self.youtube_options.next_page() {
            Ok(_) => self.sync_youtube_options(),
            Err(e) => self.mount_error_popup(format!("search error: {e}")),
        }
    }
    pub fn sync_youtube_options(&mut self) {
        if self.youtube_options.items.is_empty() {
            let table = TableBuilder::default()
                .add_col(TextSpan::from("Empty result."))
                .add_col(TextSpan::from(
                    "Wait 10 seconds but no results, means all servers are down.",
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
                Track::duration_formatted_short(&Duration::from_secs(record.length_seconds))
                    .to_string();
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
            path = PathBuf::from(get_parent_folder(&node_id));
        }
        let args = vec![
            Arg::new("--extract-audio"),
            // Arg::new_with_arg("--audio-format", "vorbis"),
            Arg::new_with_arg("--audio-format", "mp3"),
            Arg::new("--add-metadata"),
            Arg::new("--embed-thumbnail"),
            Arg::new_with_arg("--metadata-from-title", "%(artist) - %(title)s"),
            Arg::new("--write-sub"),
            Arg::new("--all-subs"),
            Arg::new_with_arg("--convert-subs", "lrc"),
            Arg::new_with_arg("--output", "%(title).90s.%(ext)s"),
        ];

        let ytd = YoutubeDL::new(&path, args, url)?;
        let tx = self.tx_to_main.clone();
        let url = url.to_string();

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
                    sleep(Duration::from_secs(5));
                    // here we extract the full file name from download output
                    if let Some(file_fullname) =
                        extract_filepath(result.output(), &path.to_string_lossy())
                    {
                        tx.send(Msg::Download(DLMsg::DownloadCompleted(
                            url.to_string(),
                            Some(file_fullname.clone()),
                        )))
                        .ok();

                        // here we remove downloaded live_chat.json file
                        remove_downloaded_json(&path, &file_fullname);

                        embed_downloaded_lrc(&path, &file_fullname);
                    } else {
                        tx.send(Msg::Download(DLMsg::DownloadCompleted(url.clone(), None)))
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
                    sleep(Duration::from_secs(5));
                    tx.send(Msg::Download(DLMsg::DownloadCompleted(url.clone(), None)))
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
            let name = f.file_name();
            let p = Path::new(&name);
            p.extension().map_or(false, |ext| ext == "json")
        })
        .filter(|f| {
            let path_json = Path::new(f.file_name());
            let p1: &Path = Path::new(file_fullname);
            path_json.file_stem().map_or(false, |stem_lrc| {
                p1.file_stem().map_or(false, |p_base| {
                    stem_lrc
                        .to_string_lossy()
                        .to_string()
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
        let mut t = id3::Tag::new();
        let p: &Path = Path::new(file_fullname);
        if let Some(p_base) = p.file_stem() {
            t.set_title(p_base.to_string_lossy());
        }
        t.write_to_path(p, Id3v24).ok();
        t
    };

    // here we add all downloaded lrc file
    let files = walkdir::WalkDir::new(path).follow_links(true);

    for f in files
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|f| f.file_type().is_file())
        .filter(|f| {
            let name = f.file_name();
            let p = Path::new(&name);
            p.extension().map_or(false, |ext| ext == "lrc")
        })
        .filter(|f| {
            let path_lrc = Path::new(f.file_name());
            let p1: &Path = Path::new(file_fullname);
            path_lrc.file_stem().map_or(false, |stem_lrc| {
                p1.file_stem().map_or(false, |p_base| {
                    stem_lrc
                        .to_string_lossy()
                        .to_string()
                        .contains(p_base.to_string_lossy().as_ref())
                })
            })
        })
    {
        let path_lrc = Path::new(f.file_name());
        let mut lang_ext = "eng".to_string();
        if let Some(p_short) = path_lrc.file_stem() {
            let p2 = Path::new(p_short);
            if let Some(ext2) = p2.extension() {
                lang_ext = ext2.to_string_lossy().to_string();
            }
        }
        let lyric_string = std::fs::read_to_string(f.path());
        id3_tag.add_frame(id3::frame::Lyrics {
            lang: "eng".to_string(),
            description: lang_ext,
            text: lyric_string.unwrap_or_else(|_| String::from("[00:00:01] No lyric")),
        });
        std::fs::remove_file(f.path()).ok();
    }

    id3_tag.write_to_path(file_fullname, Id3v24).ok();
}

#[cfg(test)]
#[allow(clippy::non_ascii_literal)]
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
