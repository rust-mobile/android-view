use jni::{JNIEnv, objects::JObject, sys::jint};

use crate::{events::KeyEvent, view::View};

#[repr(transparent)]
pub struct EditorInfo<'local>(pub JObject<'local>);

// TODO: bind the EditorInfo methods most commonly called by editors

#[allow(unused_variables)]
pub trait InputConnection {
    fn text_before_cursor<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        n: jint,
    ) -> Option<String>;
    // TODO: styled version

    fn text_after_cursor<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        n: jint,
    ) -> Option<String>;
    // TODO: styled version

    fn selected_text<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
    ) -> Option<String>;
    // TODO: styled version

    fn cursor_caps_mode<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        req_modes: jint,
    ) -> jint;

    // TODO: Do we need to bind getExtractedText? Gio's InputConnection
    // just returns null.

    fn delete_surrounding_text<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        before_length: jint,
        after_length: jint,
    ) -> bool;

    fn delete_surrounding_text_in_code_points<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        before_length: jint,
        after_length: jint,
    ) -> bool;

    fn set_composing_text<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        text: &str,
        new_cursor_position: jint,
    ) -> bool;
    // TODO: styled version

    fn set_composing_region<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        start: jint,
        end: jint,
    ) -> bool;

    fn finish_composing_text<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
    ) -> bool;

    // TODO: Do we need to bind commitCompletion or commitCoorrection?
    // Gio's InputConnection just returns false for both.

    fn set_selection<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        start: jint,
        end: jint,
    ) -> bool;

    fn perform_editor_action<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        editor_action: jint,
    ) -> bool;

    fn perform_context_menu_action<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        id: jint,
    ) -> bool;

    fn begin_batch_edit<'local>(&mut self, env: &mut JNIEnv<'local>, view: &View<'local>) -> bool;

    fn end_batch_edit<'local>(&mut self, env: &mut JNIEnv<'local>, view: &View<'local>) -> bool;

    fn send_key_event<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        event: &KeyEvent<'local>,
    ) -> bool;

    fn clear_meta_key_states<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        states: jint,
    ) -> bool;

    fn report_fullscreen_mode<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        enabled: bool,
    ) -> bool;

    // TODO: Do we need to bind performPrivateCommand? Gio's InputConnection
    // just returns false.

    fn request_cursor_updates<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        cursor_update_mode: jint,
    ) -> bool;

    fn close_connection<'local>(&mut self, env: &mut JNIEnv<'local>, view: &View<'local>) {}

    // TODO: Do we need to bind commitContent? Gio's InputConnection
    // just returns false.
}
