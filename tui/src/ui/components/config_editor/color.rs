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
use crate::config::{ColorTermusic, Settings, StyleColorSymbol};
use std::convert::From;
use termusiclib::types::{ConfigEditorMsg, IdConfigEditor, Msg};
use tui_realm_stdlib::{Input, Label, Select, Table};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{
    Alignment, BorderType, Borders, Color, InputType, Style, TableBuilder, TextModifiers, TextSpan,
};
use tuirealm::tui::style::Modifier;
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

const COLOR_LIST: [ColorTermusic; 19] = [
    ColorTermusic::Reset,
    ColorTermusic::Foreground,
    ColorTermusic::Background,
    ColorTermusic::Black,
    ColorTermusic::Red,
    ColorTermusic::Green,
    ColorTermusic::Yellow,
    ColorTermusic::Blue,
    ColorTermusic::Magenta,
    ColorTermusic::Cyan,
    ColorTermusic::White,
    ColorTermusic::LightBlack,
    ColorTermusic::LightRed,
    ColorTermusic::LightGreen,
    ColorTermusic::LightYellow,
    ColorTermusic::LightBlue,
    ColorTermusic::LightMagenta,
    ColorTermusic::LightCyan,
    ColorTermusic::LightWhite,
];

#[derive(MockComponent)]
pub struct CEThemeSelectTable {
    component: Table,
    config: Settings,
}

impl CEThemeSelectTable {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: Table::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
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
                        .unwrap_or(Color::Reset),
                )
                .title(" Themes: <Enter> to preview ", Alignment::Left)
                .scroll(true)
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                .rewind(true)
                .step(4)
                .row_height(1)
                .headers(&["index", "Theme Name"])
                .column_spacing(1)
                .widths(&[18, 82])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty"))
                        .add_col(TextSpan::from("Empty Queue"))
                        .add_col(TextSpan::from("Empty"))
                        .build(),
                ),
            config: config.clone(),
        }
    }
}

impl Component<Msg, NoUserEvent> for CEThemeSelectTable {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _cmd_result = match ev {
            // Global Hotkeys
            Event::Keyboard(keyevent) if keyevent == self.config.keys.config_save.key_event() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk));
            }
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel));
            }
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_quit.key_event() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel));
            }

            // Local Hotkeys
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }

            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),

            Event::Keyboard(keyevent)
                if keyevent == self.config.keys.global_goto_top.key_event() =>
            {
                self.perform(Cmd::GoTo(Position::Begin))
            }

            Event::Keyboard(keyevent)
                if keyevent == self.config.keys.global_goto_bottom.key_event() =>
            {
                self.perform(Cmd::GoTo(Position::End))
            }

            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ThemeSelectBlurDown));
            }
            Event::Keyboard(KeyEvent {
                code: Key::BackTab,
                modifiers: KeyModifiers::SHIFT,
            }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ThemeSelectBlurUp));
            }

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::ConfigEditor(ConfigEditorMsg::ThemeSelectLoad(index)));
                }
                CmdResult::None
            }

            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct CEColorSelect {
    component: Select,
    id: IdConfigEditor,
    config: Settings,
    on_key_shift: Msg,
    on_key_backshift: Msg,
}

