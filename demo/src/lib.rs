// Derived from vello_editor
// Copyright 2024 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![deny(unsafe_op_in_unsafe_fn)]

use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, Node, Role, Tree, TreeUpdate,
};
use android_view::{
    jni::{
        JNIEnv, JavaVM,
        sys::{JNI_VERSION_1_6, JavaVM as RawJavaVM, jint, jlong},
    },
    ndk::{event::Keycode, native_window::NativeWindow},
    *,
};
use anyhow::Result;
use log::LevelFilter;
use std::borrow::Cow;
use std::ffi::c_void;
use std::num::NonZeroUsize;
use std::time::Instant;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ui_events::pointer::PointerEvent;
use vello::peniko::Color;
use vello::util::{RenderContext, RenderSurface};
use vello::wgpu::{
    self,
    rwh::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle},
};
use vello::{AaConfig, Renderer, RendererOptions, Scene};

mod access_ids;
use access_ids::{TEXT_INPUT_ID, WINDOW_ID};

mod text;

// From VelloCompose
struct AndroidWindowHandle {
    window: NativeWindow,
}

impl HasDisplayHandle for AndroidWindowHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(DisplayHandle::android())
    }
}

impl HasWindowHandle for AndroidWindowHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.window.window_handle()
    }
}

/// Helper function that creates a vello `Renderer` for a given `RenderContext` and `RenderSurface`
fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface<'_>) -> Renderer {
    Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions {
            use_cpu: false,
            antialiasing_support: vello::AaSupport::area_only(),
            num_init_threads: None,
            // TODO: add pipeline cache.
            pipeline_cache: None,
        },
    )
    .expect("Couldn't create renderer")
}

struct EditorAccessTreeSource<'a> {
    editor: &'a mut text::Editor,
    render_surface: &'a Option<RenderSurface<'static>>,
}

impl EditorAccessTreeSource<'_> {
    fn build_text_input_node(&mut self, update: &mut TreeUpdate) -> Node {
        let mut node = Node::new(Role::MultilineTextInput);
        node.add_action(Action::Click);
        if let Some(surface) = &self.render_surface {
            node.set_bounds(accesskit::Rect {
                x0: 0.0,
                y0: 0.0,
                x1: surface.config.width as _,
                y1: surface.config.height as _,
            });
        }
        self.editor.accessibility(update, &mut node);
        node
    }

    fn build_initial_tree(&mut self) -> TreeUpdate {
        let mut update = TreeUpdate {
            nodes: vec![],
            tree: Some(Tree::new(WINDOW_ID)),
            focus: TEXT_INPUT_ID,
        };
        let mut node = Node::new(Role::Window);
        node.push_child(TEXT_INPUT_ID);
        update.nodes.push((WINDOW_ID, node));
        let node = self.build_text_input_node(&mut update);
        update.nodes.push((TEXT_INPUT_ID, node));
        update
    }
}

impl ActivationHandler for EditorAccessTreeSource<'_> {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(self.build_initial_tree())
    }
}

fn show_soft_input<'local>(env: &mut JNIEnv<'local>, view: &View<'local>) {
    let imm = view.input_method_manager(env);
    imm.show_soft_input(env, view, 0);
}

struct EditorAccessActionHandler<'a, 'local> {
    ctx: &'a mut CallbackCtx<'local>,
    editor: &'a mut text::Editor,
}

impl ActionHandler for EditorAccessActionHandler<'_, '_> {
    fn do_action<'local>(&mut self, req: ActionRequest) {
        if req.target == TEXT_INPUT_ID {
            if req.action == Action::Click {
                self.ctx.push_static_deferred_callback(show_soft_input);
                return;
            }
            self.editor.handle_accesskit_action_request(&req);
        }
    }
}

struct DemoViewPeer {
    /// The vello `RenderContext` which is a global context that lasts for the
    /// lifetime of the application.
    context: RenderContext,

