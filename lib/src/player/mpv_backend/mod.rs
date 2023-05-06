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
mod libmpv;

use super::{PlayerMsg, PlayerTrait};
use crate::config::Settings;
use anyhow::Result;
use libmpv::Mpv;
use libmpv::{
    events::{Event, PropertyData},
    Format,
};
use std::cmp;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

pub struct MpvBackend {
    // player: Mpv,
    volume: i32,
    speed: i32,
    pub gapless: bool,
    message_tx: Sender<PlayerMsg>,
    command_tx: Sender<PlayerCmd>,
}

enum PlayerCmd {
    // GetProgress,
    Play(String),
    Pause,
    QueueNext(String),
    Resume,
    Seek(i64),
    SeekAbsolute(i64),
    Speed(i32),
    Stop,
    Volume(i64),
}

impl MpvBackend {
    #[allow(clippy::too_many_lines)]
    pub fn new(config: &Settings, tx: Sender<PlayerMsg>) -> Self {
        let (command_tx, command_rx): (Sender<PlayerCmd>, Receiver<PlayerCmd>) = mpsc::channel();
        let volume = config.volume;
        let speed = config.speed;
        let gapless = config.gapless;
        let message_tx = tx.clone();

        let mpv = Mpv::new().expect("Couldn't initialize MpvHandlerBuilder");
        mpv.set_property("vo", "null")
            .expect("Couldn't set vo=null in libmpv");

        #[cfg(target_os = "linux")]
        mpv.set_property("ao", "pulse")
            .expect("Couldn't set ao=pulse in libmpv");

        mpv.set_property("volume", i64::from(volume))
            .expect("Error setting volume");
        mpv.set_property("speed", f64::from(speed) / 10.0).ok();
        let gapless_setting = if gapless { "yes" } else { "no" };
        mpv.set_property("gapless-audio", gapless_setting)
            .expect("gapless setting failed");

        let mut duration: i64 = 0;
        // let mut time_pos: i64 = 0;
        std::thread::spawn(move || {
            let mut ev_ctx = mpv.create_event_context();
            ev_ctx
                .disable_deprecated_events()
                .expect("failed to disable deprecated events.");
            ev_ctx
                .observe_property("duration", Format::Int64, 0)
                .expect("failed to watch volume");
            ev_ctx
                .observe_property("time-pos", Format::Int64, 0)
                .expect("failed to watch volume");
            loop {
                // if let Some(ev) = ev_ctx.wait_event(600.) {
                if let Some(ev) = ev_ctx.wait_event(0.0) {
                    match ev {
                        Ok(Event::EndFile(e)) => {
                            // eprintln!("event end file {:?} received", e);
                            if e == 0 {
                                message_tx.send(PlayerMsg::Eos).ok();
                            }
                        }
                        Ok(Event::StartFile) => {
                            message_tx.send(PlayerMsg::CurrentTrackUpdated).ok();
                        }
                        Ok(Event::PropertyChange {
                            name,
                            change,
                            reply_userdata: _,
                        }) => match name {
                            "duration" => {
                                if let PropertyData::Int64(c) = change {
                                    duration = c;
                                }
                            }
                            "time-pos" => {
                                if let PropertyData::Int64(time_pos) = change {
                                    // time_pos = c;
                                    message_tx
                                        .send(PlayerMsg::Progress(time_pos, duration))
                                        .ok();
                                }
                            }
                            &_ => {
                                // left for debug
                                // eprintln!(
                                //     "Event not handled {:?}",
                                //     Event::PropertyChange {
                                //         name,
                                //         change,
                                //         reply_userdata
                                //     }
                                // )
                            }
                        },
                        Ok(_e) => {}  //eprintln!("Event triggered: {:?}", e),
                        Err(_e) => {} //eprintln!("Event errored: {:?}", e),
                    }
                }

                if let Ok(cmd) = command_rx.try_recv() {
                    match cmd {
                        // PlayerCmd::Eos => message_tx.send(PlayerMsg::Eos).unwrap(),
                        PlayerCmd::Play(new) => {
                            duration = 0;
                            mpv.command("loadfile", &[&format!("\"{new}\""), "replace"])
                                .ok();
                            // .expect("Error loading file");
                            // eprintln!("add and play {} ok", new);
                        }
                        PlayerCmd::QueueNext(next) => {
                            mpv.command("loadfile", &[&format!("\"{next}\""), "append"])
                                .ok();
                            // .expect("Error loading file");
                        }
                        PlayerCmd::Volume(volume) => {
                            mpv.set_property("volume", volume).ok();
                            // .expect("Error increase volume");
                        }
                        PlayerCmd::Pause => {
                            mpv.set_property("pause", true).ok();
                        }
                        PlayerCmd::Resume => {
                            mpv.set_property("pause", false).ok();
                        }
                        PlayerCmd::Speed(speed) => {
                            mpv.set_property("speed", f64::from(speed) / 10.0).ok();
                        }
                        PlayerCmd::Stop => {
                            mpv.command("stop", &[""]).ok();
                        }
                        PlayerCmd::Seek(secs) => {
                            let time_pos_seek = mpv.get_property::<i64>("time-pos").unwrap_or(0);
                            duration = mpv.get_property::<i64>("duration").unwrap_or(100);
                            let mut absolute_secs = secs + time_pos_seek;
                            absolute_secs = cmp::max(absolute_secs, 0);
                            absolute_secs = cmp::min(absolute_secs, duration - 5);
                            mpv.pause().ok();
                            mpv.command("seek", &[&format!("\"{absolute_secs}\""), "absolute"])
                                .ok();
                            mpv.unpause().ok();
                            message_tx
                                .send(PlayerMsg::Progress(time_pos_seek, duration))
                                .ok();
                        }
                        PlayerCmd::SeekAbsolute(secs) => {
                            mpv.pause().ok();
                            while mpv
                                .command("seek", &[&format!("\"{secs}\""), "absolute"])
                                .is_err()
                            {
                                // This is because we need to wait until the file is fully loaded.
                                std::thread::sleep(Duration::from_millis(100));
                            }
                            mpv.unpause().ok();
                            message_tx.send(PlayerMsg::Progress(secs, duration)).ok();
                        }
                    }
                }

                // This is important to keep the mpv running, otherwise it cannot play.
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        });

        Self {
            volume,
            speed,
            gapless,
            message_tx: tx,
            command_tx,
        }
    }

    pub fn enqueue_next(&mut self, next: &str) {
        self.command_tx
            .send(PlayerCmd::QueueNext(next.to_string()))
            .ok();
    }

    fn queue_and_play(&mut self, new: &str) {
        self.command_tx
            .send(PlayerCmd::Play(new.to_string()))
            .expect("failed to queue and play");
    }

    pub fn skip_one(&mut self) {
        self.message_tx.send(PlayerMsg::Eos).unwrap();
    }
}

impl PlayerTrait for MpvBackend {
    fn add_and_play(&mut self, current_item: &str) {
        self.queue_and_play(current_item);
    }

