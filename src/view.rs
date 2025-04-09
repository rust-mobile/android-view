use jni::{
    JNIEnv, NativeMethod,
    descriptors::Desc,
    objects::{JClass, JIntArray, JObject},
    sys::{JNI_TRUE, jboolean, jfloat, jint, jlong},
};
use ndk::event::Keycode;
use num_enum::FromPrimitive;
use std::{
    collections::BTreeMap,
    ffi::c_void,
    sync::{
        Mutex, Once,
        atomic::{AtomicI64, Ordering},
    },
};

use crate::{
    binder::*, context::*, events::*, graphics::*, ime::*, peer_result::*, surface::*, util::*,
};

#[repr(transparent)]
pub struct View<'local>(pub JObject<'local>);

impl<'local> View<'local> {
    pub fn post_frame_callback(&self, env: &mut JNIEnv<'local>) {
        env.call_method(&self.0, "postFrameCallback", "()V", &[])
            .unwrap()
            .v()
            .unwrap()
    }

    pub fn remove_frame_callback(&self, env: &mut JNIEnv<'local>) {
        env.call_method(&self.0, "removeFrameCallback", "()V", &[])
            .unwrap()
            .v()
            .unwrap()
    }

    pub fn post_delayed(&self, env: &mut JNIEnv<'local>, delay_millis: jlong) -> bool {
        env.call_method(&self.0, "postDelayed", "(J)Z", &[delay_millis.into()])
            .unwrap()
            .z()
            .unwrap()
    }

    pub fn remove_delayed_callbacks(&self, env: &mut JNIEnv<'local>) -> bool {
        env.call_method(&self.0, "removeDelayedCallbacks", "()Z", &[])
            .unwrap()
            .z()
            .unwrap()
    }

    pub fn is_focused(&self, env: &mut JNIEnv<'local>) -> bool {
        env.call_method(&self.0, "isFocused", "()Z", &[])
            .unwrap()
            .z()
            .unwrap()
    }

    pub fn input_method_manager(&self, env: &mut JNIEnv<'local>) -> InputMethodManager<'local> {
        InputMethodManager(
            env.get_field(
                &self.0,
                "mInputMethodManager",
                "Landroid/view/inputmethod/InputMethodManager;",
            )
            .unwrap()
            .l()
            .unwrap(),
        )
    }

    pub fn window_token(&self, env: &mut JNIEnv<'local>) -> IBinder<'local> {
        IBinder(
            env.call_method(&self.0, "getWindowToken", "()Landroid/os/IBinder;", &[])
                .unwrap()
                .l()
                .unwrap(),
        )
    }
}

#[allow(unused_variables)]
pub trait ViewPeer: Send {
    fn on_measure<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        width_spec: jint,
        height_spec: jint,
    ) -> PeerResult<'local, Option<(jint, jint)>> {
        None.into()
    }