    /// An array of renderers, one per wgpu device.
    renderers: Vec<Option<Renderer>>,

    /// State for our example where we store the winit Window and the wgpu Surface.
    render_surface: Option<RenderSurface<'static>>,

    /// A `vello::Scene` where the editor layout will be drawn.
    scene: Scene,

    /// Our `Editor`, which owns a `parley::PlainEditor`.
    editor: text::Editor,

    /// The last generation of the editor layout that we drew.
    last_drawn_generation: text::Generation,

    ime_active: bool,
    batch_edit_depth: usize,

    access_adapter: accesskit_android::Adapter,
    /// Pointer adapter state.
    tap_counter: TapCounter,
}

impl DemoViewPeer {
    fn enqueue_render_if_needed(&mut self, ctx: &mut CallbackCtx) {
        if self.render_surface.is_none()
            || self.last_drawn_generation == self.editor.generation()
            || self.batch_edit_depth != 0
        {
            return;
        }
        ctx.view.post_frame_callback(&mut ctx.env);
    }

    fn schedule_next_blink(&self, ctx: &mut CallbackCtx) {
        if let Some(next_time) = self.editor.next_blink_time() {
            let delay = next_time.duration_since(Instant::now());
            ctx.view.post_delayed(&mut ctx.env, delay.as_millis() as _);
        }
    }

    fn update_cursor_state(&mut self, ctx: &mut CallbackCtx, focused: bool) {
        self.last_drawn_generation = Default::default();
        ctx.view.remove_delayed_callbacks(&mut ctx.env);
        if focused {
            self.editor.cursor_reset();
            self.schedule_next_blink(ctx);
        } else {
            self.editor.disable_blink();
            self.editor.cursor_blink();
        }
    }

    fn render(&mut self, ctx: &mut CallbackCtx) {
        // Get the RenderSurface (surface + config).
        let surface = self.render_surface.as_ref().unwrap();

        // Get the window size.
        let width = surface.config.width;
        let height = surface.config.height;

        // Get a handle to the device.
        let device_handle = &self.context.devices[surface.dev_id];

        // Sometimes `Scene` is stale and needs to be redrawn.
        if self.last_drawn_generation != self.editor.generation() && self.batch_edit_depth == 0 {
            // Empty the scene of objects to draw. You could create a new Scene each time, but in this case
            // the same Scene is reused so that the underlying memory allocation can also be reused.
            self.scene.reset();

            self.last_drawn_generation = self.editor.draw(&mut self.scene);

            let mut tree_source = EditorAccessTreeSource {
                render_surface: &self.render_surface,
                editor: &mut self.editor,
            };
            if let Some(events) = self.access_adapter.update_if_active(|| {
                let mut update = TreeUpdate {
                    nodes: vec![],
                    tree: None,
                    focus: TEXT_INPUT_ID,
                };
                let node = tree_source.build_text_input_node(&mut update);
                update.nodes.push((TEXT_INPUT_ID, node));
                update
            }) {
                ctx.push_dynamic_deferred_callback(move |env, view| {
                    events.raise(env, &view.0);
                });
            }

            if self.ime_active {
                let selection = self.editor.editor().raw_selection().text_range();
                let sel_start = self.editor.utf8_to_utf16_index(selection.start) as jint;
                let sel_end = self.editor.utf8_to_utf16_index(selection.end) as jint;
                let (comp_start, comp_end) = if let Some(range) = self.editor.editor().raw_compose()
                {
                    let start = self.editor.utf8_to_utf16_index(range.start) as jint;
                    let end = self.editor.utf8_to_utf16_index(range.end) as jint;
                    (start, end)
                } else {
                    (-1, -1)
                };
                ctx.push_dynamic_deferred_callback(move |env, view| {
                    let imm = view.input_method_manager(env);
                    imm.update_selection(env, view, sel_start, sel_end, comp_start, comp_end);
                });
            }
        }

        // Render to the surface's texture.
        self.renderers[surface.dev_id]
            .as_mut()
            .unwrap()
            .render_to_texture(
                &device_handle.device,
                &device_handle.queue,
                &self.scene,
                &surface.target_view,
                &vello::RenderParams {
                    base_color: Color::from_rgb8(30, 30, 30), // Background color
                    width,
                    height,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .expect("failed to render to surface");

        // Get the surface's texture.
        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");

        // Perform the copy.
        let mut encoder =
            device_handle
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Surface Blit"),
                });
        surface.blitter.copy(
            &device_handle.device,
            &mut encoder,
            &surface.target_view,
            &surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );
        device_handle.queue.submit([encoder.finish()]);
        // Queue the texture to be presented on the surface.
        surface_texture.present();

        device_handle.device.poll(wgpu::Maintain::Poll);
    }

