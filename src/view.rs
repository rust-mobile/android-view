use jni::{
    JNIEnv, NativeMethod,
    descriptors::Desc,
    objects::{JClass, JIntArray, JObject},
    sys::{JNI_FALSE, JNI_TRUE, jboolean, jfloat, jint, jlong},
};
use std::{
    collections::BTreeMap,
    ffi::c_void,
    sync::{
        Mutex, Once,
        atomic::{AtomicI64, Ordering},
    },
};

use crate::{context::*, events::*, graphics::*, ime::*, surface::*};

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
}

#[allow(unused_variables)]
pub trait ViewPeer: Send {
    fn on_measure<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        width_spec: jint,
        height_spec: jint,
    ) -> Option<(jint, jint)> {
        None
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

    fn do_frame<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        frame_time_nanos: jlong,
    ) {
    }

    fn delayed_callback<'local>(&mut self, env: &mut JNIEnv<'local>, view: &View<'local>) {}

    fn populate_accessibility_node_info<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        host_screen_x: jint,
        host_screen_y: jint,
        virtual_view_id: jint,
        node_info: &JObject<'local>,
    ) -> bool {
        false
    }

    fn input_focus<'local>(&mut self, env: &mut JNIEnv<'local>, view: &View<'local>) -> jint {
        -1
    }

    fn virtual_view_at_point<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        x: jfloat,
        y: jfloat,
    ) -> jint {
        -1
    }

    fn perform_accessibility_action<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        virtual_view_id: jint,
        action: jint,
    ) -> bool {
        false
    }

    fn accessibility_set_text_selection<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        virtual_view_id: jint,
        anchor: jint,
        focus: jint,
    ) -> bool {
        false
    }

    fn accessibility_collapse_text_selection<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        virtual_view_id: jint,
    ) -> bool {
        false
    }

    fn accessibility_traverse_text<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        virtual_view_id: jint,
        granularity: jint,
        forward: bool,
        extend_selection: bool,
    ) -> bool {
        false
    }

    fn as_input_connection<'a>(&'a mut self) -> Option<&'a mut dyn InputConnection> {
        None
    }
}

static NEXT_PEER_ID: AtomicI64 = AtomicI64::new(0);
static PEER_MAP: Mutex<BTreeMap<jlong, Box<dyn ViewPeer>>> = Mutex::new(BTreeMap::new());

fn with_peer<F, T>(id: jlong, f: F) -> T
where
    F: FnOnce(&mut dyn ViewPeer) -> T,
{
    let mut map = PEER_MAP.lock().unwrap();
    let peer = map.get_mut(&id).unwrap();
    f(&mut **peer)
}

extern "system" fn on_measure<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    width_spec: jint,
    height_spec: jint,
) -> JIntArray<'local> {
    with_peer(peer, |peer| {
        if let Some((width, height)) = peer.on_measure(&mut env, &view, width_spec, height_spec) {
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
    peer: jlong,
    changed: jboolean,
    left: jint,
    top: jint,
    right: jint,
    bottom: jint,
) {
    with_peer(peer, |peer| {
        peer.on_layout(
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
    peer: jlong,
    w: jint,
    h: jint,
    oldw: jint,
    oldh: jint,
) {
    with_peer(peer, |peer| {
        peer.on_size_changed(&mut env, &view, w, h, oldw, oldh);
    })
}

fn to_jboolean(flag: bool) -> jboolean {
    if flag { JNI_TRUE } else { JNI_FALSE }
}

extern "system" fn on_key_down<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    key_code: jint,
    event: KeyEvent<'local>,
) -> jboolean {
    with_peer(peer, |peer| {
        to_jboolean(peer.on_key_down(&mut env, &view, key_code, &event))
    })
}

extern "system" fn on_key_up<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    key_code: jint,
    event: KeyEvent<'local>,
) -> jboolean {
    with_peer(peer, |peer| {
        to_jboolean(peer.on_key_up(&mut env, &view, key_code, &event))
    })
}

extern "system" fn on_trackball_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    with_peer(peer, |peer| {
        to_jboolean(peer.on_trackball_event(&mut env, &view, &event))
    })
}

extern "system" fn on_touch_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    with_peer(peer, |peer| {
        to_jboolean(peer.on_touch_event(&mut env, &view, &event))
    })
}

extern "system" fn on_generic_motion_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    with_peer(peer, |peer| {
        to_jboolean(peer.on_generic_motion_event(&mut env, &view, &event))
    })
}

