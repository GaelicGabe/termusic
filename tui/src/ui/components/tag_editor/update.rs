/**
 * MIT License
 *
 * termusic - Copyright (C) 2021 Larry Hao
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
use crate::ui::{Id, IdTagEditor, Model, TEMsg, TFMsg};

impl Model {
    pub fn update_tageditor(&mut self, msg: &TEMsg) {
        match msg {
            TEMsg::TagEditorRun(node_id) => {
                self.mount_tageditor(node_id);
            }
            TEMsg::TagEditorClose(_song) => {
                if let Some(s) = self.tageditor_song.clone() {
                    self.library_reload_with_node_focus(s.file());
                }
                self.umount_tageditor();
            }

            TEMsg::TECounterDeleteOk => {
                self.te_delete_lyric();
            }
            TEMsg::TESelectLyricOk(index) => {
                if let Some(mut song) = self.tageditor_song.clone() {
                    song.set_lyric_selected_index(*index);
                    self.init_by_song(&song);
                }
            }
            TEMsg::TESearch => {
                self.te_songtag_search();
            }
            TEMsg::TEDownload(index) => {
                if let Err(e) = self.te_songtag_download(*index) {
                    self.mount_error_popup(format!("download song by tag error: {e}"));
                }
            }
            TEMsg::TEEmbed(index) => {
                if let Err(e) = self.te_load_lyric_and_photo(*index) {
                    self.mount_error_popup(format!("embed error: {e}"));
                }
            }
            TEMsg::TERename => {
                if let Err(e) = self.te_rename_song_by_tag() {
                    self.mount_error_popup(format!("rename song by tag error: {e}"));
                }
            }
            TEMsg::TEFocus(m) => self.update_tag_editor_focus(m),
        }
    }

    fn update_tag_editor_focus(&mut self, msg: &TFMsg) {
        match msg {
            TFMsg::TextareaLyricBlurDown | TFMsg::InputTitleBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::InputArtist))
                    .ok();
            }
            TFMsg::InputArtistBlurDown | TFMsg::InputAlbumBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::InputTitle))
                    .ok();
            }
            TFMsg::InputTitleBlurDown | TFMsg::InputGenreBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::InputAlbum))
                    .ok();
            }
            TFMsg::InputAlbumBlurDown | TFMsg::TableLyricOptionsBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::InputGenre))
                    .ok();
            }
            TFMsg::InputGenreBlurDown | TFMsg::SelectLyricBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::TableLyricOptions))
                    .ok();
            }
            TFMsg::TableLyricOptionsBlurDown | TFMsg::CounterDeleteBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::SelectLyric))
                    .ok();
            }
            TFMsg::SelectLyricBlurDown | TFMsg::TextareaLyricBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::CounterDelete))
                    .ok();
            }
            TFMsg::CounterDeleteBlurDown | TFMsg::InputArtistBlurUp => {
                self.app
                    .active(&Id::TagEditor(IdTagEditor::TextareaLyric))
                    .ok();
            }
        }
    }
}
