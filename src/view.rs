use jni::{
    JNIEnv, NativeMethod,
    descriptors::Desc,
    objects::{JClass, JIntArray, JObject},
    sys::{JNI_TRUE, jboolean, jint, jlong},
};
use ndk::event::Keycode;
use num_enum::FromPrimitive;
use send_wrapper::SendWrapper;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    ffi::c_void,
    rc::Rc,
    sync::{
        Mutex, Once,
        atomic::{AtomicI64, Ordering},
    },
};

use crate::{
    accessibility::*, binder::*, callback_ctx::*, context::*, events::*, graphics::*, ime::*,
    surface::*, util::*, view_configuration::*,
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

    pub fn context(&self, env: &mut JNIEnv<'local>) -> Context<'local> {
        Context(
            env.call_method(&self.0, "getContext", "()Landroid/content/Context;", &[])
                .unwrap()
                .l()
                .unwrap(),
        )
    }

    pub fn view_configuration(&self, env: &mut JNIEnv<'local>) -> ViewConfiguration {
        ViewConfiguration::new(&self.0, env)
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
pub trait ViewPeer {
    fn on_measure(
        &mut self,
        ctx: &mut CallbackCtx,
        width_spec: jint,
        height_spec: jint,
    ) -> Option<(jint, jint)> {
        None
    }

    fn on_layout(
        &mut self,
        ctx: &mut CallbackCtx,
        changed: bool,
        left: jint,
        top: jint,
        right: jint,
        bottom: jint,
    ) {
    }

    fn on_size_changed(&mut self, ctx: &mut CallbackCtx, w: jint, h: jint, oldw: jint, oldh: jint) {
    }

    fn on_key_down<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        key_code: Keycode,
        event: &KeyEvent<'local>,
    ) -> bool {
        false
    }

    fn on_key_up<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        key_code: Keycode,
        event: &KeyEvent<'local>,
    ) -> bool {
        false
    }

    fn on_trackball_event<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        event: &MotionEvent<'local>,
    ) -> bool {
        false
    }

    fn on_touch_event<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        event: &MotionEvent<'local>,
    ) -> bool {
        false
    }

    fn on_generic_motion_event<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        event: &MotionEvent<'local>,
    ) -> bool {
        false
    }

    fn on_hover_event<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        event: &MotionEvent<'local>,
    ) -> bool {
        false
    }

    fn on_focus_changed<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        gain_focus: bool,
        direction: jint,
        previously_focused_rect: Option<&Rect<'local>>,
    ) {
    }

    fn on_window_focus_changed(&mut self, ctx: &mut CallbackCtx, has_window_focus: bool) {}

    fn on_attached_to_window(&mut self, ctx: &mut CallbackCtx) {}

    fn on_detached_from_window(&mut self, ctx: &mut CallbackCtx) {}

    fn on_window_visibility_changed(&mut self, ctx: &mut CallbackCtx, visibility: jint) {}

    fn surface_created<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        holder: &SurfaceHolder<'local>,
    ) {
    }

    fn surface_changed<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        holder: &SurfaceHolder<'local>,
        format: jint,
        width: jint,
        height: jint,
    ) {
    }

    fn surface_destroyed<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        holder: &SurfaceHolder<'local>,
    ) {
    }

    fn do_frame(&mut self, ctx: &mut CallbackCtx, frame_time_nanos: jlong) {}

    fn delayed_callback(&mut self, ctx: &mut CallbackCtx) {}

    fn as_accessibility_node_provider(&mut self) -> Option<&mut dyn AccessibilityNodeProvider> {
        None
    }

    fn as_input_connection(&mut self) -> Option<&mut dyn InputConnection> {
        None
    }
}

static NEXT_PEER_ID: AtomicI64 = AtomicI64::new(0);
static PEER_MAP: Mutex<BTreeMap<jlong, SendWrapper<Rc<RefCell<Box<dyn ViewPeer>>>>>> =
    Mutex::new(BTreeMap::new());

pub(crate) fn with_peer<'local, F, T: Default>(
    env: JNIEnv<'local>,
    view: View<'local>,
    id: jlong,
    f: F,
) -> T
where
    F: FnOnce(&mut CallbackCtx<'local>, &mut dyn ViewPeer) -> T,
{
    let map = PEER_MAP.lock().unwrap();
    let Some(peer) = map.get(&id) else {
        return T::default();
    };
    let peer = Rc::clone(&**peer);
    drop(map);
    let mut peer = peer.borrow_mut();
    let mut ctx = CallbackCtx::new(env, view);
    let result = f(&mut ctx, &mut **peer);
    drop(peer);
    ctx.finish();
    result
}

extern "system" fn on_measure<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    width_spec: jint,
    height_spec: jint,
) -> JIntArray<'local> {
    with_peer(env, view, peer, |ctx, peer| {
        if let Some((width, height)) = peer.on_measure(ctx, width_spec, height_spec) {
            let result = ctx.env.new_int_array(2).unwrap();
            ctx.env
                .set_int_array_region(&result, 0, &[width, height])
                .unwrap();
            result
        } else {
            JObject::null().into()
        }
    })
}

extern "system" fn on_layout<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    changed: jboolean,
    left: jint,
    top: jint,
    right: jint,
    bottom: jint,
) {
    with_peer(env, view, peer, |ctx, peer| {
        peer.on_layout(ctx, changed == JNI_TRUE, left, top, right, bottom);
    })
}

extern "system" fn on_size_changed<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    w: jint,
    h: jint,
    oldw: jint,
    oldh: jint,
) {
    with_peer(env, view, peer, |ctx, peer| {
        peer.on_size_changed(ctx, w, h, oldw, oldh);
    })
}