    #[allow(clippy::too_many_arguments)]
    fn on_layout<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        changed: bool,
        left: jint,
        top: jint,
        right: jint,
        bottom: jint,
    ) -> PeerResult<'local, ()> {
        ().into()
    }

    fn on_size_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        w: jint,
        h: jint,
        oldw: jint,
        oldh: jint,
    ) -> PeerResult<'local, ()> {
        ().into()
    }

    fn on_key_down<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        key_code: Keycode,
        event: &KeyEvent<'local>,
    ) -> PeerResult<'local, bool> {
        false.into()
    }

    fn on_key_up<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        key_code: Keycode,
        event: &KeyEvent<'local>,
    ) -> PeerResult<'local, bool> {
        false.into()
    }

    fn on_trackball_event<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        event: &MotionEvent<'local>,
    ) -> PeerResult<'local, bool> {
        false.into()
    }

    fn on_touch_event<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        event: &MotionEvent<'local>,
    ) -> PeerResult<'local, bool> {
        false.into()
    }

    fn on_generic_motion_event<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        event: &MotionEvent<'local>,
    ) -> PeerResult<'local, bool> {
        false.into()
    }

    fn on_focus_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        gain_focus: bool,
        direction: jint,
        previously_focused_rect: Option<&Rect<'local>>,
    ) -> PeerResult<'local, ()> {
        ().into()
    }

    fn on_window_focus_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        has_window_focus: bool,
    ) -> PeerResult<'local, ()> {
        ().into()
    }

    fn on_attached_to_window<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
    ) -> PeerResult<'local, ()> {
        ().into()
    }

    fn on_detached_from_window<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
    ) -> PeerResult<'local, ()> {
        ().into()
    }

    fn on_window_visibility_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        visibility: jint,
    ) -> PeerResult<'local, ()> {
        ().into()
    }

    fn surface_created<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        holder: &SurfaceHolder<'local>,
    ) -> PeerResult<'local, ()> {
        ().into()
    }

    fn surface_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        holder: &SurfaceHolder<'local>,
        format: jint,
        width: jint,
        height: jint,
    ) -> PeerResult<'local, ()> {
        ().into()
    }

    fn surface_destroyed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        holder: &SurfaceHolder<'local>,
    ) -> PeerResult<'local, ()> {
        ().into()
    }

    fn do_frame<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        frame_time_nanos: jlong,
    ) -> PeerResult<'local, ()> {
        ().into()
    }

    fn delayed_callback<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
    ) -> PeerResult<'local, ()> {
        ().into()
    }

    fn populate_accessibility_node_info<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        host_screen_x: jint,
        host_screen_y: jint,
        virtual_view_id: jint,
        node_info: &JObject<'local>,
    ) -> PeerResult<'local, bool> {
        false.into()
    }

    fn input_focus<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
    ) -> PeerResult<'local, jint> {
        (-1).into()
    }

    fn virtual_view_at_point<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        x: jfloat,
        y: jfloat,
    ) -> PeerResult<'local, jint> {
        (-1).into()
    }

    fn perform_accessibility_action<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        virtual_view_id: jint,
        action: jint,
    ) -> PeerResult<'local, bool> {
        false.into()
    }

    fn accessibility_set_text_selection<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        virtual_view_id: jint,
        anchor: jint,
        focus: jint,
    ) -> PeerResult<'local, bool> {
        false.into()
    }

    fn accessibility_collapse_text_selection<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        virtual_view_id: jint,
    ) -> PeerResult<'local, bool> {
        false.into()
    }

    fn accessibility_traverse_text<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        virtual_view_id: jint,
        granularity: jint,
        forward: bool,
        extend_selection: bool,
    ) -> PeerResult<'local, bool> {
        false.into()
    }

    fn on_create_input_connection<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        out_attrs: &EditorInfo<'local>,
    ) -> PeerResult<'local, bool> {
        false.into()
    }

    fn as_input_connection(&mut self) -> &mut dyn InputConnection {
        unimplemented!()
    }
}

static NEXT_PEER_ID: AtomicI64 = AtomicI64::new(0);
static PEER_MAP: Mutex<BTreeMap<jlong, Box<dyn ViewPeer>>> = Mutex::new(BTreeMap::new());

pub(crate) fn with_peer_and_default<'local, F, T>(
    env: &mut JNIEnv<'local>,
    view: &View<'local>,
    id: jlong,
    default: T,
    f: F,
) -> T
where
    F: FnOnce(&mut JNIEnv<'local>, &View<'local>, &mut dyn ViewPeer) -> PeerResult<'local, T>,
{
    let mut map = PEER_MAP.lock().unwrap();
    let Some(peer) = map.get_mut(&id) else {
        return default;
    };
    let result = f(env, view, &mut **peer);
    drop(map);
    result.finish(env, view)
}

fn with_peer<'local, F, T: Default>(
    env: &mut JNIEnv<'local>,
    view: &View<'local>,
    id: jlong,
    f: F,
) -> T
where
    F: FnOnce(&mut JNIEnv<'local>, &View<'local>, &mut dyn ViewPeer) -> PeerResult<'local, T>,
{
    with_peer_and_default(env, view, id, T::default(), f)
}