    fn set_composing_text_internal(&mut self, text: &str, new_cursor_position: jint) {
        let mut drv = self.editor.driver();
        if text.is_empty() {
            if drv.editor.is_composing() {
                drv.clear_compose();
            } else {
                drv.delete_selection();
            }
        } else {
            // We always pass a cursor offset of 0 to `PlainEditor::set_compose`
            // and then set the cursor using our own logic.
            drv.set_compose(text, Some((0, 0)));
        }
        let range = drv
            .editor
            .raw_compose()
            .clone()
            .unwrap_or_else(|| drv.editor.raw_selection().text_range());
        let start_utf16 = self.editor.utf8_to_utf16_index(range.start);
        let end_utf16 = self.editor.utf8_to_utf16_index(range.end);
        let cursor_pos_utf16 = if new_cursor_position > 0 {
            let len_utf16 = self
                .editor
                .utf8_to_utf16_index(self.editor.editor().raw_text().len());
            end_utf16
                .saturating_add((new_cursor_position - 1) as usize)
                .min(len_utf16)
        } else {
            start_utf16.saturating_sub(-new_cursor_position as usize)
        };
        let cursor_pos = self.editor.utf16_to_utf8_index(cursor_pos_utf16);
        let mut drv = self.editor.driver();
        drv.move_to_byte(cursor_pos);
    }
}

