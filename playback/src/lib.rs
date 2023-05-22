/*
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

#[cfg(all(feature = "gst", not(feature = "mpv")))]
mod gstreamer_backend;
#[cfg(feature = "mpv")]
mod mpv_backend;
pub mod playlist;
#[cfg(not(any(feature = "mpv", feature = "gst")))]
mod rusty_backend;
use anyhow::{anyhow, Result};
#[cfg(feature = "mpv")]
use mpv_backend::MpvBackend;
pub use playlist::{Playlist, Status};
use std::sync::mpsc::{self, Receiver, Sender};
use termusiclib::config::{SeekStep, Settings};
// #[cfg(not(any(feature = "mpv", feature = "gst")))]
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{
    io::{BufReader, Read, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
};

#[macro_use]
extern crate log;

lazy_static! {
    pub static ref TMP_DIR: String = format!(
        "/tmp/termusic-{}/",
        std::env::var("USER").expect("What is your name again?")
    );
    // static ref LOG: Log = Log::get("termusicd", "termusic");
    // static ref PLAYER: RwLock<GeneralPlayer> = RwLock::new(GeneralPlayer::new());
    // static ref CONFIG: MLConfig = MLConfig::load();
}
#[allow(clippy::module_name_repetitions, dead_code)]
pub enum PlayerMsg {
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    CacheStart(String),
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    CacheEnd(String),
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    Duration(u64),
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    DurationNext(u64),
    Eos,
    AboutToFinish,
    CurrentTrackUpdated,
    Progress(i64, i64),
}

#[allow(clippy::module_name_repetitions, dead_code)]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum PlayerCmd {
    GetProgress,
    MessageOnEnd,
    Play(String, bool),
    TogglePause,
    QueueNext(String, bool),
    Resume,
    Seek(i64),
    SeekRelative(i64),
    Skip,
    Speed(i32),
    Stop,
    Volume(i64),
    // Load(Playlist),
    // CycleRepeat,
    // Play,
    // Restart,
    // Next,
    // Prev,
    // Resume,
    // Pause,
    // Stop,
    // Seek(u64),
    // SetQueue(Playlist),
    // Shuffle,
    // SetPos(Song),
    ProcessID,
    // CurrentTime,
    // Status,
    // GetQueue
    Eos,
    VolumeUp,
    VolumeDown,
    CurrentTrackUpdated(String),
    CheckPlaylistChanged,
    ResetPlaylistChanged,
    ReloadPlaylist,
    StartPlay,
    FetchStatus,
    Progress(i64),
    SeekForward,
    SeekBackward,
}

impl PlayerCmd {
    /// Is this command mutable?
    pub fn is_mut(&self) -> bool {
        matches!(
            *self,
            // Self::Load(_)
            // | Self::CycleRepeat
            // | Self::Play
            // | Self::Restart
            // | Self::Next
            // | Self::Prev
            Self::CheckPlaylistChanged
                | Self::ResetPlaylistChanged
                | Self::ReloadPlaylist
                | Self::VolumeUp
                | Self::VolumeDown
                | Self::Eos
                | Self::Skip
                | Self::Resume
                | Self::TogglePause
                | Self::Stop
                | Self::Seek(_)
                | Self::StartPlay
                | Self::SeekForward
                | Self::SeekBackward
        )
    }
}

pub fn audio_cmd<T: for<'de> serde::Deserialize<'de>>(cmd: PlayerCmd, silent: bool) -> Result<T> {
    let socket_file = format!("{}/socket", *TMP_DIR);
    match UnixStream::connect(socket_file) {
        Ok(mut stream) => {
            let encoded = bincode::serialize(&cmd).expect("What went wrong?!");
            stream
                .write_all(&encoded)
                .expect("Unable to write to socket!");
            stream.shutdown(Shutdown::Write).expect("What went wrong?!");
            let buffer = BufReader::new(&stream);
            let encoded: Vec<u8> = buffer.bytes().map(|r| r.unwrap_or(0)).collect();
            Ok(bincode::deserialize(&encoded).expect("What went wrong?!"))
        }

        Err(why) => {
            if !silent {
                error!("unable to connect to socket: {why}");
            }
            Err(anyhow!(why.to_string()))
        }
    }
}
#[allow(clippy::module_name_repetitions)]
pub struct GeneralPlayer {
    #[cfg(all(feature = "gst", not(feature = "mpv")))]
    player: gstreamer_backend::GStreamer,
    #[cfg(feature = "mpv")]
    player: MpvBackend,
    #[cfg(not(any(feature = "mpv", feature = "gst")))]
    pub player: rusty_backend::Player,
    pub message_tx: Sender<PlayerMsg>,
    pub message_rx: Receiver<PlayerMsg>,
    pub playlist: Playlist,
    pub config: Settings,
}

impl GeneralPlayer {
    pub fn new(config: &Settings) -> Self {
        let (message_tx, message_rx): (Sender<PlayerMsg>, Receiver<PlayerMsg>) = mpsc::channel();
        #[cfg(all(feature = "gst", not(feature = "mpv")))]
        let player = gstreamer_backend::GStreamer::new(config, message_tx.clone());
        #[cfg(feature = "mpv")]
        let player = MpvBackend::new(config, message_tx.clone());
        #[cfg(not(any(feature = "mpv", feature = "gst")))]
        let player = rusty_backend::Player::new(config, message_tx.clone());
        let playlist = Playlist::new(config).unwrap_or_default();
        Self {
            player,
            message_tx,
            message_rx,
            playlist,
            config: config.clone(),
        }
    }
    pub fn toggle_gapless(&mut self) -> bool {
        self.player.gapless = !self.player.gapless;
        self.player.gapless
    }

    pub fn start_play(&mut self) {
        if self.playlist.is_stopped() | self.playlist.is_paused() {
            self.playlist.set_status(Status::Running);
        }

        if self.playlist.current_track().is_none() {
            // self.playlist.handle_current_track();
        }

        if let Some(file) = self.playlist.get_current_track() {
            if self.playlist.has_next_track() {
                self.playlist.set_next_track(None);
                // eprintln!("next track played");
                #[cfg(not(any(feature = "mpv", feature = "gst")))]
                {
                    // self.player.total_duration = Some(self.playlist.next_track_duration());
                    let mut t = self.player.total_duration.lock().unwrap();
                    *t = self.playlist.next_track_duration();
                    self.player.message_on_end();
                    self.message_tx
                        .send(PlayerMsg::CurrentTrackUpdated)
                        .expect("fail to send track updated signal");
                }
                return;
            }

            self.add_and_play(&file);
            // eprintln!("completely new track added");
            #[cfg(not(any(feature = "mpv", feature = "gst")))]
            {
                self.player.message_on_end();
                self.message_tx
                    .send(PlayerMsg::CurrentTrackUpdated)
                    .expect("fail to send track updated signal");
            }
        }
    }

    pub fn enqueue_next(&mut self) {
        if self.playlist.next_track().is_some() {
            return;
        }

        let track = match self.playlist.fetch_next_track() {
            Some(t) => t.clone(),
            None => return,
        };

        self.playlist.set_next_track(Some(&track));
        if let Some(file) = track.file() {
            #[cfg(not(any(feature = "mpv", feature = "gst")))]
            self.player.enqueue_next(file);
            // if let Some(d) = self.player.enqueue_next(file) {
            //     self.playlist.set_next_track_duration(d);
            //     // eprintln!("next track queued");
            // }
            #[cfg(all(feature = "gst", not(feature = "mpv")))]
            {
                self.player.enqueue_next(file);
                // eprintln!("next track queued");
                self.playlist.set_next_track(None);
                // self.playlist.handle_current_track();
            }

            #[cfg(feature = "mpv")]
            {
                self.player.enqueue_next(file);
                // eprintln!("next track queued");
            }
        }
    }

    pub fn skip(&mut self) {
        if self.playlist.current_track().is_some() {
            info!("skip route 1 which is in most cases.");
            self.playlist.set_next_track(None);
            self.player.skip_one();
            info!("current track is: {:?}", self.playlist.get_current_track());
        } else {
            info!("skip route 2 cause no current track.");
            self.message_tx.send(PlayerMsg::Eos).ok();
        }
    }
    pub fn toggle_pause(&mut self) {
        match self.playlist.status() {
            Status::Running => {
                self.player.pause();
                self.playlist.set_status(Status::Paused);
            }
            Status::Stopped => {}
            Status::Paused => {
                self.player.resume();
                self.playlist.set_status(Status::Running);
            }
        }
    }
    pub fn seek_relative(&mut self, forward: bool) {
        let mut offset = match self.config.seek_step {
            SeekStep::Short => -5_i64,
            SeekStep::Long => -30,
            SeekStep::Auto => {
                if let Some(track) = self.playlist.current_track() {
                    if track.duration().as_secs() >= 600 {
                        -30
                    } else {
                        -5
                    }
                } else {
                    -5
                }
            }
        };
        if forward {
            offset = -offset;
        }
        self.player.seek(offset).expect("Error in player seek.");
    }
}

impl PlayerTrait for GeneralPlayer {
    fn add_and_play(&mut self, current_track: &str) {
        self.player.add_and_play(current_track);
    }
    fn volume(&self) -> i32 {
        self.player.volume()
    }
    fn volume_up(&mut self) {
        self.player.volume_up();
    }
    fn volume_down(&mut self) {
        self.player.volume_down();
    }
    fn set_volume(&mut self, volume: i32) {
        self.player.set_volume(volume);
    }
    fn pause(&mut self) {
        self.playlist.set_status(Status::Paused);
        self.player.pause();
    }
    fn resume(&mut self) {
        self.playlist.set_status(Status::Running);
        self.player.resume();
    }
    fn is_paused(&self) -> bool {
        self.playlist.is_paused()
    }
    fn seek(&mut self, secs: i64) -> Result<()> {
        self.player.seek(secs)
    }
    fn seek_to(&mut self, last_pos: Duration) {
        self.player.seek_to(last_pos);
    }

    fn set_speed(&mut self, speed: i32) {
        self.player.set_speed(speed);
    }

    fn speed_up(&mut self) {
        self.player.speed_up();
    }

    fn speed_down(&mut self) {
        self.player.speed_down();
    }

    fn speed(&self) -> i32 {
        self.player.speed()
    }

    fn stop(&mut self) {
        self.playlist.set_status(Status::Stopped);
        self.playlist.set_next_track(None);
        self.playlist.clear_current_track();
        self.player.stop();
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait PlayerTrait {
    fn add_and_play(&mut self, current_track: &str);
    fn volume(&self) -> i32;
    fn volume_up(&mut self);
    fn volume_down(&mut self);
    fn set_volume(&mut self, volume: i32);
    fn pause(&mut self);
    fn resume(&mut self);
    fn is_paused(&self) -> bool;
    fn seek(&mut self, secs: i64) -> Result<()>;
    fn seek_to(&mut self, last_pos: Duration);
    // fn get_progress(&self) -> Result<()>;
    fn set_speed(&mut self, speed: i32);
    fn speed_up(&mut self);
    fn speed_down(&mut self);
    fn speed(&self) -> i32;
    fn stop(&mut self);
}
