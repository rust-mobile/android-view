use jni::{JNIEnv, objects::JObject, sys::jint};

#[repr(transparent)]
pub struct Rect<'local>(pub JObject<'local>);

impl<'local> Rect<'local> {
    pub fn left(&self, env: &mut JNIEnv<'local>) -> jint {
        env.get_field(&self.0, "left", "I").unwrap().i().unwrap()
    }

    pub fn top(&self, env: &mut JNIEnv<'local>) -> jint {
        env.get_field(&self.0, "top", "I").unwrap().i().unwrap()
    }

    pub fn right(&self, env: &mut JNIEnv<'local>) -> jint {
        env.get_field(&self.0, "right", "I").unwrap().i().unwrap()
    }

    pub fn bottom(&self, env: &mut JNIEnv<'local>) -> jint {
        env.get_field(&self.0, "bottom", "I").unwrap().i().unwrap()
    }
}
