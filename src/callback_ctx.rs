use jni::JNIEnv;
use smallvec::SmallVec;

use crate::view::View;

enum DeferredCallback<'local> {
    Static(fn(&mut JNIEnv<'local>, &View<'local>)),
    Dynamic(Box<dyn FnOnce(&mut JNIEnv<'local>, &View<'local>)>),
}

pub struct CallbackCtx<'local> {
    pub env: JNIEnv<'local>,
    pub view: View<'local>,
    deferred_callbacks: SmallVec<[DeferredCallback<'local>; 4]>,
}

impl<'local> CallbackCtx<'local> {
    pub(crate) fn new(env: JNIEnv<'local>, view: View<'local>) -> Self {
        Self {
            env,
            view,
            deferred_callbacks: SmallVec::new(),
        }
    }

    pub fn push_static_deferred_callback(
        &mut self,
        callback: fn(&mut JNIEnv<'local>, &View<'local>),
    ) {
        self.deferred_callbacks
            .push(DeferredCallback::Static(callback));
    }

    pub fn push_dynamic_deferred_callback(
        &mut self,
        callback: impl 'static + FnOnce(&mut JNIEnv<'local>, &View<'local>),
    ) {
        self.deferred_callbacks
            .push(DeferredCallback::Dynamic(Box::new(callback)));
    }
}

impl CallbackCtx<'_> {
    pub(crate) fn finish(mut self) {
        for callback in self.deferred_callbacks {
            match callback {
                DeferredCallback::Static(f) => f(&mut self.env, &self.view),
                DeferredCallback::Dynamic(f) => f(&mut self.env, &self.view),
            }
        }
    }
}