impl ViewPeer for DemoViewPeer {
    fn on_key_down<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        _: Keycode,
        event: &KeyEvent<'local>,
    ) -> bool {
        if !self
            .editor
            .on_keyboard_event(event.to_keyboard_event(&mut ctx.env))
        {
            return false;
        }
        self.enqueue_render_if_needed(ctx);
        true
    }

    fn on_key_up<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        _: Keycode,
        event: &KeyEvent<'local>,
    ) -> bool {
        if !self
            .editor
            .on_keyboard_event(event.to_keyboard_event(&mut ctx.env))
        {
            return false;
        }
        self.enqueue_render_if_needed(ctx);
        true
    }

    fn on_touch_event<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        event: &MotionEvent<'local>,
    ) -> bool {
        let Some(ev) = event.to_pointer_event(&mut ctx.env, &self.tap_counter.vc) else {
            return false;
        };

        if matches!(ev, PointerEvent::Up { .. }) {
            ctx.push_static_deferred_callback(show_soft_input);
        }

        if self
            .editor
            .handle_pointer_event(self.tap_counter.attach_count(ev))
        {
            self.enqueue_render_if_needed(ctx);
        }

        true
    }

    fn on_generic_motion_event<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        event: &MotionEvent<'local>,
    ) -> bool {
        self.on_touch_event(ctx, event)
    }

    fn on_hover_event<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        event: &MotionEvent<'local>,
    ) -> bool {
        let mut tree_source = EditorAccessTreeSource {
            render_surface: &self.render_surface,
            editor: &mut self.editor,
        };
        let action = event.action(&mut ctx.env);
        let x = event.x(&mut ctx.env);
        let y = event.y(&mut ctx.env);
        if let Some(events) = self
            .access_adapter
            .on_hover_event(&mut tree_source, action, x, y)
        {
            ctx.push_dynamic_deferred_callback(move |env, view| {
                events.raise(env, &view.0);
            });
            true
        } else {
            self.on_touch_event(ctx, event)
        }
    }

    fn on_focus_changed<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        gain_focus: bool,
        _direction: jint,
        _previously_focused_rect: Option<&Rect<'local>>,
    ) {
        self.update_cursor_state(ctx, gain_focus);
        self.enqueue_render_if_needed(ctx);
    }

    fn surface_changed<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        holder: &SurfaceHolder<'local>,
        _format: jint,
        width: jint,
        height: jint,
    ) {
        self.tap_counter = TapCounter::new(ctx.view.view_configuration(&mut ctx.env));
        let editor = self.editor.editor_mut();
        editor.set_scale(1.0);
        editor.set_width(Some(width as f32 - 2_f32 * text::INSET));
        self.last_drawn_generation = Default::default();
        let focused = ctx.view.is_focused(&mut ctx.env);
        self.update_cursor_state(ctx, focused);

        let window = holder.surface(&mut ctx.env).to_native_window(&mut ctx.env);
        // Drop the old surface, if any, that owned the native window
        // before creating a new one. Otherwise, we crash with
        // ERROR_NATIVE_WINDOW_IN_USE_KHR.
        self.render_surface = None;
        let surface = self
            .context
            .instance
            .create_surface(wgpu::SurfaceTarget::from(AndroidWindowHandle { window }))
            .expect("Error creating surface");
        let dev_id =
            pollster::block_on(self.context.device(Some(&surface))).expect("No compatible device");
        let device_handle = &self.context.devices[dev_id];
        let capabilities = surface.get_capabilities(device_handle.adapter());
        let present_mode = if capabilities
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::AutoVsync
        };

        let surface_future =
            self.context
                .create_render_surface(surface, width as _, height as _, present_mode);
        let surface = pollster::block_on(surface_future).expect("Error creating surface");

        // Create a vello Renderer for the surface (using its device id)
        self.renderers
            .resize_with(self.context.devices.len(), || None);
        self.renderers[surface.dev_id]
            .get_or_insert_with(|| create_vello_renderer(&self.context, &surface));
        self.render_surface = Some(surface);

        self.render(ctx);
    }

    fn surface_destroyed<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        _holder: &SurfaceHolder<'local>,
    ) {
        self.render_surface = None;
        ctx.view.remove_frame_callback(&mut ctx.env);
        ctx.view.remove_delayed_callbacks(&mut ctx.env);
    }

    fn do_frame(&mut self, ctx: &mut CallbackCtx, _frame_time_nanos: jlong) {
        self.render(ctx);
    }

    fn delayed_callback(&mut self, ctx: &mut CallbackCtx) {
        self.editor.cursor_blink();
        self.last_drawn_generation = Default::default();
        self.enqueue_render_if_needed(ctx);
        self.schedule_next_blink(ctx);
    }

    fn as_accessibility_node_provider(&mut self) -> Option<&mut dyn AccessibilityNodeProvider> {
        Some(self)
    }

    fn as_input_connection(&mut self) -> Option<&mut dyn InputConnection> {
        Some(self)
    }
}