extern "system" fn on_measure<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    width_spec: jint,
    height_spec: jint,
) -> JIntArray<'local> {
    if let Some((width, height)) = with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_measure(env, view, width_spec, height_spec)
    }) {
        let result = env.new_int_array(2).unwrap();
        env.set_int_array_region(&result, 0, &[width, height])
            .unwrap();
        result
    } else {
        JObject::null().into()
    }
}

extern "system" fn on_layout<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    changed: jboolean,
    left: jint,
    top: jint,
    right: jint,
    bottom: jint,
) {
    with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_layout(env, view, changed == JNI_TRUE, left, top, right, bottom)
    })
}

extern "system" fn on_size_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    w: jint,
    h: jint,
    oldw: jint,
    oldh: jint,
) {
    with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_size_changed(env, view, w, h, oldw, oldh)
    })
}

extern "system" fn on_key_down<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    key_code: jint,
    event: KeyEvent<'local>,
) -> jboolean {
    to_jboolean(with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_key_down(env, view, Keycode::from_primitive(key_code), &event)
    }))
}

extern "system" fn on_key_up<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    key_code: jint,
    event: KeyEvent<'local>,
) -> jboolean {
    to_jboolean(with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_key_up(env, view, Keycode::from_primitive(key_code), &event)
    }))
}

extern "system" fn on_trackball_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    to_jboolean(with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_trackball_event(env, view, &event)
    }))
}

extern "system" fn on_touch_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    to_jboolean(with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_touch_event(env, view, &event)
    }))
}

extern "system" fn on_generic_motion_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    to_jboolean(with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_generic_motion_event(env, view, &event)
    }))
}

extern "system" fn on_focus_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    gain_focus: jboolean,
    direction: jint,
    previously_focused_rect: Rect<'local>,
) {
    with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_focus_changed(
            env,
            view,
            gain_focus == JNI_TRUE,
            direction,
            (!previously_focused_rect.0.as_raw().is_null()).then_some(&previously_focused_rect),
        )
    })
}

extern "system" fn on_window_focus_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    has_window_focus: jboolean,
) {
    with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_window_focus_changed(env, view, has_window_focus == JNI_TRUE)
    })
}

extern "system" fn on_attached_to_window<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
) {
    with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_attached_to_window(env, view)
    })
}

extern "system" fn on_detached_from_window<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
) {
    let mut map = PEER_MAP.lock().unwrap();
    let mut peer = map.remove(&peer).unwrap();
    let result = peer.on_detached_from_window(&mut env, &view);
    drop(map);
    result.finish(&mut env, &view);
    view.remove_frame_callback(&mut env);
    view.remove_delayed_callbacks(&mut env);
}

extern "system" fn on_window_visibility_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    visibility: jint,
) {
    with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_window_visibility_changed(env, view, visibility)
    })
}

extern "system" fn surface_created<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    holder: SurfaceHolder<'local>,
) {
    with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.surface_created(env, view, &holder)
    })
}

extern "system" fn surface_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    holder: SurfaceHolder<'local>,
    format: jint,
    width: jint,
    height: jint,
) {
    with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.surface_changed(env, view, &holder, format, width, height)
    })
}

extern "system" fn surface_destroyed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    holder: SurfaceHolder<'local>,
) {
    with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.surface_destroyed(env, view, &holder)
    })
}

extern "system" fn do_frame<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    frame_time_nanos: jlong,
) {
    with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.do_frame(env, view, frame_time_nanos)
    })
}

extern "system" fn delayed_callback<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
) {
    with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.delayed_callback(env, view)
    })
}

extern "system" fn populate_accessibility_node_info<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    host_screen_x: jint,
    host_screen_y: jint,
    virtual_view_id: jint,
    node_info: JObject<'local>,
) -> jboolean {
    to_jboolean(with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.populate_accessibility_node_info(
            env,
            view,
            host_screen_x,
            host_screen_y,
            virtual_view_id,
            &node_info,
        )
    }))
}

extern "system" fn get_input_focus<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
) -> jint {
    with_peer_and_default(&mut env, &view, peer, -1, |env, view, peer| {
        peer.input_focus(env, view)
    })
}

