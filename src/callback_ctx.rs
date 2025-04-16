use jni::JNIEnv;

use crate::view::View;

pub struct CallbackCtx<'local> {
    pub env: JNIEnv<'local>,
    pub view: View<'local>,
}

impl<'local> CallbackCtx<'local> {
    pub(crate) fn new(env: JNIEnv<'local>, view: View<'local>) -> Self {
        Self { env, view }
    }
}
