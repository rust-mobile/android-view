#![deny(unsafe_op_in_unsafe_fn)]

use android_view::{
    jni::{
        JNIEnv, JavaVM,
        sys::{JNI_VERSION_1_6, JavaVM as RawJavaVM, jint, jlong},
    },
    *,
};
use std::ffi::c_void;

struct DemoViewCallback;

impl ViewCallback for DemoViewCallback {
    // TODO
}

extern "system" fn view_new_native<'local>(
    _env: JNIEnv<'local>,
    _view: View<'local>,
    _context: Context<'local>,
) -> jlong {
    new_view_handle(DemoViewCallback)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn JNI_OnLoad(vm: *mut RawJavaVM, _: *mut c_void) -> jint {
    let vm = unsafe { JavaVM::from_raw(vm) }.unwrap();
    let mut env = vm.get_env().unwrap();
    register_view_class(
        &mut env,
        "org/linebender/android/viewdemo/DemoView",
        view_new_native,
    );
    JNI_VERSION_1_6
}
