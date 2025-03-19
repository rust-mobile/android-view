use jni::JNIEnv;

use crate::view::View;

pub trait InputConnection {
    fn text_before_cursor<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        n: usize,
    ) -> Option<String>;

    // TODO
}
