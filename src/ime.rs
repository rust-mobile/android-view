use jni::{JNIEnv, objects::JObject};

use crate::view::View;

#[repr(transparent)]
pub struct EditorInfo<'local>(pub JObject<'local>);

// TODO: bind the EditorInfo methods most commonly called by editors

pub trait InputConnection {
    fn text_before_cursor<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        n: usize,
    ) -> Option<String>;

    // TODO
}