    fn volume(&self) -> i32 {
        self.volume
    }

    fn volume_up(&mut self) {
        self.volume = cmp::min(self.volume + 5, 100);
        self.set_volume(self.volume);
    }

    fn volume_down(&mut self) {
        self.volume = cmp::max(self.volume - 5, 0);
        self.set_volume(self.volume);
    }
    fn set_volume(&mut self, volume: i32) {
        self.volume = volume.clamp(0, 100);
        self.command_tx
            .send(PlayerCmd::Volume(i64::from(self.volume)))
            .ok();
    }

    fn pause(&mut self) {
        self.command_tx.send(PlayerCmd::Pause).ok();
    }

    fn resume(&mut self) {
        self.command_tx.send(PlayerCmd::Resume).ok();
    }

    fn is_paused(&self) -> bool {
        true
    }

    fn seek(&mut self, secs: i64) -> Result<()> {
        self.command_tx.send(PlayerCmd::Seek(secs))?;
        Ok(())
    }

    #[allow(clippy::cast_possible_wrap)]
    fn seek_to(&mut self, last_pos: Duration) {
        self.command_tx
            .send(PlayerCmd::SeekAbsolute(last_pos.as_secs() as i64))
            .ok();
    }
    fn speed(&self) -> i32 {
        self.speed
    }

    fn set_speed(&mut self, speed: i32) {
        self.speed = speed;
        self.command_tx.send(PlayerCmd::Speed(self.speed)).ok();
    }

    fn speed_up(&mut self) {
        let mut speed = self.speed + 1;
        if speed > 30 {
            speed = 30;
        }
        self.set_speed(speed);
    }

    fn speed_down(&mut self) {
        let mut speed = self.speed - 1;
        if speed < 1 {
            speed = 1;
        }
        self.set_speed(speed);
    }
    fn stop(&mut self) {
        self.command_tx.send(PlayerCmd::Stop).ok();
    }
}