extern "system" fn get_virtual_view_at_point<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    x: jfloat,
    y: jfloat,
) -> jint {
    with_peer_and_default(&mut env, &view, peer, -1, |env, view, peer| {
        peer.virtual_view_at_point(env, view, x, y)
    })
}

extern "system" fn perform_accessibility_action<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    virtual_view_id: jint,
    action: jint,
) -> jboolean {
    to_jboolean(with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.perform_accessibility_action(env, view, virtual_view_id, action)
    }))
}

extern "system" fn accessibility_set_text_selection<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    virtual_view_id: jint,
    anchor: jint,
    focus: jint,
) -> jboolean {
    to_jboolean(with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.accessibility_set_text_selection(env, view, virtual_view_id, anchor, focus)
    }))
}

extern "system" fn accessibility_collapse_text_selection<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    virtual_view_id: jint,
) -> jboolean {
    to_jboolean(with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.accessibility_collapse_text_selection(env, view, virtual_view_id)
    }))
}

extern "system" fn accessibility_traverse_text<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    virtual_view_id: jint,
    granularity: jint,
    forward: jboolean,
    extend_selection: jboolean,
) -> jboolean {
    to_jboolean(with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.accessibility_traverse_text(
            env,
            view,
            virtual_view_id,
            granularity,
            forward == JNI_TRUE,
            extend_selection == JNI_TRUE,
        )
    }))
}

extern "system" fn on_create_input_connection<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    out_attrs: EditorInfo<'local>,
) -> jboolean {
    to_jboolean(with_peer(&mut env, &view, peer, |env, view, peer| {
        peer.on_create_input_connection(env, view, &out_attrs)
    }))
}

pub fn register_view_peer(peer: impl 'static + ViewPeer) -> jlong {
    let id = NEXT_PEER_ID.fetch_add(1, Ordering::Relaxed);
    let mut map = PEER_MAP.lock().unwrap();
    map.insert(id, Box::new(peer));
    id
}