impl CEColorSelect {
    pub fn new(
        name: &str,
        id: IdConfigEditor,
        color: Color,
        config: &Settings,
        on_key_shift: Msg,
        on_key_backshift: Msg,
    ) -> Self {
        let init_value = Self::init_color_select(&id, &config.style_color_symbol);
        let mut choices = vec![];
        for color in &COLOR_LIST {
            choices.push(String::from(*color));
        }
        Self {
            component: Select::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(color),
                )
                // .foreground(color)
                .background(color)
                .title(name, Alignment::Left)
                .rewind(false)
                .inactive(Style::default().add_modifier(Modifier::BOLD).bg(color))
                .highlighted_color(Color::LightGreen)
                .highlighted_str(">> ")
                .choices(&choices)
                .value(init_value),
            id,
            config: config.clone(),
            on_key_shift,
            on_key_backshift,
        }
    }

    const fn init_color_select(
        id: &IdConfigEditor,
        style_color_symbol: &StyleColorSymbol,
    ) -> usize {
        match *id {
            IdConfigEditor::LibraryForeground => style_color_symbol.library_foreground.as_usize(),
            IdConfigEditor::LibraryBackground => style_color_symbol.library_background.as_usize(),
            IdConfigEditor::LibraryBorder => style_color_symbol.library_border.as_usize(),
            IdConfigEditor::LibraryHighlight => style_color_symbol.library_highlight.as_usize(),
            IdConfigEditor::PlaylistForeground => style_color_symbol.playlist_foreground.as_usize(),
            IdConfigEditor::PlaylistBackground => style_color_symbol.playlist_background.as_usize(),
            IdConfigEditor::PlaylistBorder => style_color_symbol.playlist_border.as_usize(),
            IdConfigEditor::PlaylistHighlight => style_color_symbol.playlist_highlight.as_usize(),
            IdConfigEditor::ProgressForeground => style_color_symbol.progress_foreground.as_usize(),
            IdConfigEditor::ProgressBackground => style_color_symbol.progress_background.as_usize(),
            IdConfigEditor::ProgressBorder => style_color_symbol.progress_border.as_usize(),
            IdConfigEditor::LyricForeground => style_color_symbol.lyric_foreground.as_usize(),
            IdConfigEditor::LyricBackground => style_color_symbol.lyric_background.as_usize(),
            IdConfigEditor::LyricBorder => style_color_symbol.lyric_border.as_usize(),
            _ => 0,
        }
    }

    fn update_color(&mut self, index: usize) -> Msg {
        if let Some(color_config) = COLOR_LIST.get(index) {
            let color = color_config
                .color(&self.config.style_color_symbol.alacritty_theme)
                .unwrap_or(Color::Red);
            // self.attr(Attribute::Foreground, AttrValue::Color(color));
            self.attr(Attribute::Background, AttrValue::Color(color));
            self.attr(
                Attribute::Borders,
                AttrValue::Borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(color),
                ),
            );
            self.attr(
                Attribute::FocusStyle,
                AttrValue::Style(Style::default().add_modifier(Modifier::BOLD).bg(color)),
            );
            Msg::ConfigEditor(ConfigEditorMsg::ColorChanged(
                self.id.clone(),
                *color_config,
            ))
        } else {
            self.attr(Attribute::Background, AttrValue::Color(Color::Red));
            self.attr(
                Attribute::Borders,
                AttrValue::Borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::Red),
                ),
            );
            self.attr(
                Attribute::FocusStyle,
                AttrValue::Style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Red)),
            );

            Msg::None
        }
    }
}

impl Component<Msg, NoUserEvent> for CEColorSelect {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            // Global Hotkeys
            Event::Keyboard(keyevent) if keyevent == self.config.keys.config_save.key_event() => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk));
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout));
            }

            Event::Keyboard(key) if key == self.config.keys.global_esc.key_event() => {
                match self.state() {
                    State::One(_) => return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel)),
                    _ => self.perform(Cmd::Cancel),
                }
            }
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_quit.key_event() => {
                match self.state() {
                    State::One(_) => return Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel)),
                    _ => self.perform(Cmd::Cancel),
                }
            }

            Event::Keyboard(key) if key == self.config.keys.global_up.key_event() => {
                match self.state() {
                    State::One(_) => return Some(self.on_key_backshift.clone()),
                    _ => self.perform(Cmd::Move(Direction::Up)),
                }
            }

            Event::Keyboard(key) if key == self.config.keys.global_down.key_event() => {
                match self.state() {
                    State::One(_) => return Some(self.on_key_shift.clone()),
                    _ => self.perform(Cmd::Move(Direction::Down)),
                }
            }

            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => match self.state() {
                State::One(_) => return Some(self.on_key_backshift.clone()),
                _ => self.perform(Cmd::Move(Direction::Up)),
            },

            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => match self.state() {
                State::One(_) => return Some(self.on_key_shift.clone()),
                _ => self.perform(Cmd::Move(Direction::Down)),
            },

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::Submit(State::One(StateValue::Usize(index))) => {
                Some(self.update_color(index))
            }
            _ => Some(Msg::None),
        }
    }
}
#[derive(MockComponent)]
pub struct ConfigLibraryTitle {
    component: Label,
}

impl Default for ConfigLibraryTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text(" Library style "),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryForeground {
    component: CEColorSelect,
}

