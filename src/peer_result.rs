use jni::JNIEnv;

use crate::view::View;

pub struct PeerResult<'local, T> {
    result: T,
    deferred_fn: Option<fn(&mut JNIEnv<'local>, &View<'local>)>,
}

impl<'local, T> PeerResult<'local, T> {
    pub fn with_deferred_fn(
        result: T,
        deferred_fn: fn(&mut JNIEnv<'local>, &View<'local>),
    ) -> Self {
        Self {
            result,
            deferred_fn: Some(deferred_fn),
        }
    }

    pub(crate) fn finish(self, env: &mut JNIEnv<'local>, view: &View<'local>) -> T {
        if let Some(deferred_fn) = self.deferred_fn {
            deferred_fn(env, view);
        }
        self.result
    }

    pub(crate) fn map<U>(self, f: impl FnOnce(T) -> U) -> PeerResult<'local, U> {
        PeerResult {
            result: f(self.result),
            deferred_fn: self.deferred_fn,
        }
    }
}

impl<T> From<T> for PeerResult<'_, T> {
    fn from(result: T) -> Self {
        Self {
            result,
            deferred_fn: None,
        }
    }
}
