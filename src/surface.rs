use jni::{JNIEnv, objects::JObject};
use ndk::native_window::NativeWindow;

#[repr(transparent)]
pub struct Surface<'local>(pub JObject<'local>);

impl<'local> Surface<'local> {
    pub fn to_native_window(&self, env: &mut JNIEnv<'local>) -> NativeWindow {
        unsafe { NativeWindow::from_surface(env.get_raw(), self.0.as_raw()) }.unwrap()
    }
}

#[repr(transparent)]
pub struct SurfaceHolder<'local>(pub JObject<'local>);

impl<'local> SurfaceHolder<'local> {
    pub fn surface(&self, env: &mut JNIEnv<'local>) -> Surface<'local> {
        Surface(
            env.call_method(&self.0, "getSurface", "()Landroid/view/Surface;", &[])
                .unwrap()
                .l()
                .unwrap(),
        )
    }
}
