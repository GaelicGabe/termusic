use crate::config::{BindingForEvent, Keys, Settings, StyleColorSymbol};
/**
 * MIT License
 *
 * tuifeed - Copyright (c) 2021 Christian Visintin
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
use crate::ui::{Id, Model, Msg, PCMsg};
use tui_realm_stdlib::{Input, Paragraph, Radio, Table};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{
    Alignment, BorderType, Borders, Color, InputType, TableBuilder, TextModifiers, TextSpan,
};
use tuirealm::{Component, Event, MockComponent, State, StateValue};

#[derive(MockComponent)]
pub struct QuitPopup {
    component: Radio,
    keys: Keys,
}

impl QuitPopup {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Radio::default()
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::Yellow),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .title(" Are sure you want to quit?", Alignment::Center)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0),
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for QuitPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),

            Event::Keyboard(key) if key == self.keys.global_left.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.keys.global_right.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.keys.global_quit.key_event() => {
                return Some(Msg::QuitPopupCloseCancel)
            }
            Event::Keyboard(key) if key == self.keys.global_esc.key_event() => {
                return Some(Msg::QuitPopupCloseCancel)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => return None,
        };
        if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(0)))
        ) {
            Some(Msg::QuitPopupCloseCancel)
        } else if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(1)))
        ) {
            Some(Msg::QuitPopupCloseOk)
        } else {
            Some(Msg::None)
        }
    }
}

#[derive(MockComponent)]
pub struct ErrorPopup {
    component: Paragraph,
}

impl ErrorPopup {
    pub fn new<S: AsRef<str>>(msg: S) -> Self {
        Self {
            component: Paragraph::default()
                .borders(
                    Borders::default()
                        .color(Color::Red)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::Red)
                // .background(Color::Black)
                .modifiers(TextModifiers::BOLD)
                .alignment(Alignment::Center)
                .text(vec![TextSpan::from(msg.as_ref().to_string())].as_slice()),
        }
    }
}

impl Component<Msg, NoUserEvent> for ErrorPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter | Key::Esc,
                ..
            }) => Some(Msg::ErrorPopupClose),
            _ => None,
        }
    }
}

#[derive(MockComponent)]
pub struct HelpPopup {
    component: Table,
    keys: Keys,
}

impl HelpPopup {
    fn key(keys: &[BindingForEvent]) -> TextSpan {
        let mut text = String::new();
        for (idx, key) in keys.iter().enumerate() {
            if idx > 0 {
                text.push_str(", ");
            }
            text.push_str(&format!("<{key}>"));
        }
        TextSpan::from(text).bold().fg(Color::Cyan)
    }
    fn comment(text: &str) -> TextSpan {
        TextSpan::new(text)
    }
    #[allow(clippy::too_many_lines)]
    pub fn new(config: &Settings) -> Self {
        let keys = &config.keys;
        Self {
            component: Table::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Green),
                    ),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Black),
                )
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                .scroll(true)
                .title("Help: Esc or Enter to exit.", Alignment::Center)
                .rewind(false)
                .step(4)
                .row_height(1)
                .headers(&["Key", "Function"])
                .column_spacing(3)
                .widths(&[40, 60])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::new("Global").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(Self::key(&[keys.global_esc, keys.global_quit]))
                        .add_col(Self::comment("Exit"))
                        .add_row()
                        .add_col(TextSpan::new("<TAB>, <SHIFT+TAB>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Switch focus"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_left,
                            keys.global_right,
                            keys.global_up,
                            keys.global_down,
                            keys.global_goto_top,
                            keys.global_goto_bottom,
                        ]))
                        .add_col(Self::comment("Move cursor(vim style by default)"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_player_seek_forward,
                            keys.global_player_seek_backward,
                        ]))
                        .add_col(Self::comment("Seek forward/backward 5 seconds"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_lyric_adjust_forward,
                            keys.global_lyric_adjust_backward,
                        ]))
                        .add_col(Self::comment("Seek forward/backward 1 second for lyrics"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_player_speed_up,
                            keys.global_player_speed_down,
                        ]))
                        .add_col(Self::comment("Playback speed up/down 10 percent"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_player_toggle_gapless]))
                        .add_col(Self::comment("Toggle gapless playback"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_lyric_adjust_forward,
                            keys.global_lyric_adjust_backward,
                        ]))
                        .add_col(Self::comment("Before 10 seconds,adjust offset of lyrics"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_lyric_cycle]))
                        .add_col(Self::comment("Switch lyrics if more than 1 available"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_player_next,
                            keys.global_player_previous,
                            keys.global_player_toggle_pause,
                        ]))
                        .add_col(Self::comment("Next/Previous/Pause current track"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_player_volume_plus_1,
                            keys.global_player_volume_plus_2,
                            keys.global_player_volume_minus_1,
                            keys.global_player_volume_minus_2,
                        ]))
                        .add_col(Self::comment("Increase/Decrease volume"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_config_open]))
                        .add_col(Self::comment("Open Config Editor(all configuration)"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_save_playlist]))
                        .add_col(Self::comment("Save Playlist to m3u"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_layout_treeview]))
                        .add_col(Self::comment("Switch layout to treeview"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_layout_database]))
                        .add_col(Self::comment("Switch layout to database"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_layout_podcast]))
                        .add_col(Self::comment("Switch layout to podcast"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_xywh_move_left,
                            keys.global_xywh_move_right,
                        ]))
                        .add_col(Self::comment("Move album cover left/right"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_xywh_move_up,
                            keys.global_xywh_move_down,
                        ]))
                        .add_col(Self::comment("Move album cover up/down"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_xywh_zoom_in,
                            keys.global_xywh_zoom_out,
                        ]))
                        .add_col(Self::comment("Zoom in/out album cover"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_xywh_hide]))
                        .add_col(Self::comment("Hide/Show album cover"))
                        .add_row()
                        .add_col(TextSpan::new("Library").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(Self::key(&[keys.global_right, keys.library_load_dir]))
                        .add_col(Self::comment("Add one/all tracks to playlist"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_delete]))
                        .add_col(Self::comment("Delete track or folder"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_search_youtube]))
                        .add_col(Self::comment("Search or download track from youtube"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_tag_editor_open]))
                        .add_col(Self::comment("Open tag editor for tag and lyric download"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_yank, keys.library_paste]))
                        .add_col(Self::comment("Yank and Paste files"))
                        .add_row()
                        .add_col(TextSpan::new("<Enter>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Open sub directory as root"))
                        .add_row()
                        .add_col(TextSpan::new("<Backspace>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Go back to parent directory"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_search]))
                        .add_col(Self::comment("Search in library"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_switch_root]))
                        .add_col(Self::comment("Switch among several root folders"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_add_root]))
                        .add_col(Self::comment("Add new root folder"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_remove_root]))
                        .add_col(Self::comment("Remove current root from root folder list"))
                        .add_row()
                        .add_col(TextSpan::new("Playlist").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_delete, keys.playlist_delete_all]))
                        .add_col(Self::comment("Delete one/all tracks from playlist"))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_play_selected]))
                        .add_col(Self::comment("Play selected"))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_shuffle]))
                        .add_col(Self::comment("Randomize playlist"))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_mode_cycle]))
                        .add_col(Self::comment("Loop mode cycle"))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_add_front]))
                        .add_col(Self::comment(
                            "Add a track to the front of playlist or back",
                        ))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_search]))
                        .add_col(Self::comment("Search in playlist"))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_swap_down, keys.playlist_swap_up]))
                        .add_col(Self::comment("Swap track down/up in playlist"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.playlist_cmus_tqueue,
                            keys.playlist_cmus_lqueue,
                        ]))
                        .add_col(Self::comment("Select random tracks/albums to playlist"))
                        .add_row()
                        .add_col(TextSpan::new("Database").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(Self::key(&[keys.global_right, keys.database_add_all]))
                        .add_col(Self::comment("Add one/all track(s) to playlist"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_search]))
                        .add_col(Self::comment("Search in database"))
                        .add_row()
                        .add_col(TextSpan::new("Podcast").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(Self::key(&[keys.podcast_search_add_feed]))
                        .add_col(Self::comment("Feeds: search or add feed"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.podcast_delete_feed,
                            keys.podcast_delete_all_feeds,
                        ]))
                        .add_col(Self::comment("Feeds : delete one/all feeds"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.podcast_refresh_feed,
                            keys.podcast_refresh_all_feeds,
                        ]))
                        .add_col(Self::comment("Feeds : refresh one/all feeds"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.podcast_mark_played,
                            keys.podcast_mark_all_played,
                        ]))
                        .add_col(Self::comment("Episode: Mark one/all episodes played"))
                        .add_row()
                        .add_col(Self::key(&[keys.podcast_episode_download]))
                        .add_col(Self::comment("Episode: Download episode"))
                        .add_row()
                        .add_col(Self::key(&[keys.podcast_episode_delete_file]))
                        .add_col(Self::comment("Episode: delete episode local file"))
                        .build(),
                ),
            keys: keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for HelpPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::HelpPopupClose),

            Event::Keyboard(key) if key == self.keys.global_quit.key_event() => {
                return Some(Msg::HelpPopupClose)
            }
            Event::Keyboard(key) if key == self.keys.global_esc.key_event() => {
                return Some(Msg::HelpPopupClose)
            }

            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            _ => CmdResult::None,
        };

        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct DeleteConfirmRadioPopup {
    component: Radio,
    keys: Keys,
}

impl DeleteConfirmRadioPopup {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Radio::default()
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::LightRed),
                )
                // .background(Color::Black)
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightRed),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .title("Are sure you want to delete?", Alignment::Left)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0),
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for DeleteConfirmRadioPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),

            Event::Keyboard(key) if key == self.keys.global_left.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.keys.global_right.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.keys.global_quit.key_event() => {
                return Some(Msg::DeleteConfirmCloseCancel)
            }
            Event::Keyboard(key) if key == self.keys.global_esc.key_event() => {
                return Some(Msg::DeleteConfirmCloseCancel)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => return None,
        };
        if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(0)))
        ) {
            Some(Msg::DeleteConfirmCloseCancel)
        } else if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(1)))
        ) {
            Some(Msg::DeleteConfirmCloseOk)
        } else {
            Some(Msg::None)
        }
    }
}

#[derive(MockComponent)]
pub struct DeleteConfirmInputPopup {
    component: Input,
}

impl DeleteConfirmInputPopup {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: Input::default()
                .foreground(
                    style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .background(
                    style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .borders(
                    Borders::default()
                        .color(style_color_symbol.library_border().unwrap_or(Color::Green))
                        .modifiers(BorderType::Rounded),
                )
                // .invalid_style(Style::default().fg(Color::Red))
                .input_type(InputType::Text)
                .title("Type DELETE to confirm:", Alignment::Left),
        }
    }
}

impl Component<Msg, NoUserEvent> for DeleteConfirmInputPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => self.perform(Cmd::Delete),
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
            }) => self.perform(Cmd::Type(ch)),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::DeleteConfirmCloseCancel);
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::String(input_string))) => {
                if input_string == *"DELETE" {
                    return Some(Msg::DeleteConfirmCloseOk);
                }
                Some(Msg::DeleteConfirmCloseCancel)
            }
            _ => Some(Msg::None),
        }

        // if cmd_result == CmdResult::Submit(State::One(StateValue::String("DELETE".to_string()))) {
        //     Some(Msg::DeleteConfirmCloseOk)
        // } else {
        //     Some(Msg::DeleteConfirmCloseCancel)
        // }
    }
}

#[derive(MockComponent)]
pub struct MessagePopup {
    component: Paragraph,
}

impl MessagePopup {
    pub fn new<S: AsRef<str>>(title: S, msg: S) -> Self {
        Self {
            component: Paragraph::default()
                .borders(
                    Borders::default()
                        .color(Color::Cyan)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::Green)
                // .background(Color::Black)
                .modifiers(TextModifiers::BOLD)
                .alignment(Alignment::Center)
                .title(title, Alignment::Center)
                .text(vec![TextSpan::from(msg.as_ref().to_string())].as_slice()),
        }
    }
}

impl Component<Msg, NoUserEvent> for MessagePopup {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        // match ev {
        //     _ => None,
        // }
        None
    }
}

#[derive(MockComponent)]
pub struct SavePlaylistPopup {
    component: Input,
}

impl SavePlaylistPopup {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: Input::default()
                .foreground(
                    style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .background(
                    style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .borders(
                    Borders::default()
                        .color(style_color_symbol.library_border().unwrap_or(Color::Green))
                        .modifiers(BorderType::Rounded),
                )
                // .invalid_style(Style::default().fg(Color::Red))
                .input_type(InputType::Text)
                .title("Save Playlist as: (Enter to confirm)", Alignment::Left),
        }
    }
}

impl Component<Msg, NoUserEvent> for SavePlaylistPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => {
                self.perform(Cmd::Delete);
                self.perform(Cmd::Submit)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
            }) => {
                self.perform(Cmd::Type(ch));
                self.perform(Cmd::Submit)
            }
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::SavePlaylistPopupCloseCancel);
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => match self.component.state() {
                State::One(StateValue::String(input_string)) => {
                    return Some(Msg::SavePlaylistPopupCloseOk(input_string))
                }
                _ => return Some(Msg::None),
            },
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::String(input_string))) => {
                Some(Msg::SavePlaylistPopupUpdate(input_string))
            }
            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct SavePlaylistConfirm {
    component: Radio,
    keys: Keys,
    filename: String,
}

impl SavePlaylistConfirm {
    pub fn new(config: &Settings, filename: &str) -> Self {
        Self {
            component: Radio::default()
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::Yellow),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .title(
                    // format!(" Playlist {filename} exists. Overwrite? "),
                    " Playlist exists. Overwrite? ",
                    Alignment::Center,
                )
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0),
            keys: config.keys.clone(),
            filename: filename.to_string(),
        }
    }
}

impl Component<Msg, NoUserEvent> for SavePlaylistConfirm {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),

            Event::Keyboard(key) if key == self.keys.global_left.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.keys.global_right.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.keys.global_quit.key_event() => {
                return Some(Msg::SavePlaylistConfirmCloseCancel)
            }
            Event::Keyboard(key) if key == self.keys.global_esc.key_event() => {
                return Some(Msg::SavePlaylistConfirmCloseCancel)
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => return None,
        };
        if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(0)))
        ) {
            Some(Msg::SavePlaylistConfirmCloseCancel)
        } else if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(1)))
        ) {
            Some(Msg::SavePlaylistConfirmCloseOk(self.filename.clone()))
        } else {
            Some(Msg::None)
        }
    }
}

#[derive(MockComponent)]
pub struct PodcastAddPopup {
    component: Input,
}

impl PodcastAddPopup {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: Input::default()
                .foreground(
                    style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .background(
                    style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .borders(
                    Borders::default()
                        .color(style_color_symbol.library_border().unwrap_or(Color::Green))
                        .modifiers(BorderType::Rounded),
                )
                // .invalid_style(Style::default().fg(Color::Red))
                .input_type(InputType::Text)
                .title(
                    " Add or search podcast feed : (Enter to confirm) ",
                    Alignment::Left,
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for PodcastAddPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => self.perform(Cmd::Delete),
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
            }) => self.perform(Cmd::Type(ch)),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::Podcast(PCMsg::PodcastAddPopupCloseCancel));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => match self.component.state() {
                State::One(StateValue::String(input_string)) => {
                    return Some(Msg::Podcast(PCMsg::PodcastAddPopupCloseOk(input_string)));
                }
                _ => return Some(Msg::None),
            },
            _ => CmdResult::None,
        };
        // match cmd_result {
        //     CmdResult::Submit(State::One(StateValue::String(input_string))) => {
        //         Some(Msg::SavePlaylistPopupUpdate(input_string))
        //     }
        Some(Msg::None)
        // }
    }
}

#[derive(MockComponent)]
pub struct FeedDeleteConfirmRadioPopup {
    component: Radio,
    keys: Keys,
}

impl FeedDeleteConfirmRadioPopup {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Radio::default()
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::LightRed),
                )
                // .background(Color::Black)
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::LightRed),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .title("Are sure you to delete the feed?", Alignment::Left)
                .rewind(true)
                .choices(&["No", "Yes"])
                .value(0),
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for FeedDeleteConfirmRadioPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),

            Event::Keyboard(key) if key == self.keys.global_left.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.keys.global_right.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Left))
            }
            Event::Keyboard(key) if key == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Right))
            }
            Event::Keyboard(key) if key == self.keys.global_quit.key_event() => {
                return Some(Msg::Podcast(PCMsg::FeedDeleteCloseCancel))
            }
            Event::Keyboard(key) if key == self.keys.global_esc.key_event() => {
                return Some(Msg::Podcast(PCMsg::FeedDeleteCloseCancel))
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(0)))
        ) {
            Some(Msg::Podcast(PCMsg::FeedDeleteCloseCancel))
        } else if matches!(
            cmd_result,
            CmdResult::Submit(State::One(StateValue::Usize(1)))
        ) {
            Some(Msg::Podcast(PCMsg::FeedDeleteCloseOk))
        } else {
            Some(Msg::None)
        }
    }
}

#[derive(MockComponent)]
pub struct FeedDeleteConfirmInputPopup {
    component: Input,
}

impl FeedDeleteConfirmInputPopup {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: Input::default()
                .foreground(
                    style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .background(
                    style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .borders(
                    Borders::default()
                        .color(style_color_symbol.library_border().unwrap_or(Color::Green))
                        .modifiers(BorderType::Rounded),
                )
                // .invalid_style(Style::default().fg(Color::Red))
                .input_type(InputType::Text)
                .title(
                    "You're about the erase all feeds. Type DELETE to confirm:",
                    Alignment::Left,
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for FeedDeleteConfirmInputPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => self.perform(Cmd::Delete),
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
            }) => self.perform(Cmd::Type(ch)),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::Podcast(PCMsg::FeedsDeleteCloseCancel));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::String(input_string))) => {
                if input_string == *"DELETE" {
                    return Some(Msg::Podcast(PCMsg::FeedsDeleteCloseOk));
                }
                Some(Msg::Podcast(PCMsg::FeedsDeleteCloseCancel))
            }
            _ => Some(Msg::None),
        }

        // if cmd_result == CmdResult::Submit(State::One(StateValue::String("DELETE".to_string()))) {
        //     Some(Msg::DeleteConfirmCloseOk)
        // } else {
        //     Some(Msg::DeleteConfirmCloseCancel)
        // }
    }
}

#[derive(MockComponent)]
pub struct PodcastSearchTablePopup {
    component: Table,
    keys: Keys,
}

impl PodcastSearchTablePopup {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Table::default()
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Magenta),
                )
                .borders(
                    Borders::default()
                        .color(
                            config
                                .style_color_symbol
                                .library_border()
                                .unwrap_or(Color::Magenta),
                        )
                        .modifiers(BorderType::Rounded),
                )
                // .foreground(Color::Yellow)
                .title(" Enter to add feed: ", Alignment::Left)
                .scroll(true)
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                // .highlighted_str("🚀")
                .rewind(false)
                .step(4)
                .row_height(1)
                .headers(&[" Name ", " url "])
                .column_spacing(3)
                .widths(&[40, 60])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty result."))
                        .add_col(TextSpan::from("Loading..."))
                        .build(),
                ),
            keys: config.keys.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for PodcastSearchTablePopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::Podcast(PCMsg::SearchItunesCloseCancel))
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.global_quit.key_event() => {
                return Some(Msg::Podcast(PCMsg::SearchItunesCloseCancel))
            }
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),

            Event::Keyboard(keyevent) if keyevent == self.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }

            Event::Keyboard(keyevent) if keyevent == self.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(keyevent) if keyevent == self.keys.global_goto_top.key_event() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(keyevent) if keyevent == self.keys.global_goto_bottom.key_event() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            // Event::Keyboard(KeyEvent {
            //     code: Key::Tab,
            //     modifiers: KeyModifiers::NONE,
            // }) => return Some(Msg::YoutubeSearch(YSMsg::TablePopupNext)),
            // Event::Keyboard(KeyEvent {
            //     code: Key::BackTab,
            //     modifiers: KeyModifiers::SHIFT,
            // }) => return Some(Msg::YoutubeSearch(YSMsg::TablePopupPrevious)),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::SearchItunesCloseOk(index)));
                }
                CmdResult::None
            }
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

impl Model {
    pub fn mount_confirm_radio(&mut self) {
        assert!(self
            .app
            .remount(
                Id::DeleteConfirmRadioPopup,
                Box::new(DeleteConfirmRadioPopup::new(&self.config)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::DeleteConfirmRadioPopup).is_ok());
    }

    pub fn mount_confirm_input(&mut self) {
        assert!(self
            .app
            .remount(
                Id::DeleteConfirmInputPopup,
                Box::new(DeleteConfirmInputPopup::new(
                    &self.config.style_color_symbol
                )),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::DeleteConfirmInputPopup).is_ok());
    }

    pub fn mount_feed_delete_confirm_radio(&mut self) {
        assert!(self
            .app
            .remount(
                Id::FeedDeleteConfirmRadioPopup,
                Box::new(FeedDeleteConfirmRadioPopup::new(&self.config)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::FeedDeleteConfirmRadioPopup).is_ok());
    }

    pub fn umount_feed_delete_confirm_radio(&mut self) {
        if self.app.mounted(&Id::FeedDeleteConfirmRadioPopup) {
            assert!(self.app.umount(&Id::FeedDeleteConfirmRadioPopup).is_ok());
        }
    }
    pub fn mount_feed_delete_confirm_input(&mut self) {
        assert!(self
            .app
            .remount(
                Id::FeedDeleteConfirmInputPopup,
                Box::new(FeedDeleteConfirmInputPopup::new(
                    &self.config.style_color_symbol
                )),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::FeedDeleteConfirmInputPopup).is_ok());
    }
    pub fn umount_feed_delete_confirm_input(&mut self) {
        if self.app.mounted(&Id::FeedDeleteConfirmInputPopup) {
            assert!(self.app.umount(&Id::FeedDeleteConfirmInputPopup).is_ok());
        }
    }

    pub fn mount_podcast_search_table(&mut self) {
        assert!(self
            .app
            .remount(
                Id::PodcastSearchTablePopup,
                Box::new(PodcastSearchTablePopup::new(&self.config)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::PodcastSearchTablePopup).is_ok());
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {e}"));
        }
    }

    pub fn update_podcast_search_table(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();
        let mut idx = 0;
        if let Some(vec) = &self.podcast_search_vec {
            for record in vec {
                if idx > 0 {
                    table.add_row();
                }

                let title = record
                    .title
                    .clone()
                    .unwrap_or_else(|| "no title found".to_string());

                table
                    .add_col(TextSpan::new(title).bold())
                    .add_col(TextSpan::new(record.url.clone()));
                // .add_col(TextSpan::new(record.album().unwrap_or("Unknown Album")));
                idx += 1;
            }
            if self.player.playlist.is_empty() {
                table.add_col(TextSpan::from("0"));
                table.add_col(TextSpan::from("empty playlist"));
                table.add_col(TextSpan::from(""));
            }
        }
        let table = table.build();

        self.app
            .attr(
                &Id::PodcastSearchTablePopup,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();
    }
    pub fn umount_podcast_search_table(&mut self) {
        if self.app.mounted(&Id::PodcastSearchTablePopup) {
            assert!(self.app.umount(&Id::PodcastSearchTablePopup).is_ok());
        }
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(format!("update photo error: {e}"));
        }
    }
}