pub fn register_view_class<'local, 'other_local>(
    env: &mut JNIEnv<'local>,
    class: impl Desc<'local, JClass<'other_local>>,
    new_peer: for<'a> extern "system" fn(JNIEnv<'a>, View<'a>, Context<'a>) -> jlong,
) {
    static REGISTER_BASE_NATIVES: Once = Once::new();
    REGISTER_BASE_NATIVES.call_once(|| {
        env.register_native_methods(
            "org/linebender/android/rustview/RustView",
            &[
                NativeMethod {
                    name: "onMeasureNative".into(),
                    sig: "(JII)[I".into(),
                    fn_ptr: on_measure as *mut c_void,
                },
                NativeMethod {
                    name: "onLayoutNative".into(),
                    sig: "(JZIIII)V".into(),
                    fn_ptr: on_layout as *mut c_void,
                },
                NativeMethod {
                    name: "onSizeChangedNative".into(),
                    sig: "(JIIII)V".into(),
                    fn_ptr: on_size_changed as *mut c_void,
                },
                NativeMethod {
                    name: "onKeyDownNative".into(),
                    sig: "(JILandroid/view/KeyEvent;)Z".into(),
                    fn_ptr: on_key_down as *mut c_void,
                },
                NativeMethod {
                    name: "onKeyUpNative".into(),
                    sig: "(JILandroid/view/KeyEvent;)Z".into(),
                    fn_ptr: on_key_up as *mut c_void,
                },
                NativeMethod {
                    name: "onTrackballEventNative".into(),
                    sig: "(JLandroid/view/MotionEvent;)Z".into(),
                    fn_ptr: on_trackball_event as *mut c_void,
                },
                NativeMethod {
                    name: "onTouchEventNative".into(),
                    sig: "(JLandroid/view/MotionEvent;)Z".into(),
                    fn_ptr: on_touch_event as *mut c_void,
                },
                NativeMethod {
                    name: "onGenericMotionEventNative".into(),
                    sig: "(JLandroid/view/MotionEvent;)Z".into(),
                    fn_ptr: on_generic_motion_event as *mut c_void,
                },
                NativeMethod {
                    name: "onFocusChangedNative".into(),
                    sig: "(JZILandroid/graphics/Rect;)V".into(),
                    fn_ptr: on_focus_changed as *mut c_void,
                },
                NativeMethod {
                    name: "onWindowFocusChangedNative".into(),
                    sig: "(JZ)V".into(),
                    fn_ptr: on_window_focus_changed as *mut c_void,
                },
                NativeMethod {
                    name: "onAttachedToWindowNative".into(),
                    sig: "(J)V".into(),
                    fn_ptr: on_attached_to_window as *mut c_void,
                },
                NativeMethod {
                    name: "onDetachedFromWindowNative".into(),
                    sig: "(J)V".into(),
                    fn_ptr: on_detached_from_window as *mut c_void,
                },
                NativeMethod {
                    name: "onWindowVisibilityChangedNative".into(),
                    sig: "(JI)V".into(),
                    fn_ptr: on_window_visibility_changed as *mut c_void,
                },
                NativeMethod {
                    name: "surfaceCreatedNative".into(),
                    sig: "(JLandroid/view/SurfaceHolder;)V".into(),
                    fn_ptr: surface_created as *mut c_void,
                },
                NativeMethod {
                    name: "surfaceChangedNative".into(),
                    sig: "(JLandroid/view/SurfaceHolder;III)V".into(),
                    fn_ptr: surface_changed as *mut c_void,
                },
                NativeMethod {
                    name: "surfaceDestroyedNative".into(),
                    sig: "(JLandroid/view/SurfaceHolder;)V".into(),
                    fn_ptr: surface_destroyed as *mut c_void,
                },
                NativeMethod {
                    name: "doFrameNative".into(),
                    sig: "(JJ)V".into(),
                    fn_ptr: do_frame as *mut c_void,
                },
                NativeMethod {
                    name: "delayedCallbackNative".into(),
                    sig: "(J)V".into(),
                    fn_ptr: delayed_callback as *mut c_void,
                },
                NativeMethod {
                    name: "populateAccessibilityNodeInfoNative".into(),
                    sig: "(JIIILandroid/view/accessibility/AccessibilityNodeInfo;)Z".into(),
                    fn_ptr: populate_accessibility_node_info as *mut c_void,
                },
                NativeMethod {
                    name: "getInputFocusNative".into(),
                    sig: "(J)I".into(),
                    fn_ptr: get_input_focus as *mut c_void,
                },
                NativeMethod {
                    name: "getVirtualViewAtPointNative".into(),
                    sig: "(JFF)I".into(),
                    fn_ptr: get_virtual_view_at_point as *mut c_void,
                },
                NativeMethod {
                    name: "performAccessibilityActionNative".into(),
                    sig: "(JII)Z".into(),
                    fn_ptr: perform_accessibility_action as *mut c_void,
                },
                NativeMethod {
                    name: "accessibilitySetTextSelectionNative".into(),
                    sig: "(JIII)Z".into(),
                    fn_ptr: accessibility_set_text_selection as *mut c_void,
                },
                NativeMethod {
                    name: "accessibilityCollapseTextSelectionNative".into(),
                    sig: "(JI)Z".into(),
                    fn_ptr: accessibility_collapse_text_selection as *mut c_void,
                },
                NativeMethod {
                    name: "accessibilityTraverseTextNative".into(),
                    sig: "(JIIZZ)Z".into(),
                    fn_ptr: accessibility_traverse_text as *mut c_void,
                },
                NativeMethod {
                    name: "onCreateInputConnectionNative".into(),
                    sig: "(JLandroid/view/inputmethod/EditorInfo;)Z".into(),
                    fn_ptr: on_create_input_connection as *mut c_void,
                },
                NativeMethod {
                    name: "getTextBeforeCursorNative".into(),
                    sig: "(JI)Ljava/lang/String;".into(),
                    fn_ptr: get_text_before_cursor as *mut c_void,
                },
                NativeMethod {
                    name: "getTextAfterCursorNative".into(),
                    sig: "(JI)Ljava/lang/String;".into(),
                    fn_ptr: get_text_after_cursor as *mut c_void,
                },
                NativeMethod {
                    name: "getSelectedTextNative".into(),
                    sig: "(J)Ljava/lang/String;".into(),
                    fn_ptr: get_selected_text as *mut c_void,
                },
                NativeMethod {
                    name: "getCursorCapsModeNative".into(),
                    sig: "(JI)I".into(),
                    fn_ptr: get_cursor_caps_mode as *mut c_void,
                },
                NativeMethod {
                    name: "deleteSurroundingTextNative".into(),
                    sig: "(JII)Z".into(),
                    fn_ptr: delete_surrounding_text as *mut c_void,
                },
                NativeMethod {
                    name: "deleteSurroundingTextInCodePointsNative".into(),
                    sig: "(JII)Z".into(),
                    fn_ptr: delete_surrounding_text_in_code_points as *mut c_void,
                },
                NativeMethod {
                    name: "setComposingTextNative".into(),
                    sig: "(JLjava/lang/String;I)Z".into(),
                    fn_ptr: set_composing_text as *mut c_void,
                },
                NativeMethod {
                    name: "setComposingRegionNative".into(),
                    sig: "(JII)Z".into(),
                    fn_ptr: set_composing_region as *mut c_void,
                },
                NativeMethod {
                    name: "finishComposingTextNative".into(),
                    sig: "(J)Z".into(),
                    fn_ptr: finish_composing_text as *mut c_void,
                },
                NativeMethod {
                    name: "commitTextNative".into(),
                    sig: "(JLjava/lang/String;I)Z".into(),
                    fn_ptr: commit_text as *mut c_void,
                },
                NativeMethod {
                    name: "setSelectionNative".into(),
                    sig: "(JII)Z".into(),
                    fn_ptr: set_selection as *mut c_void,
                },
                NativeMethod {
                    name: "performEditorActionNative".into(),
                    sig: "(JI)Z".into(),
                    fn_ptr: perform_editor_action as *mut c_void,
                },
                NativeMethod {
                    name: "performContextMenuActionNative".into(),
                    sig: "(JI)Z".into(),
                    fn_ptr: perform_context_menu_action as *mut c_void,
                },
                NativeMethod {
                    name: "beginBatchEditNative".into(),
                    sig: "(J)Z".into(),
                    fn_ptr: begin_batch_edit as *mut c_void,
                },
                NativeMethod {
                    name: "endBatchEditNative".into(),
                    sig: "(J)Z".into(),
                    fn_ptr: end_batch_edit as *mut c_void,
                },
                NativeMethod {
                    name: "inputConnectionSendKeyEventNative".into(),
                    sig: "(JLandroid/view/KeyEvent;)Z".into(),
                    fn_ptr: input_connection_send_key_event as *mut c_void,
                },
                NativeMethod {
                    name: "inputConnectionClearMetaKeyStatesNative".into(),
                    sig: "(JI)Z".into(),
                    fn_ptr: input_connection_clear_meta_key_states as *mut c_void,
                },
                NativeMethod {
                    name: "inputConnectionReportFullscreenModeNative".into(),
                    sig: "(JZ)Z".into(),
                    fn_ptr: input_connection_report_fullscreen_mode as *mut c_void,
                },
                NativeMethod {
                    name: "requestCursorUpdatesNative".into(),
                    sig: "(JI)Z".into(),
                    fn_ptr: request_cursor_updates as *mut c_void,
                },
                NativeMethod {
                    name: "closeInputConnectionNative".into(),
                    sig: "(J)V".into(),
                    fn_ptr: close_input_connection as *mut c_void,
                },
            ],
        )
        .unwrap();
    });
    env.register_native_methods(
        class,
        &[NativeMethod {
            name: "newViewPeer".into(),
            sig: "(Landroid/content/Context;)J".into(),
            fn_ptr: new_peer as *mut c_void,
        }],
    )
    .unwrap();
}
