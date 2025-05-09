use jni::{JNIEnv, objects::JObject, sys::jfloat};

#[repr(transparent)]
pub struct Context<'local>(pub JObject<'local>);

impl<'local> Context<'local> {
    pub fn resources(&self, env: &mut JNIEnv<'local>) -> Resources<'local> {
        Resources(
            env.call_method(
                &self.0,
                "getResources",
                "()Landroid/content/res/Resources;",
                &[],
            )
            .unwrap()
            .l()
            .unwrap(),
        )
    }

    // TODO: more methods?
}

#[repr(transparent)]
pub struct Resources<'local>(pub JObject<'local>);

impl<'local> Resources<'local> {
    pub fn display_metrics(&self, env: &mut JNIEnv<'local>) -> DisplayMetrics<'local> {
        DisplayMetrics(
            env.call_method(
                &self.0,
                "getDisplayMetrics",
                "()Landroid/util/DisplayMetrics;",
                &[],
            )
            .unwrap()
            .l()
            .unwrap(),
        )
    }
}

#[repr(transparent)]
pub struct DisplayMetrics<'local>(pub JObject<'local>);

impl<'local> DisplayMetrics<'local> {
    pub fn density(&self, env: &mut JNIEnv<'local>) -> jfloat {
        env.get_field(&self.0, "density", "F").unwrap().f().unwrap()
    }
}
