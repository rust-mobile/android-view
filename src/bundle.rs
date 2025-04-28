use jni::objects::JObject;

#[repr(transparent)]
pub struct Bundle<'local>(pub JObject<'local>);
