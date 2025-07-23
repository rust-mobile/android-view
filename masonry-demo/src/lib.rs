// Copyright 2024 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

#![deny(unsafe_op_in_unsafe_fn)]

use android_view::{
    jni::{
        JNIEnv, JavaVM,
        sys::{JNI_VERSION_1_6, JavaVM as RawJavaVM, jint, jlong},
    },
    *,
};
use masonry::{
    core::{ErasedAction, NewWidget, Properties, Widget, WidgetId},
    properties::Padding,
    theme::default_property_set,
    widgets::{Button, ButtonPress, Flex, Label, Portal, TextAction, TextArea, TextInput},
};
use masonry_android::{AppDriver, DriverCtx};
use std::{ffi::c_void, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const WIDGET_SPACING: f64 = 5.0;

struct Driver {
    next_task: String,
}

impl AppDriver for Driver {
    fn on_action(&mut self, ctx: &mut DriverCtx<'_>, _widget_id: WidgetId, action: ErasedAction) {
        if action.is::<ButtonPress>() {
            ctx.render_root().edit_root_widget(|mut root| {
                let mut portal = root.downcast::<Portal<Flex>>();
                let mut flex = Portal::child_mut(&mut portal);
                Flex::add_child(&mut flex, Label::new(self.next_task.clone()).with_auto_id());

                let mut first_row = Flex::child_mut(&mut flex, 0).unwrap();
                let mut first_row = first_row.downcast::<Flex>();
                let mut text_input = Flex::child_mut(&mut first_row, 0).unwrap();
                let mut text_input = text_input.downcast::<TextInput>();
                let mut text_area = TextInput::text_mut(&mut text_input);
                TextArea::reset_text(&mut text_area, "");
            });
        } else if action.is::<TextAction>() {
            let action = action.downcast::<TextAction>().unwrap();
            match *action {
                TextAction::Changed(new_text) => {
                    self.next_task = new_text.clone();
                }
                TextAction::Entered(_) => {}
            }
        }
    }
}

fn make_widget_tree() -> impl Widget {
    Portal::new(
        Flex::column()
            .with_child(NewWidget::new_with_props(
                Flex::row()
                    .with_flex_child(TextInput::new("").with_auto_id(), 1.0)
                    .with_child(Button::new("Add task").with_auto_id()),
                Properties::new().with(Padding::all(WIDGET_SPACING)),
            ))
            .with_spacer(WIDGET_SPACING)
            .with_auto_id(),
    )
}

extern "system" fn new_view_peer<'local>(
    mut env: JNIEnv<'local>,
    _view: View<'local>,
    context: Context<'local>,
) -> jlong {
    masonry_android::new_view_peer(
        &mut env,
        &context,
        NewWidget::new(make_widget_tree()).erased(),
        Driver {
            next_task: String::new(),
        },
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
