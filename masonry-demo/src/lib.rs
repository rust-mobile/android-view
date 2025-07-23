// Copyright 2024 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

//! A demonstration of the [`VirtualScroll`] widget, producing an infinite[^1] FizzBuzz.
//!
//! [^1]: Limited to `i64::MIN..i64::MAX-1`; that is, there are `2^64-1` possible items.
//! However, there is (currently...) no way to jump to a specific item, so it's impossible to reach the end.

#![deny(unsafe_op_in_unsafe_fn)]

use android_view::{
    jni::{
        JNIEnv, JavaVM,
        sys::{JNI_VERSION_1_6, JavaVM as RawJavaVM, jint, jlong},
    },
    *,
};
use masonry::{
    core::{ArcStr, ErasedAction, NewWidget, StyleProperty, WidgetId},
    theme::default_property_set,
    widgets::{Label, VirtualScroll, VirtualScrollAction},
};
use masonry_android::{AppDriver, DriverCtx};
use std::{ffi::c_void, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// The widget kind contained in the scroll area. This is a type parameter (`W`) of [`VirtualScroll`],
/// although note that [`dyn Widget`](masonry::core::Widget) can also be used for dynamic children kinds.
///
/// We use a type alias for this, as when we downcast to the `VirtualScroll`, we need to be sure to
/// always use the same type for `W`.
type ScrollContents = Label;

/// Function to create the virtual scroll area.
fn init() -> VirtualScroll<ScrollContents> {
    // We start our fizzbuzzing with the top of the screen at item 0
    VirtualScroll::new(0)
}

struct Driver {
    scroll_id: WidgetId,
    fizz: ArcStr,
    buzz: ArcStr,
    fizzbuzz: ArcStr,
}

impl AppDriver for Driver {
    fn on_action(&mut self, ctx: &mut DriverCtx<'_>, widget_id: WidgetId, action: ErasedAction) {
        if widget_id == self.scroll_id {
            // The VirtualScroll widget will send us a VirtualScrollAction every time it wants different
            // items to be loaded or unloaded.
            let action = action
                .downcast::<VirtualScrollAction>()
                .expect("Only expected Virtual Scroll actions");
            ctx.render_root().edit_root_widget(|mut root| {
                let mut scroll = root.downcast::<VirtualScroll<ScrollContents>>();
                // We need to tell the `VirtualScroll` which request this is associated with
                // This is so that the controller knows which actions have been handled.
                VirtualScroll::will_handle_action(&mut scroll, &action);
                for idx in action.old_active.clone() {
                    if !action.target.contains(&idx) {
                        // If we had different work to do in response to the item being unloaded
                        // (for example, saving some related data?), then we'd do it here
                        VirtualScroll::remove_child(&mut scroll, idx);
                    }
                }
                for idx in action.target.clone() {
                    if !action.old_active.contains(&idx) {
                        let label: ArcStr = match (idx % 3 == 0, idx % 5 == 0) {
                            (false, true) => self.buzz.clone(),
                            (true, false) => self.fizz.clone(),
                            (true, true) => self.fizzbuzz.clone(),
                            (false, false) => format!("{idx}").into(),
                        };
                        VirtualScroll::add_child(
                            &mut scroll,
                            idx,
                            NewWidget::new(Label::new(label).with_style(StyleProperty::FontSize(
                                if idx % 100 == 0 { 40. } else { 20. },
                            ))),
                        );
                    }
                }
            });
        } else {
            tracing::warn!("Got unexpected action {action:?}");
        }
    }
}

extern "system" fn new_view_peer<'local>(
    mut env: JNIEnv<'local>,
    _view: View<'local>,
    context: Context<'local>,
) -> jlong {
    let scroll_id = WidgetId::next();
    let main_widget = NewWidget::new_with_id(init(), scroll_id).erased();
    let driver = Driver {
        scroll_id,
        fizz: "Fizz".into(),
        buzz: "Buzz".into(),
        fizzbuzz: "FizzBuzz".into(),
    };
    masonry_android::new_view_peer(
        &mut env,
        &context,
        main_widget,
        driver,
        Arc::new(default_property_set()),
    )
}

/// Symbol run at JNI load time.
///
/// # Safety
/// There is no alternative, interacting with JNI is always unsafe at some level.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn JNI_OnLoad(vm: *mut RawJavaVM, _: *mut c_void) -> jint {
    // This will try to create a "log" logger, and error because one was already created above
    // We therefore ignore the error
    // Ideally, we'd only ignore the SetLoggerError, but the only way that's possible is to inspect
    // `Debug/Display` on the TryInitError, which is awful.
    let _ = tracing_subscriber::registry()
        .with(tracing_android_trace::AndroidTraceLayer::new())
        .try_init();

    let vm = unsafe { JavaVM::from_raw(vm) }.unwrap();
    let mut env = vm.get_env().unwrap();
    register_view_class(
        &mut env,
        "org/linebender/android/masonrydemo/DemoView",
        new_view_peer,
    );
    JNI_VERSION_1_6
}