extern "system" fn on_key_down<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    key_code: jint,
    event: KeyEvent<'local>,
) -> jboolean {
    as_jboolean(with_peer(env, view, peer, |ctx, peer| {
        peer.on_key_down(ctx, Keycode::from_primitive(key_code), &event)
    }))
}

extern "system" fn on_key_up<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    key_code: jint,
    event: KeyEvent<'local>,
) -> jboolean {
    as_jboolean(with_peer(env, view, peer, |ctx, peer| {
        peer.on_key_up(ctx, Keycode::from_primitive(key_code), &event)
    }))
}

extern "system" fn on_trackball_event<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    as_jboolean(with_peer(env, view, peer, |ctx, peer| {
        peer.on_trackball_event(ctx, &event)
    }))
}

extern "system" fn on_touch_event<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    as_jboolean(with_peer(env, view, peer, |ctx, peer| {
        peer.on_touch_event(ctx, &event)
    }))
}

extern "system" fn on_generic_motion_event<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    as_jboolean(with_peer(env, view, peer, |ctx, peer| {
        peer.on_generic_motion_event(ctx, &event)
    }))
}

extern "system" fn on_hover_event<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    as_jboolean(with_peer(env, view, peer, |ctx, peer| {
        peer.on_hover_event(ctx, &event)
    }))
}

extern "system" fn on_focus_changed<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    gain_focus: jboolean,
    direction: jint,
    previously_focused_rect: Rect<'local>,
) {
    with_peer(env, view, peer, |ctx, peer| {
        peer.on_focus_changed(
            ctx,
            gain_focus == JNI_TRUE,
            direction,
            (!previously_focused_rect.0.as_raw().is_null()).then_some(&previously_focused_rect),
        );
    })
}

extern "system" fn on_window_focus_changed<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    has_window_focus: jboolean,
) {
    with_peer(env, view, peer, |ctx, peer| {
        peer.on_window_focus_changed(ctx, has_window_focus == JNI_TRUE);
    })
}

extern "system" fn on_attached_to_window<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
) {
    with_peer(env, view, peer, |ctx, peer| {
        peer.on_attached_to_window(ctx);
    })
}

extern "system" fn on_detached_from_window<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
) {
    let mut map = PEER_MAP.lock().unwrap();
    let peer = map.remove(&peer).unwrap();
    drop(map);
    let mut peer = peer.borrow_mut();
    let mut ctx = CallbackCtx::new(env, view);
    peer.on_detached_from_window(&mut ctx);
    drop(peer);
    ctx.view.remove_frame_callback(&mut ctx.env);
    ctx.view.remove_delayed_callbacks(&mut ctx.env);
    ctx.finish();
}

extern "system" fn on_window_visibility_changed<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    visibility: jint,
) {
    with_peer(env, view, peer, |ctx, peer| {
        peer.on_window_visibility_changed(ctx, visibility);
    })
}

extern "system" fn surface_created<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    holder: SurfaceHolder<'local>,
) {
    with_peer(env, view, peer, |ctx, peer| {
        peer.surface_created(ctx, &holder);
    })
}

extern "system" fn surface_changed<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    holder: SurfaceHolder<'local>,
    format: jint,
    width: jint,
    height: jint,
) {
    with_peer(env, view, peer, |ctx, peer| {
        peer.surface_changed(ctx, &holder, format, width, height);
    })
}

extern "system" fn surface_destroyed<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    holder: SurfaceHolder<'local>,
) {
    with_peer(env, view, peer, |ctx, peer| {
        peer.surface_destroyed(ctx, &holder);
    })
}

extern "system" fn do_frame<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    frame_time_nanos: jlong,
) {
    with_peer(env, view, peer, |ctx, peer| {
        peer.do_frame(ctx, frame_time_nanos);
    })
}

extern "system" fn delayed_callback<'local>(env: JNIEnv<'local>, view: View<'local>, peer: jlong) {
    with_peer(env, view, peer, |ctx, peer| {
        peer.delayed_callback(ctx);
    })
}

pub fn register_view_peer(peer: impl 'static + ViewPeer) -> jlong {
    let id = NEXT_PEER_ID.fetch_add(1, Ordering::Relaxed);
    let mut map = PEER_MAP.lock().unwrap();
    map.insert(id, SendWrapper::new(Rc::new(RefCell::new(Box::new(peer)))));
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
                    name: "onHoverEventNative".into(),
                    sig: "(JLandroid/view/MotionEvent;)Z".into(),
                    fn_ptr: on_hover_event as *mut c_void,
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
                    name: "hasAccessibilityNodeProviderNative".into(),
                    sig: "(J)Z".into(),
                    fn_ptr: has_accessibility_node_provider as *mut c_void,
                },
                NativeMethod {
                    name: "createAccessibilityNodeInfoNative".into(),
                    sig: "(JI)Landroid/view/accessibility/AccessibilityNodeInfo;".into(),
                    fn_ptr: create_accessibility_node_info as *mut c_void,
                },
                NativeMethod {
                    name: "accessibilityFindFocusNative".into(),
                    sig: "(JI)Landroid/view/accessibility/AccessibilityNodeInfo;".into(),
                    fn_ptr: accessibility_find_focus as *mut c_void,
                },
                NativeMethod {
                    name: "performAccessibilityActionNative".into(),
                    sig: "(JIILandroid/os/Bundle;)Z".into(),
                    fn_ptr: perform_accessibility_action as *mut c_void,
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
