use jni::objects::JObject;

#[repr(transparent)]
pub struct Context<'local>(pub JObject<'local>);

// TODO: What methods do we need to expose from this class?
