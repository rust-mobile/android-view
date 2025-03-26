use jni::{
    JNIEnv,
    objects::JObject,
    sys::{jfloat, jint, jlong},
};
use ndk::event::{
    KeyAction, KeyEventFlags, Keycode, MetaState, MotionAction, MotionEventFlags, Source,
};
use num_enum::FromPrimitive;

#[repr(transparent)]
pub struct KeyEvent<'local>(pub JObject<'local>);

impl<'local> KeyEvent<'local> {
    pub fn device_id(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getDeviceId", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn source(&self, env: &mut JNIEnv<'local>) -> Source {
        Source::from_primitive(
            env.call_method(&self.0, "getSource", "()I", &[])
                .unwrap()
                .i()
                .unwrap(),
        )
    }

    pub fn action(&self, env: &mut JNIEnv<'local>) -> KeyAction {
        KeyAction::from_primitive(
            env.call_method(&self.0, "getAction", "()I", &[])
                .unwrap()
                .i()
                .unwrap(),
        )
    }

    pub fn event_time(&self, env: &mut JNIEnv<'local>) -> jlong {
        env.call_method(&self.0, "getEventTime", "()J", &[])
            .unwrap()
            .j()
            .unwrap()
    }

    pub fn down_time(&self, env: &mut JNIEnv<'local>) -> jlong {
        env.call_method(&self.0, "getDownTime", "()J", &[])
            .unwrap()
            .j()
            .unwrap()
    }

    pub fn flags(&self, env: &mut JNIEnv<'local>) -> KeyEventFlags {
        KeyEventFlags(
            env.call_method(&self.0, "getFlags", "()I", &[])
                .unwrap()
                .i()
                .unwrap() as u32,
        )
    }

    pub fn meta_state(&self, env: &mut JNIEnv<'local>) -> MetaState {
        MetaState(
            env.call_method(&self.0, "getMetaState", "()I", &[])
                .unwrap()
                .i()
                .unwrap() as u32,
        )
    }

    pub fn repeat_count(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getRepeatCount", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn key_code(&self, env: &mut JNIEnv<'local>) -> Keycode {
        Keycode::from_primitive(
            env.call_method(&self.0, "getKeyCode", "()I", &[])
                .unwrap()
                .i()
                .unwrap(),
        )
    }

    pub fn scan_code(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getScanCode", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn unicode_char(&self, env: &mut JNIEnv<'local>) -> Option<char> {
        let i = env
            .call_method(&self.0, "getUnicodeChar", "()I", &[])
            .unwrap()
            .i()
            .unwrap();
        if i <= 0 {
            return None;
        }
        char::from_u32(i as _)
    }
}

#[repr(transparent)]
pub struct MotionEvent<'local>(pub JObject<'local>);

impl<'local> MotionEvent<'local> {
    pub fn device_id(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getDeviceId", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn source(&self, env: &mut JNIEnv<'local>) -> Source {
        Source::from_primitive(
            env.call_method(&self.0, "getSource", "()I", &[])
                .unwrap()
                .i()
                .unwrap(),
        )
    }

    pub fn action(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getAction", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn action_masked(&self, env: &mut JNIEnv<'local>) -> MotionAction {
        MotionAction::from_primitive(
            env.call_method(&self.0, "getActionMasked", "()I", &[])
                .unwrap()
                .i()
                .unwrap(),
        )
    }

    pub fn action_index(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getActionIndex", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn event_time(&self, env: &mut JNIEnv<'local>) -> jlong {
        env.call_method(&self.0, "getEventTime", "()J", &[])
            .unwrap()
            .j()
            .unwrap()
    }

    pub fn down_time(&self, env: &mut JNIEnv<'local>) -> jlong {
        env.call_method(&self.0, "getDownTime", "()J", &[])
            .unwrap()
            .j()
            .unwrap()
    }

    pub fn flags(&self, env: &mut JNIEnv<'local>) -> MotionEventFlags {
        MotionEventFlags(
            env.call_method(&self.0, "getFlags", "()I", &[])
                .unwrap()
                .i()
                .unwrap() as u32,
        )
    }

    pub fn meta_state(&self, env: &mut JNIEnv<'local>) -> MetaState {
        MetaState(
            env.call_method(&self.0, "getMetaState", "()I", &[])
                .unwrap()
                .i()
                .unwrap() as u32,
        )
    }

    pub fn pointer_count(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getPointerCount", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn pointer_id(&self, env: &mut JNIEnv<'local>, pointer_index: jint) -> jint {
        env.call_method(&self.0, "getPointerId", "(I)I", &[pointer_index.into()])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn x(&self, env: &mut JNIEnv<'local>) -> jfloat {
        env.call_method(&self.0, "getX", "()F", &[])
            .unwrap()
            .f()
            .unwrap()
    }

    pub fn x_at(&self, env: &mut JNIEnv<'local>, pointer_index: jint) -> jfloat {
        env.call_method(&self.0, "getX", "(I)F", &[pointer_index.into()])
            .unwrap()
            .f()
            .unwrap()
    }

    pub fn y(&self, env: &mut JNIEnv<'local>) -> jfloat {
        env.call_method(&self.0, "getY", "()F", &[])
            .unwrap()
            .f()
            .unwrap()
    }

    pub fn y_at(&self, env: &mut JNIEnv<'local>, pointer_index: jint) -> jfloat {
        env.call_method(&self.0, "getY", "(I)F", &[pointer_index.into()])
            .unwrap()
            .f()
            .unwrap()
    }

    pub fn pressure(&self, env: &mut JNIEnv<'local>) -> jfloat {
        env.call_method(&self.0, "getPressure", "()F", &[])
            .unwrap()
            .f()
            .unwrap()
    }

    pub fn pressure_at(&self, env: &mut JNIEnv<'local>, pointer_index: jint) -> jfloat {
        env.call_method(&self.0, "getPressure", "(I)F", &[pointer_index.into()])
            .unwrap()
            .f()
            .unwrap()
    }
}
