use jni::{
    objects::{JIntArray, JObject},
    sys::{jboolean, jfloat, jint, jlong, JNI_FALSE, JNI_TRUE},
    JNIEnv,
};
use ndk::native_window::NativeWindow;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicI64, Ordering},
        Mutex,
    },
};

pub use jni;
pub use ndk;

#[repr(transparent)]
pub struct KeyEvent<'local>(pub JObject<'local>);

impl<'local> KeyEvent<'local> {
    pub fn device_id(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getDeviceId", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn source(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getSource", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn action(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getAction", "()I", &[])
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

    pub fn flags(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getFlags", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn meta_state(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getMetaState", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn modifiers(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getModifiers", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn repeat_count(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getRepeatCount", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn key_code(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getKeyCode", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
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

    pub fn source(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getSource", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn action(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getAction", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn action_masked(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getActionMasked", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
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

    pub fn flags(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getFlags", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn meta_state(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getMetaState", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
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

#[repr(transparent)]
pub struct View<'local>(pub JObject<'local>);

#[allow(unused_variables)]
pub trait ViewCallback: Send {
    fn on_measure<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        width_spec: jint,
        height_spec: jint,
    ) -> Option<(jint, jint)> {
        None
    }

    fn on_layout<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        changed: bool,
        left: jint,
        top: jint,
        right: jint,
        bottom: jint,
    ) {
    }

    fn on_size_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        w: jint,
        h: jint,
        oldw: jint,
        oldh: jint,
    ) {
    }

    fn on_key_down<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        key_code: jint,
        event: &KeyEvent<'local>,
    ) -> bool {
        false
    }

    fn on_key_up<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        key_code: jint,
        event: &KeyEvent<'local>,
    ) -> bool {
        false
    }

    fn on_trackball_event<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        event: &MotionEvent<'local>,
    ) -> bool {
        false
    }

    fn on_touch_event<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        event: &MotionEvent<'local>,
    ) -> bool {
        false
    }

    fn on_generic_motion_event<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        event: &MotionEvent<'local>,
    ) -> bool {
        false
    }

    fn on_hover_event<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        event: &MotionEvent<'local>,
    ) -> bool {
        false
    }

    fn on_focus_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        gain_focus: bool,
        direction: jint,
        previously_focused_rect: Option<&Rect<'local>>,
    ) {
    }

    fn on_window_focus_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        has_window_focus: bool,
    ) {
    }

    fn on_attached_to_window<'local>(&mut self, env: &mut JNIEnv<'local>, view: &View<'local>) {}

    fn on_detached_from_window<'local>(&mut self, env: &mut JNIEnv<'local>, view: &View<'local>) {}

    fn on_window_visibility_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        visibility: jint,
    ) {
    }

    fn surface_created<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        holder: &SurfaceHolder<'local>,
    ) {
    }

    fn surface_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        holder: &SurfaceHolder<'local>,
        format: jint,
        width: jint,
        height: jint,
    ) {
    }

    fn surface_destroyed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        holder: &SurfaceHolder<'local>,
    ) {
    }
}

static NEXT_VIEW_CALLBACK_HANDLE: AtomicI64 = AtomicI64::new(0);
static VIEW_CALLBACK_HANDLE_MAP: Mutex<BTreeMap<jlong, Box<dyn ViewCallback>>> =
    Mutex::new(BTreeMap::new());

fn with_view_callback<F, T>(handle: jlong, f: F) -> T
where
    F: FnOnce(&mut dyn ViewCallback) -> T,
{
    let mut map = VIEW_CALLBACK_HANDLE_MAP.lock().unwrap();
    let callback = map.get_mut(&handle).unwrap();
    f(&mut **callback)
}

extern "system" fn on_measure<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    width_spec: jint,
    height_spec: jint,
) -> JIntArray<'local> {
    with_view_callback(handle, |callback| {
        if let Some((width, height)) = callback.on_measure(&mut env, &view, width_spec, height_spec)
        {
            let result = env.new_int_array(2).unwrap();
            env.set_int_array_region(&result, 0, &[width, height])
                .unwrap();
            result
        } else {
            JObject::null().into()
        }
    })
}