impl AccessibilityNodeProvider for DemoViewPeer {
    fn create_accessibility_node_info<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        virtual_view_id: jint,
    ) -> AccessibilityNodeInfo<'local> {
        let mut tree_source = EditorAccessTreeSource {
            render_surface: &self.render_surface,
            editor: &mut self.editor,
        };
        AccessibilityNodeInfo(self.access_adapter.create_accessibility_node_info(
            &mut tree_source,
            &mut ctx.env,
            &ctx.view.0,
            virtual_view_id,
        ))
    }

    fn find_focus<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        focus_type: jint,
    ) -> AccessibilityNodeInfo<'local> {
        let mut tree_source = EditorAccessTreeSource {
            render_surface: &self.render_surface,
            editor: &mut self.editor,
        };
        AccessibilityNodeInfo(self.access_adapter.find_focus(
            &mut tree_source,
            &mut ctx.env,
            &ctx.view.0,
            focus_type,
        ))
    }

    fn perform_action<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        virtual_view_id: jint,
        action: jint,
        arguments: &Bundle<'local>,
    ) -> bool {
        let Some(action) =
            accesskit_android::PlatformAction::from_java(&mut ctx.env, action, &arguments.0)
        else {
            return false;
        };
        let mut action_handler = EditorAccessActionHandler {
            ctx,
            editor: &mut self.editor,
        };
        if let Some(events) =
            self.access_adapter
                .perform_action(&mut action_handler, virtual_view_id, &action)
        {
            ctx.push_dynamic_deferred_callback(move |env, view| {
                events.raise(env, &view.0);
            });
            self.enqueue_render_if_needed(ctx);
            true
        } else {
            false
        }
    }
}

