use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean};

pub(crate) fn as_jboolean(flag: bool) -> jboolean {
    if flag { JNI_TRUE } else { JNI_FALSE }
}