extern "system" fn on_focus_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    gain_focus: jboolean,
    direction: jint,
    previously_focused_rect: Rect<'local>,
) {
    with_peer(peer, |peer| {
        peer.on_focus_changed(
            &mut env,
            &view,
            gain_focus == JNI_TRUE,
            direction,
            (!previously_focused_rect.0.as_raw().is_null()).then_some(&previously_focused_rect),
        );
    })
}

extern "system" fn on_window_focus_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    has_window_focus: jboolean,
) {
    with_peer(peer, |peer| {
        peer.on_window_focus_changed(&mut env, &view, has_window_focus == JNI_TRUE);
    })
}

extern "system" fn on_attached_to_window<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
) {
    with_peer(peer, |peer| {
        peer.on_attached_to_window(&mut env, &view);
    })
}

extern "system" fn on_detached_from_window<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
) {
    let mut map = PEER_MAP.lock().unwrap();
    let mut peer = map.remove(&peer).unwrap();
    peer.on_detached_from_window(&mut env, &view);
    view.remove_frame_callback(&mut env);
    view.remove_delayed_callbacks(&mut env);
}

extern "system" fn on_window_visibility_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    visibility: jint,
) {
    with_peer(peer, |peer| {
        peer.on_window_visibility_changed(&mut env, &view, visibility);
    })
}

extern "system" fn surface_created<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    holder: SurfaceHolder<'local>,
) {
    with_peer(peer, |peer| {
        peer.surface_created(&mut env, &view, &holder);
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
    with_peer(peer, |peer| {
        peer.surface_changed(&mut env, &view, &holder, format, width, height);
    })
}

extern "system" fn surface_destroyed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    holder: SurfaceHolder<'local>,
) {
    with_peer(peer, |peer| {
        peer.surface_destroyed(&mut env, &view, &holder);
    })
}

extern "system" fn do_frame<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    frame_time_nanos: jlong,
) {
    with_peer(peer, |peer| {
        peer.do_frame(&mut env, &view, frame_time_nanos);
    })
}

extern "system" fn delayed_callback<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
) {
    with_peer(peer, |peer| {
        peer.delayed_callback(&mut env, &view);
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
    with_peer(peer, |peer| {
        if peer.populate_accessibility_node_info(
            &mut env,
            &view,
            host_screen_x,
            host_screen_y,
            virtual_view_id,
            &node_info,
        ) {
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    })
}

extern "system" fn get_input_focus<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
) -> jint {
    with_peer(peer, |peer| peer.input_focus(&mut env, &view))
}

extern "system" fn get_virtual_view_at_point<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    x: jfloat,
    y: jfloat,
) -> jint {
    with_peer(peer, |peer| {
        peer.virtual_view_at_point(&mut env, &view, x, y)
    })
}

extern "system" fn perform_accessibility_action<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    virtual_view_id: jint,
    action: jint,
) -> jboolean {
    with_peer(peer, |peer| {
        if peer.perform_accessibility_action(&mut env, &view, virtual_view_id, action) {
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    })
}

extern "system" fn accessibility_set_text_selection<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    virtual_view_id: jint,
    anchor: jint,
    focus: jint,
) -> jboolean {
    with_peer(peer, |peer| {
        if peer.accessibility_set_text_selection(&mut env, &view, virtual_view_id, anchor, focus) {
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    })
}

extern "system" fn accessibility_collapse_text_selection<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    virtual_view_id: jint,
) -> jboolean {
    with_peer(peer, |peer| {
        if peer.accessibility_collapse_text_selection(&mut env, &view, virtual_view_id) {
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    })
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
    with_peer(peer, |peer| {
        if peer.accessibility_traverse_text(
            &mut env,
            &view,
            virtual_view_id,
            granularity,
            forward == JNI_TRUE,
            extend_selection == JNI_TRUE,
        ) {
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    })
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
            "org/linebender/android/RustView",
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