impl InputConnection for DemoViewPeer {
    fn on_create_input_connection<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        out_attrs: &EditorInfo<'local>,
    ) {
        out_attrs.set_input_type(
            &mut ctx.env,
            INPUT_TYPE_CLASS_TEXT
                | INPUT_TYPE_TEXT_FLAG_CAP_SENTENCES
                | INPUT_TYPE_TEXT_FLAG_AUTO_CORRECT
                | INPUT_TYPE_TEXT_FLAG_MULTI_LINE,
        );
        out_attrs.set_ime_options(
            &mut ctx.env,
            IME_FLAG_NO_FULLSCREEN | IME_FLAG_NO_EXTRACT_UI | IME_FLAG_NO_ENTER_ACTION,
        );
        let selection = self.editor.editor().raw_selection().text_range();
        let sel_start = self.editor.utf8_to_utf16_index(selection.start);
        let sel_end = self.editor.utf8_to_utf16_index(selection.end);
        out_attrs.set_initial_sel_start(&mut ctx.env, sel_start as jint);
        out_attrs.set_initial_sel_end(&mut ctx.env, sel_end as jint);
        let text = self.editor.editor().raw_text();
        let initial_caps_mode = caps_mode(&mut ctx.env, text, sel_start, CAP_MODE_SENTENCES);
        out_attrs.set_initial_caps_mode(&mut ctx.env, initial_caps_mode);
        self.editor.driver().clear_compose();
        self.enqueue_render_if_needed(ctx);
        self.ime_active = true;
    }

    fn text_before_cursor<'slf>(
        &'slf mut self,
        _ctx: &mut CallbackCtx,
        n: jint,
    ) -> Option<Cow<'slf, str>> {
        if n < 0 {
            return None;
        }
        let n = n as usize;
        let editor = self.editor.editor();
        let text = editor.raw_text();
        let selection = editor.raw_selection().text_range();
        let range_end = selection.start;
        let range_end_utf16 = self.editor.utf8_to_utf16_index(range_end);
        let range_start = if range_end_utf16 <= n {
            0
        } else {
            self.editor.utf16_to_utf8_index(range_end_utf16 - n)
        };
        Some(Cow::Borrowed(&text[range_start..range_end]))
    }

    fn text_after_cursor<'slf>(
        &'slf mut self,
        _ctx: &mut CallbackCtx,
        n: jint,
    ) -> Option<Cow<'slf, str>> {
        if n < 0 {
            return None;
        }
        let n = n as usize;
        let editor = self.editor.editor();
        let text = editor.raw_text();
        let selection = editor.raw_selection().text_range();
        let range_start = selection.end;
        let range_start_utf16 = self.editor.utf8_to_utf16_index(range_start);
        let len_utf16 = self.editor.utf8_to_utf16_index(text.len());
        let range_end = if range_start_utf16 + n >= len_utf16 {
            text.len()
        } else {
            self.editor.utf16_to_utf8_index(range_start_utf16 + n)
        };
        Some(Cow::Borrowed(&text[range_start..range_end]))
    }

    fn selected_text<'slf>(&'slf mut self, _ctx: &mut CallbackCtx) -> Option<Cow<'slf, str>> {
        self.editor.editor().selected_text().map(Cow::Borrowed)
    }

    fn cursor_caps_mode(&mut self, ctx: &mut CallbackCtx, req_modes: u32) -> u32 {
        let editor = self.editor.editor();
        let text = editor.raw_text();
        let offset = editor.raw_selection().focus().index();
        let offset_utf16 = self.editor.utf8_to_utf16_index(offset);
        caps_mode(&mut ctx.env, text, offset_utf16, req_modes)
    }

    fn delete_surrounding_text(
        &mut self,
        ctx: &mut CallbackCtx,
        before_length: jint,
        after_length: jint,
    ) -> bool {
        if before_length > 0 {
            let sel_range = self.editor.editor().raw_selection().text_range();
            let sel_start_utf16 = self.editor.utf8_to_utf16_index(sel_range.start);
            let before_start_utf16 = sel_start_utf16.saturating_sub(before_length as usize);
            let before_start = self.editor.utf16_to_utf8_index(before_start_utf16);
            if let Some(len) = NonZeroUsize::new(sel_range.start - before_start) {
                let mut drv = self.editor.driver();
                drv.delete_bytes_before_selection(len);
            }
        }
        if after_length > 0 {
            let sel_range = self.editor.editor().raw_selection().text_range();
            let sel_end_utf16 = self.editor.utf8_to_utf16_index(sel_range.end);
            let len_utf16 = self
                .editor
                .utf8_to_utf16_index(self.editor.editor().raw_text().len());
            let after_end_utf16 = sel_end_utf16
                .saturating_add(after_length as usize)
                .min(len_utf16);
            let after_end = self.editor.utf16_to_utf8_index(after_end_utf16);
            if let Some(len) = NonZeroUsize::new(after_end - sel_range.end) {
                let mut drv = self.editor.driver();
                drv.delete_bytes_after_selection(len);
            }
        }
        self.enqueue_render_if_needed(ctx);
        true
    }

    fn delete_surrounding_text_in_code_points(
        &mut self,
        ctx: &mut CallbackCtx,
        before_length: jint,
        after_length: jint,
    ) -> bool {
        if before_length > 0 {
            let sel_range = self.editor.editor().raw_selection().text_range();
            let sel_start_usv = self.editor.utf8_to_usv_index(sel_range.start);
            let before_start_usv = sel_start_usv.saturating_sub(before_length as usize);
            let before_start = self.editor.usv_to_utf8_index(before_start_usv);
            if let Some(len) = NonZeroUsize::new(sel_range.start - before_start) {
                let mut drv = self.editor.driver();
                drv.delete_bytes_before_selection(len);
            }
        }
        if after_length > 0 {
            let sel_range = self.editor.editor().raw_selection().text_range();
            let sel_end_usv = self.editor.utf8_to_usv_index(sel_range.end);
            let len_usv = self
                .editor
                .utf8_to_usv_index(self.editor.editor().raw_text().len());
            let after_end_usv = sel_end_usv
                .saturating_add(after_length as usize)
                .min(len_usv);
            let after_end = self.editor.usv_to_utf8_index(after_end_usv);
            if let Some(len) = NonZeroUsize::new(after_end - sel_range.end) {
                let mut drv = self.editor.driver();
                drv.delete_bytes_after_selection(len);
            }
        }
        self.enqueue_render_if_needed(ctx);
        true
    }

    fn set_composing_text(
        &mut self,
        ctx: &mut CallbackCtx,
        text: &str,
        new_cursor_position: jint,
    ) -> bool {
        self.set_composing_text_internal(text, new_cursor_position);
        self.enqueue_render_if_needed(ctx);
        true
    }

    fn set_composing_region(&mut self, ctx: &mut CallbackCtx, start: jint, end: jint) -> bool {
        let start = start.max(0) as usize;
        let end = end.max(0) as usize;
        let len_utf16 = self
            .editor
            .utf8_to_utf16_index(self.editor.editor().raw_text().len());
        let start = start.min(len_utf16);
        let end = end.min(len_utf16);
        let mut drv = self.editor.driver();
        if start == end {
            drv.finish_compose();
        } else {
            let (start, end) = (start.min(end), end.max(start));
            drv.set_compose_byte_range(start, end);
        }
        self.enqueue_render_if_needed(ctx);
        true
    }

    fn finish_composing_text(&mut self, ctx: &mut CallbackCtx) -> bool {
        let mut drv = self.editor.driver();
        drv.finish_compose();
        self.enqueue_render_if_needed(ctx);
        true
    }

    fn commit_text(
        &mut self,
        ctx: &mut CallbackCtx,
        text: &str,
        new_cursor_position: jint,
    ) -> bool {
        self.set_composing_text_internal(text, new_cursor_position);
        self.finish_composing_text(ctx)
    }

    fn set_selection(&mut self, ctx: &mut CallbackCtx, start: jint, end: jint) -> bool {
        if start < 0 || end < 0 {
            return false;
        }
        let start = self.editor.utf16_to_utf8_index(start as _);
        let end = self.editor.utf16_to_utf8_index(end as _);
        let mut drv = self.editor.driver();
        drv.select_byte_range(start, end);
        self.enqueue_render_if_needed(ctx);
        true
    }

    fn perform_editor_action(&mut self, _ctx: &mut CallbackCtx, _editor_action: jint) -> bool {
        // TODO: Do we need to implement this at all for this demo?
        // It would surely be needed for a proper framework implementation.
        false
    }

    fn begin_batch_edit(&mut self, _ctx: &mut CallbackCtx) -> bool {
        self.batch_edit_depth += 1;
        true
    }

    fn end_batch_edit(&mut self, ctx: &mut CallbackCtx) -> bool {
        if self.batch_edit_depth == 0 {
            return false;
        }
        self.batch_edit_depth -= 1;
        if self.batch_edit_depth == 0 {
            self.enqueue_render_if_needed(ctx);
            false
        } else {
            true
        }
    }

    fn send_key_event<'local>(
        &mut self,
        ctx: &mut CallbackCtx<'local>,
        event: &KeyEvent<'local>,
    ) -> bool {
        if !self
            .editor
            .on_keyboard_event(event.to_keyboard_event(&mut ctx.env))
        {
            return false;
        }
        self.enqueue_render_if_needed(ctx);
        true
    }

    fn request_cursor_updates(
        &mut self,
        _ctx: &mut CallbackCtx,
        _cursor_update_mode: jint,
    ) -> bool {
        // TODO: Do we need to implement this?
        false
    }
}

extern "system" fn new_view_peer<'local>(
    _env: JNIEnv<'local>,
    _view: View<'local>,
    _context: Context<'local>,
) -> jlong {
    let peer = DemoViewPeer {
        context: RenderContext::new(),
        renderers: vec![],
        render_surface: None,
        scene: Scene::new(),
        editor: text::Editor::new(text::LOREM),
        last_drawn_generation: Default::default(),
        ime_active: false,
        batch_edit_depth: 0,
        access_adapter: Default::default(),
        tap_counter: TapCounter::default(),
    };
    register_view_peer(peer)
}

/// Symbol run at JNI load time.
///
/// # Safety
/// There is no alternative, interacting with JNI is always unsafe at some level.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn JNI_OnLoad(vm: *mut RawJavaVM, _: *mut c_void) -> jint {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("android-view-demo"),
    );
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
        "org/linebender/android/viewdemo/DemoView",
        new_view_peer,
    );
    JNI_VERSION_1_6
}
