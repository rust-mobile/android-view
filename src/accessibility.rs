use jni::{
    JNIEnv,
    objects::JObject,
    sys::{jboolean, jint, jlong},
};

use crate::{bundle::*, callback_ctx::*, util::*, view::*};

#[derive(Default)]
#[repr(transparent)]
pub struct AccessibilityNodeInfo<'local>(pub JObject<'local>);

#[allow(unused_variables)]
pub trait AccessibilityNodeProvider {
    fn create_accessibility_node_info<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        virtual_view_id: jint,
    ) -> AccessibilityNodeInfo<'local>;

    fn find_focus<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        focus_type: jint,
    ) -> AccessibilityNodeInfo<'local>;

    fn perform_action<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        virtual_view_id: jint,
        action: jint,
        arguments: &Bundle<'local>,
    ) -> bool;
}

fn with_accessibility_node_provider<'local, F, T: Default>(
    env: JNIEnv<'local>,
    view: View<'local>,
    id: jlong,
    f: F,
) -> T
where
    F: FnOnce(&mut CallbackCtx<'local>, &mut dyn AccessibilityNodeProvider) -> T,
{
    with_peer(env, view, id, |ctx, peer| {
        let Some(anp) = peer.as_accessibility_node_provider() else {
            return T::default();
        };
        f(ctx, anp)
    })
}

pub(crate) extern "system" fn has_accessibility_node_provider<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
) -> jboolean {
    as_jboolean(with_accessibility_node_provider(
        env,
        view,
        peer,
        |_ctx, _anp| true,
    ))
}

pub(crate) extern "system" fn create_accessibility_node_info<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    virtual_view_id: jint,
) -> AccessibilityNodeInfo<'local> {
    with_accessibility_node_provider(env, view, peer, |ctx, anp| {
        anp.create_accessibility_node_info(ctx, virtual_view_id)
    })
}

pub(crate) extern "system" fn accessibility_find_focus<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    focus_type: jint,
) -> AccessibilityNodeInfo<'local> {
    with_accessibility_node_provider(env, view, peer, |ctx, anp| anp.find_focus(ctx, focus_type))
}

pub(crate) extern "system" fn perform_accessibility_action<'local>(
    env: JNIEnv<'local>,
    view: View<'local>,
    peer: jlong,
    virtual_view_id: jint,
    action: jint,
    arguments: Bundle<'local>,
) -> jboolean {
    as_jboolean(with_accessibility_node_provider(
        env,
        view,
        peer,
        |ctx, anp| anp.perform_action(ctx, virtual_view_id, action, &arguments),
    ))
}
