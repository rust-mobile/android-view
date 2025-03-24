use jni::objects::JObject;

#[repr(transparent)]
pub struct IBinder<'local>(pub JObject<'local>);