extern "system" fn on_layout<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    changed: jboolean,
    left: jint,
    top: jint,
    right: jint,
    bottom: jint,
) {
    with_view_callback(handle, |callback| {
        callback.on_layout(
            &mut env,
            &view,
            changed == JNI_TRUE,
            left,
            top,
            right,
            bottom,
        );
    })
}

extern "system" fn on_size_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    w: jint,
    h: jint,
    oldw: jint,
    oldh: jint,
) {
    with_view_callback(handle, |callback| {
        callback.on_size_changed(&mut env, &view, w, h, oldw, oldh);
    })
}

fn to_jboolean(flag: bool) -> jboolean {
    if flag {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

extern "system" fn on_key_down<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    key_code: jint,
    event: KeyEvent<'local>,
) -> jboolean {
    with_view_callback(handle, |callback| {
        to_jboolean(callback.on_key_down(&mut env, &view, key_code, &event))
    })
}

extern "system" fn on_key_up<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    key_code: jint,
    event: KeyEvent<'local>,
) -> jboolean {
    with_view_callback(handle, |callback| {
        to_jboolean(callback.on_key_up(&mut env, &view, key_code, &event))
    })
}

extern "system" fn on_trackball_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    with_view_callback(handle, |callback| {
        to_jboolean(callback.on_trackball_event(&mut env, &view, &event))
    })
}

extern "system" fn on_touch_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    with_view_callback(handle, |callback| {
        to_jboolean(callback.on_touch_event(&mut env, &view, &event))
    })
}

extern "system" fn on_generic_motion_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    with_view_callback(handle, |callback| {
        to_jboolean(callback.on_generic_motion_event(&mut env, &view, &event))
    })
}

extern "system" fn on_hover_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    with_view_callback(handle, |callback| {
        to_jboolean(callback.on_hover_event(&mut env, &view, &event))
    })
}

extern "system" fn on_focus_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    gain_focus: jboolean,
    direction: jint,
    previously_focused_rect: Rect<'local>,
) {
    with_view_callback(handle, |callback| {
        callback.on_focus_changed(
            &mut env,
            &view,
            gain_focus == JNI_TRUE,
            direction,
            (!previously_focused_rect.0.as_raw().is_null()).then(|| &previously_focused_rect),
        );
    })
}

extern "system" fn on_window_focus_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    has_window_focus: jboolean,
) {
    with_view_callback(handle, |callback| {
        callback.on_window_focus_changed(&mut env, &view, has_window_focus == JNI_TRUE);
    })
}

extern "system" fn on_attached_to_window<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
) {
    with_view_callback(handle, |callback| {
        callback.on_attached_to_window(&mut env, &view);
    })
}

extern "system" fn on_detached_from_window<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
) {
    let mut map = VIEW_CALLBACK_HANDLE_MAP.lock().unwrap();
    let mut callback = map.remove(&handle).unwrap();
    callback.on_detached_from_window(&mut env, &view);
}

extern "system" fn on_window_visibility_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    visibility: jint,
) {
    with_view_callback(handle, |callback| {
        callback.on_window_visibility_changed(&mut env, &view, visibility);
    })
}

extern "system" fn surface_created<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    holder: SurfaceHolder<'local>,
) {
    with_view_callback(handle, |callback| {
        callback.surface_created(&mut env, &view, &holder);
    })
}

extern "system" fn surface_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    holder: SurfaceHolder<'local>,
    format: jint,
    width: jint,
    height: jint,
) {
    with_view_callback(handle, |callback| {
        callback.surface_changed(&mut env, &view, &holder, format, width, height);
    })
}

extern "system" fn surface_destroyed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    holder: SurfaceHolder<'local>,
) {
    with_view_callback(handle, |callback| {
        callback.surface_destroyed(&mut env, &view, &holder);
    })
}

#[repr(transparent)]
pub struct Context<'local>(pub JObject<'local>);

pub fn new_view<'local, C, F>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    context: Context<'local>,
    callback_factory: F,
) -> jlong
where
    C: ViewCallback + 'static,
    F: FnOnce(&mut JNIEnv<'local>, &View<'local>, &Context<'local>) -> C,
{
    let callback = callback_factory(&mut env, &view, &context);
    let handle = NEXT_VIEW_CALLBACK_HANDLE.fetch_add(1, Ordering::Relaxed);
    let mut map = VIEW_CALLBACK_HANDLE_MAP.lock().unwrap();
    map.insert(handle, Box::new(callback));
    handle
}
