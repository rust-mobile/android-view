use jni::{
    descriptors::Desc,
    objects::{JClass, JIntArray, JObject},
    sys::{jboolean, jint, jlong, JNI_FALSE, JNI_TRUE},
    JNIEnv, NativeMethod,
};
use std::{
    collections::BTreeMap,
    ffi::c_void,
    sync::{
        atomic::{AtomicI64, Ordering},
        Mutex, Once,
    },
};

use crate::{context::*, events::*, graphics::*, surface::*};

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

static NEXT_HANDLE: AtomicI64 = AtomicI64::new(0);
static HANDLE_MAP: Mutex<BTreeMap<jlong, Box<dyn ViewCallback>>> = Mutex::new(BTreeMap::new());

fn with_callback<F, T>(handle: jlong, f: F) -> T
where
    F: FnOnce(&mut dyn ViewCallback) -> T,
{
    let mut map = HANDLE_MAP.lock().unwrap();
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
    with_callback(handle, |callback| {
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
    with_callback(handle, |callback| {
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
    with_callback(handle, |callback| {
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
    with_callback(handle, |callback| {
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
    with_callback(handle, |callback| {
        to_jboolean(callback.on_key_up(&mut env, &view, key_code, &event))
    })
}

extern "system" fn on_trackball_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    with_callback(handle, |callback| {
        to_jboolean(callback.on_trackball_event(&mut env, &view, &event))
    })
}

extern "system" fn on_touch_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    with_callback(handle, |callback| {
        to_jboolean(callback.on_touch_event(&mut env, &view, &event))
    })
}

extern "system" fn on_generic_motion_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    with_callback(handle, |callback| {
        to_jboolean(callback.on_generic_motion_event(&mut env, &view, &event))
    })
}

extern "system" fn on_hover_event<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    event: MotionEvent<'local>,
) -> jboolean {
    with_callback(handle, |callback| {
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
    with_callback(handle, |callback| {
        callback.on_focus_changed(
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
    handle: jlong,
    has_window_focus: jboolean,
) {
    with_callback(handle, |callback| {
        callback.on_window_focus_changed(&mut env, &view, has_window_focus == JNI_TRUE);
    })
}

extern "system" fn on_attached_to_window<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
) {
    with_callback(handle, |callback| {
        callback.on_attached_to_window(&mut env, &view);
    })
}

extern "system" fn on_detached_from_window<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
) {
    let mut map = HANDLE_MAP.lock().unwrap();
    let mut callback = map.remove(&handle).unwrap();
    callback.on_detached_from_window(&mut env, &view);
}

extern "system" fn on_window_visibility_changed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    visibility: jint,
) {
    with_callback(handle, |callback| {
        callback.on_window_visibility_changed(&mut env, &view, visibility);
    })
}

extern "system" fn surface_created<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    holder: SurfaceHolder<'local>,
) {
    with_callback(handle, |callback| {
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
    with_callback(handle, |callback| {
        callback.surface_changed(&mut env, &view, &holder, format, width, height);
    })
}

extern "system" fn surface_destroyed<'local>(
    mut env: JNIEnv<'local>,
    view: View<'local>,
    handle: jlong,
    holder: SurfaceHolder<'local>,
) {
    with_callback(handle, |callback| {
        callback.surface_destroyed(&mut env, &view, &holder);
    })
}

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
    let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    let mut map = HANDLE_MAP.lock().unwrap();
    map.insert(handle, Box::new(callback));
    handle
}

pub fn register_view_class<'local, 'other_local>(
    env: &mut JNIEnv<'local>,
    class: impl Desc<'local, JClass<'other_local>>,
    new_native: for<'a> extern "system" fn(JNIEnv<'a>, View<'a>, Context<'a>) -> jlong,
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
            ],
        )
        .unwrap();
    });
    env.register_native_methods(
        class,
        &[NativeMethod {
            name: "newNative".into(),
            sig: "(Landroid/content/Context;)J".into(),
            fn_ptr: new_native as *mut c_void,
        }],
    )
    .unwrap();
}