impl ConfigLibraryForeground {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Foreground ",
                IdConfigEditor::LibraryForeground,
                config
                    .style_color_symbol
                    .library_foreground()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryForegroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryBackground {
    component: CEColorSelect,
}

impl ConfigLibraryBackground {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Background ",
                IdConfigEditor::LibraryBackground,
                config
                    .style_color_symbol
                    .library_background()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryBackgroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryBorder {
    component: CEColorSelect,
}

impl ConfigLibraryBorder {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Border ",
                IdConfigEditor::LibraryBorder,
                config
                    .style_color_symbol
                    .library_border()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryBorderBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryHighlight {
    component: CEColorSelect,
}

impl ConfigLibraryHighlight {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Highlight ",
                IdConfigEditor::LibraryHighlight,
                config
                    .style_color_symbol
                    .library_highlight()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LibraryHighlightBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LibraryHighlightBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryHighlight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistTitle {
    component: Label,
}

impl Default for ConfigPlaylistTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text(" Playlist style "),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistForeground {
    component: CEColorSelect,
}

impl ConfigPlaylistForeground {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Foreground ",
                IdConfigEditor::PlaylistForeground,
                config
                    .style_color_symbol
                    .playlist_foreground()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistForegroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistBackground {
    component: CEColorSelect,
}

impl ConfigPlaylistBackground {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Background ",
                IdConfigEditor::PlaylistBackground,
                config
                    .style_color_symbol
                    .playlist_background()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistBackgroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistBorder {
    component: CEColorSelect,
}

impl ConfigPlaylistBorder {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Border ",
                IdConfigEditor::PlaylistBorder,
                config
                    .style_color_symbol
                    .playlist_border()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistBorderBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistHighlight {
    component: CEColorSelect,
}

impl ConfigPlaylistHighlight {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Highlight ",
                IdConfigEditor::PlaylistHighlight,
                config
                    .style_color_symbol
                    .playlist_highlight()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistHighlightBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::PlaylistHighlightBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistHighlight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigProgressTitle {
    component: Label,
}

impl Default for ConfigProgressTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text(" Progress style "),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigProgressTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct ConfigProgressForeground {
    component: CEColorSelect,
}

impl ConfigProgressForeground {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Foreground ",
                IdConfigEditor::ProgressForeground,
                config
                    .style_color_symbol
                    .progress_foreground()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::ProgressForegroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::ProgressForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigProgressForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigProgressBackground {
    component: CEColorSelect,
}

impl ConfigProgressBackground {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Background ",
                IdConfigEditor::ProgressBackground,
                config
                    .style_color_symbol
                    .progress_background()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::ProgressBackgroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::ProgressBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigProgressBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigProgressBorder {
    component: CEColorSelect,
}

impl ConfigProgressBorder {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Border ",
                IdConfigEditor::ProgressBorder,
                config
                    .style_color_symbol
                    .progress_border()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::ProgressBorderBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::ProgressBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigProgressBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLyricTitle {
    component: Label,
}

impl Default for ConfigLyricTitle {
    fn default() -> Self {
        Self {
            component: Label::default()
                .modifiers(TextModifiers::BOLD)
                .text(" Lyric style "),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLyricTitle {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}

#[derive(MockComponent)]
pub struct ConfigLyricForeground {
    component: CEColorSelect,
}

impl ConfigLyricForeground {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Foreground ",
                IdConfigEditor::LyricForeground,
                config
                    .style_color_symbol
                    .lyric_foreground()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LyricForegroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LyricForegroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLyricForeground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLyricBackground {
    component: CEColorSelect,
}

impl ConfigLyricBackground {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Background ",
                IdConfigEditor::LyricBackground,
                config
                    .style_color_symbol
                    .lyric_background()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LyricBackgroundBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LyricBackgroundBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLyricBackground {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigLyricBorder {
    component: CEColorSelect,
}

impl ConfigLyricBorder {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: CEColorSelect::new(
                " Border ",
                IdConfigEditor::LyricBorder,
                config
                    .style_color_symbol
                    .lyric_border()
                    .unwrap_or(Color::Blue),
                config,
                Msg::ConfigEditor(ConfigEditorMsg::LyricBorderBlurDown),
                Msg::ConfigEditor(ConfigEditorMsg::LyricBorderBlurUp),
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLyricBorder {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigInputHighlight {
    component: Input,
    id: IdConfigEditor,
    config: Settings,
}

impl ConfigInputHighlight {
    pub fn new(name: &str, id: IdConfigEditor, config: &Settings) -> Self {
        let highlight_str = match id {
            IdConfigEditor::LibraryHighlightSymbol => {
                &config.style_color_symbol.library_highlight_symbol
            }
            _ => &config.style_color_symbol.playlist_highlight_symbol,
        };
        Self {
            component: Input::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Blue),
                    ),
                )
                // .foreground(color)
                .input_type(InputType::Text)
                .placeholder(
                    "1f984/1f680/1f8a5",
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                )
                .title(name, Alignment::Left)
                .value(highlight_str),
            id,
            config: config.clone(),
        }
    }
    fn update_symbol(&mut self, result: CmdResult) -> Msg {
        if let CmdResult::Changed(State::One(StateValue::String(symbol))) = result.clone() {
            if symbol.is_empty() {
                self.update_symbol_after(
                    self.config
                        .style_color_symbol
                        .library_border()
                        .unwrap_or(Color::Blue),
                );
                return Msg::None;
            }
            if let Some(s) = Self::string_to_unicode_char(&symbol) {
                // success getting a unicode letter
                self.update_symbol_after(Color::Green);
                return Msg::ConfigEditor(ConfigEditorMsg::SymbolChanged(
                    self.id.clone(),
                    s.to_string(),
                ));
            }
            // fail to get a unicode letter
            self.update_symbol_after(Color::Red);
        }

        // press enter to see preview
        if let CmdResult::Submit(State::One(StateValue::String(symbol))) = result {
            if let Some(s) = Self::string_to_unicode_char(&symbol) {
                self.attr(Attribute::Value, AttrValue::String(s.to_string()));
                self.update_symbol_after(Color::Green);
                return Msg::ConfigEditor(ConfigEditorMsg::SymbolChanged(
                    self.id.clone(),
                    s.to_string(),
                ));
            }
        }
        Msg::None
    }
    fn update_symbol_after(&mut self, color: Color) {
        self.attr(Attribute::Foreground, AttrValue::Color(color));
        self.attr(
            Attribute::Borders,
            AttrValue::Borders(
                Borders::default()
                    .modifiers(BorderType::Rounded)
                    .color(color),
            ),
        );
    }
    fn string_to_unicode_char(s: &str) -> Option<char> {
        // Do something more appropriate to find the actual number
        // let number = &s[..];

        u32::from_str_radix(s, 16)
            .ok()
            .and_then(std::char::from_u32)
    }
}

impl Component<Msg, NoUserEvent> for ConfigInputHighlight {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            // Global Hotkeys
            Event::Keyboard(keyevent) if keyevent == self.config.keys.config_save.key_event() => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::CloseOk))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::ChangeLayout))
            }
            Event::Keyboard(keyevent) if keyevent == self.config.keys.global_esc.key_event() => {
                Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel))
            }
            // Event::Keyboard(keyevent) if keyevent == self.config.keys.global_quit.key_event() => {
            //     Some(Msg::ConfigEditor(ConfigEditorMsg::CloseCancel))
            // }
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => {
                self.perform(Cmd::Move(Direction::Left));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => {
                self.perform(Cmd::Move(Direction::Right));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => {
                self.perform(Cmd::GoTo(Position::Begin));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End));
                Some(Msg::None)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => {
                let result = self.perform(Cmd::Cancel);
                Some(self.update_symbol(result))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => {
                let result = self.perform(Cmd::Delete);
                Some(self.update_symbol(result))
            }

            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            }) => {
                let result = self.perform(Cmd::Type(ch));
                Some(self.update_symbol(result))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => match self.id {
                IdConfigEditor::LibraryHighlightSymbol => Some(Msg::ConfigEditor(
                    ConfigEditorMsg::LibraryHighlightSymbolBlurDown,
                )),
                IdConfigEditor::PlaylistHighlightSymbol => Some(Msg::ConfigEditor(
                    ConfigEditorMsg::PlaylistHighlightSymbolBlurDown,
                )),
                _ => Some(Msg::None),
            },
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => match self.id {
                IdConfigEditor::LibraryHighlightSymbol => Some(Msg::ConfigEditor(
                    ConfigEditorMsg::LibraryHighlightSymbolBlurUp,
                )),
                IdConfigEditor::PlaylistHighlightSymbol => Some(Msg::ConfigEditor(
                    ConfigEditorMsg::PlaylistHighlightSymbolBlurUp,
                )),
                _ => Some(Msg::None),
            },

            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                let result = self.perform(Cmd::Submit);
                Some(self.update_symbol(result))
            }
            _ => Some(Msg::None),
        }
    }
}

#[derive(MockComponent)]
pub struct ConfigLibraryHighlightSymbol {
    component: ConfigInputHighlight,
}

impl ConfigLibraryHighlightSymbol {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: ConfigInputHighlight::new(
                " Highlight Symbol ",
                IdConfigEditor::LibraryHighlightSymbol,
                config,
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigLibraryHighlightSymbol {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}

#[derive(MockComponent)]
pub struct ConfigPlaylistHighlightSymbol {
    component: ConfigInputHighlight,
}

impl ConfigPlaylistHighlightSymbol {
    pub fn new(config: &Settings) -> Self {
        Self {
            component: ConfigInputHighlight::new(
                " Highlight Symbol ",
                IdConfigEditor::PlaylistHighlightSymbol,
                config,
            ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ConfigPlaylistHighlightSymbol {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(ev)
    }
}
