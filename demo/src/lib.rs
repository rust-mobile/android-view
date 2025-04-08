// Derived from vello_editor
// Copyright 2024 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![deny(unsafe_op_in_unsafe_fn)]

use accesskit::{ActionRequest, ActivationHandler, Node, Role, Tree, TreeUpdate};
use accesskit_android::ActionHandlerWithAndroidContext;
use android_view::{
    jni::{
        JNIEnv, JavaVM,
        objects::JObject,
        sys::{JNI_VERSION_1_6, JavaVM as RawJavaVM, jfloat, jint, jlong},
    },
    ndk::{
        event::{KeyAction, Keycode},
        native_window::NativeWindow,
    },
    *,
};
use anyhow::Result;
use log::LevelFilter;
use std::borrow::Cow;
use std::ffi::c_void;
use std::num::NonZeroUsize;
use std::time::Instant;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vello::kurbo;
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
            surface_format: Some(surface.format),
            use_cpu: false,
            antialiasing_support: vello::AaSupport::area_only(),
            num_init_threads: None,
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

struct EditorAccessActionHandler<'a> {
    editor: &'a mut text::Editor,
    last_drawn_generation: &'a text::Generation,
    render_surface: &'a Option<RenderSurface<'static>>,
}

impl ActionHandlerWithAndroidContext for EditorAccessActionHandler<'_> {
    fn do_action<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &JObject<'local>,
        req: ActionRequest,
    ) {
        if req.target == TEXT_INPUT_ID {
            self.editor.handle_accesskit_action_request(&req);
            if *self.last_drawn_generation != self.editor.generation()
                && self.render_surface.is_some()
            {
                // TODO: Is there a way to refactor android-view's wrappers so
                // we don't have to clone the local reference here?
                let view = View(env.new_local_ref(view).unwrap());
                view.post_frame_callback(env);
            }
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

    /// The IME cursor area we last sent to the platform.
    last_sent_ime_cursor_area: kurbo::Rect,

    ime_active: bool,
    batch_edit_depth: usize,

    access_adapter: accesskit_android::Adapter,
}

impl DemoViewPeer {
    fn enqueue_render_if_needed<'local>(&mut self, env: &mut JNIEnv<'local>, view: &View<'local>) {
        if self.render_surface.is_none()
            || self.last_drawn_generation == self.editor.generation()
            || self.batch_edit_depth != 0
        {
            return;
        }
        view.post_frame_callback(env);
    }

    fn schedule_next_blink<'local>(&self, env: &mut JNIEnv<'local>, view: &View<'local>) {
        if let Some(next_time) = self.editor.next_blink_time() {
            let delay = next_time.duration_since(Instant::now());
            view.post_delayed(env, delay.as_millis() as _);
        }
    }

    fn update_cursor_state<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        focused: bool,
    ) {
        self.last_drawn_generation = Default::default();
        view.remove_delayed_callbacks(env);
        if focused {
            self.editor.cursor_reset();
            self.schedule_next_blink(env, view);
        } else {
            self.editor.disable_blink();
            self.editor.cursor_blink();
        }
    }

    fn render<'local>(&mut self, env: &mut JNIEnv<'local>, view: &View<'local>) {
        // Get the RenderSurface (surface + config).
        let surface = self.render_surface.as_ref().unwrap();

        // Get the window size.
        let width = surface.config.width;
        let height = surface.config.height;

        // Get a handle to the device.
        let device_handle = &self.context.devices[surface.dev_id];

        // Get the surface's texture.
        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");

        // Sometimes `Scene` is stale and needs to be redrawn.
        if self.last_drawn_generation != self.editor.generation() && self.batch_edit_depth == 0 {
            // Empty the scene of objects to draw. You could create a new Scene each time, but in this case
            // the same Scene is reused so that the underlying memory allocation can also be reused.
            self.scene.reset();

            self.last_drawn_generation = self.editor.draw(&mut self.scene);

            let view_class = env.get_object_class(&view.0).unwrap();
            let mut tree_source = EditorAccessTreeSource {
                render_surface: &self.render_surface,
                editor: &mut self.editor,
            };
            self.access_adapter.update_if_active(
                || {
                    let mut update = TreeUpdate {
                        nodes: vec![],
                        tree: None,
                        focus: TEXT_INPUT_ID,
                    };
                    let node = tree_source.build_text_input_node(&mut update);
                    update.nodes.push((TEXT_INPUT_ID, node));
                    update
                },
                env,
                &view_class,
                &view.0,
            );

            if self.ime_active {
                let imm = view.input_method_manager(env);
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
                imm.update_selection(env, view, sel_start, sel_end, comp_start, comp_end);
            }
        }

        // Render to the surface's texture.
        self.renderers[surface.dev_id]
            .as_mut()
            .unwrap()
            .render_to_surface(
                &device_handle.device,
                &device_handle.queue,
                &self.scene,
                &surface_texture,
                &vello::RenderParams {
                    base_color: Color::from_rgb8(30, 30, 30), // Background color
                    width,
                    height,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .expect("failed to render to surface");

        // Queue the texture to be presented on the surface.
        surface_texture.present();

        device_handle.device.poll(wgpu::Maintain::Poll);
    }
}

impl ViewPeer for DemoViewPeer {
    // TODO

    fn on_key_down<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        key_code: Keycode,
        event: &KeyEvent<'local>,
    ) -> bool {
        if !self.editor.on_key_down(env, key_code, event) {
            return false;
        }
        self.enqueue_render_if_needed(env, view);
        true
    }

    fn on_focus_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        gain_focus: bool,
        _direction: jint,
        _previously_focused_rect: Option<&Rect<'local>>,
    ) {
        self.update_cursor_state(env, view, gain_focus);
        self.enqueue_render_if_needed(env, view);
    }

    fn surface_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        holder: &SurfaceHolder<'local>,
        _format: jint,
        width: jint,
        height: jint,
    ) {
        let editor = self.editor.editor_mut();
        editor.set_scale(1.0);
        editor.set_width(Some(width as f32 - 2_f32 * text::INSET));
        self.last_drawn_generation = Default::default();
        let focused = view.is_focused(env);
        self.update_cursor_state(env, view, focused);

        let window = holder.surface(env).to_native_window(env);
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

        self.render(env, view);
    }

    fn surface_destroyed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        _holder: &SurfaceHolder<'local>,
    ) {
        self.render_surface = None;
        view.remove_frame_callback(env);
        view.remove_delayed_callbacks(env);
    }

    fn do_frame<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        _frame_time_nanos: jlong,
    ) {
        self.render(env, view)
    }

    fn delayed_callback<'local>(&mut self, env: &mut JNIEnv<'local>, view: &View<'local>) {
        self.editor.cursor_blink();
        self.last_drawn_generation = Default::default();
        self.enqueue_render_if_needed(env, view);
        self.schedule_next_blink(env, view);
    }

    fn populate_accessibility_node_info<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        host_screen_x: jint,
        host_screen_y: jint,
        virtual_view_id: jint,
        node_info: &JObject<'local>,
    ) -> bool {
        let mut tree_source = EditorAccessTreeSource {
            render_surface: &self.render_surface,
            editor: &mut self.editor,
        };
        self.access_adapter
            .populate_node_info(
                &mut tree_source,
                env,
                &view.0,
                host_screen_x,
                host_screen_y,
                virtual_view_id,
                node_info,
            )
            .unwrap()
    }

    fn input_focus<'local>(&mut self, env: &mut JNIEnv<'local>, view: &View<'local>) -> jint {
        let mut tree_source = EditorAccessTreeSource {
            render_surface: &self.render_surface,
            editor: &mut self.editor,
        };
        self.access_adapter
            .input_focus(&mut tree_source, env, &view.0)
    }

    fn virtual_view_at_point<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        x: jfloat,
        y: jfloat,
    ) -> jint {
        let mut tree_source = EditorAccessTreeSource {
            render_surface: &self.render_surface,
            editor: &mut self.editor,
        };
        self.access_adapter
            .virtual_view_at_point(&mut tree_source, env, &view.0, x, y)
    }

    fn perform_accessibility_action<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        virtual_view_id: jint,
        action: jint,
    ) -> bool {
        let mut action_handler = EditorAccessActionHandler {
            render_surface: &self.render_surface,
            editor: &mut self.editor,
            last_drawn_generation: &self.last_drawn_generation,
        };
        self.access_adapter.perform_action(
            &mut action_handler,
            env,
            &view.0,
            virtual_view_id,
            action,
        )
    }

    fn accessibility_set_text_selection<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        virtual_view_id: jint,
        anchor: jint,
        focus: jint,
    ) -> bool {
        let mut action_handler = EditorAccessActionHandler {
            render_surface: &self.render_surface,
            editor: &mut self.editor,
            last_drawn_generation: &self.last_drawn_generation,
        };
        let view_class = env.get_object_class(&view.0).unwrap();
        self.access_adapter.set_text_selection(
            &mut action_handler,
            env,
            &view_class,
            &view.0,
            virtual_view_id,
            anchor,
            focus,
        )
    }

    fn accessibility_collapse_text_selection<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        virtual_view_id: jint,
    ) -> bool {
        let mut action_handler = EditorAccessActionHandler {
            render_surface: &self.render_surface,
            editor: &mut self.editor,
            last_drawn_generation: &self.last_drawn_generation,
        };
        let view_class = env.get_object_class(&view.0).unwrap();
        self.access_adapter.collapse_text_selection(
            &mut action_handler,
            env,
            &view_class,
            &view.0,
            virtual_view_id,
        )
    }

    fn accessibility_traverse_text<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        virtual_view_id: jint,
        granularity: jint,
        forward: bool,
        extend_selection: bool,
    ) -> bool {
        let mut action_handler = EditorAccessActionHandler {
            render_surface: &self.render_surface,
            editor: &mut self.editor,
            last_drawn_generation: &self.last_drawn_generation,
        };
        let view_class = env.get_object_class(&view.0).unwrap();
        self.access_adapter.traverse_text(
            &mut action_handler,
            env,
            &view_class,
            &view.0,
            virtual_view_id,
            granularity,
            forward,
            extend_selection,
        )
    }

    fn as_input_connection(&mut self) -> &mut dyn InputConnection {
        self
    }
}

impl InputConnection for DemoViewPeer {
    fn text_before_cursor<'slf, 'local>(
        &'slf mut self,
        _env: &mut JNIEnv<'local>,
        _view: &View<'local>,
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

    fn text_after_cursor<'slf, 'local>(
        &'slf mut self,
        _env: &mut JNIEnv<'local>,
        _view: &View<'local>,
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

    fn selected_text<'slf, 'local>(
        &'slf mut self,
        _env: &mut JNIEnv<'local>,
        _view: &View<'local>,
    ) -> Option<Cow<'slf, str>> {
        self.editor.editor().selected_text().map(Cow::Borrowed)
    }

    fn cursor_caps_mode<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        _view: &View<'local>,
        req_modes: jint,
    ) -> jint {
        let editor = self.editor.editor();
        let text = editor.raw_text();
        let offset = editor.raw_selection().focus().index();
        let offset_utf16 = self.editor.utf8_to_utf16_index(offset);
        caps_mode(env, text, offset_utf16, req_modes)
    }

    fn delete_surrounding_text<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
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
        self.enqueue_render_if_needed(env, view);
        true
    }

    fn delete_surrounding_text_in_code_points<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
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
        self.enqueue_render_if_needed(env, view);
        true
    }

    fn set_composing_text<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        text: &str,
        new_cursor_position: jint,
    ) -> bool {
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
        drop(drv);
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
        self.enqueue_render_if_needed(env, view);
        true
    }

    fn set_composing_region<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        start: jint,
        end: jint,
    ) -> bool {
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
        self.enqueue_render_if_needed(env, view);
        true
    }

    fn finish_composing_text<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
    ) -> bool {
        let mut drv = self.editor.driver();
        drv.finish_compose();
        self.enqueue_render_if_needed(env, view);
        true
    }

    fn set_selection<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        start: jint,
        end: jint,
    ) -> bool {
        if start < 0 || end < 0 {
            return false;
        }
        let start = self.editor.utf16_to_utf8_index(start as _);
        let end = self.editor.utf16_to_utf8_index(end as _);
        let mut drv = self.editor.driver();
        drv.select_byte_range(start, end);
        self.enqueue_render_if_needed(env, view);
        true
    }

    fn perform_editor_action<'local>(
        &mut self,
        _env: &mut JNIEnv<'local>,
        _view: &View<'local>,
        _editor_action: jint,
    ) -> bool {
        // TODO: Do we need to implement this at all for this demo?
        // It would surely be needed for a proper framework implementation.
        false
    }

    fn begin_batch_edit<'local>(
        &mut self,
        _env: &mut JNIEnv<'local>,
        _view: &View<'local>,
    ) -> bool {
        self.batch_edit_depth += 1;
        true
    }

    fn end_batch_edit<'local>(&mut self, env: &mut JNIEnv<'local>, view: &View<'local>) -> bool {
        if self.batch_edit_depth == 0 {
            return false;
        }
        self.batch_edit_depth -= 1;
        if self.batch_edit_depth == 0 {
            self.enqueue_render_if_needed(env, view);
            false
        } else {
            true
        }
    }

    fn send_key_event<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        event: &KeyEvent<'local>,
    ) -> bool {
        if event.action(env) != KeyAction::Down {
            return false;
        }
        let key_code = event.key_code(env);
        self.on_key_down(env, view, key_code, event)
    }

    fn request_cursor_updates<'local>(
        &mut self,
        _env: &mut JNIEnv<'local>,
        _view: &View<'local>,
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
        last_sent_ime_cursor_area: kurbo::Rect::new(f64::NAN, f64::NAN, f64::NAN, f64::NAN),
        ime_active: false,
        batch_edit_depth: 0,
        access_adapter: Default::default(),
    };
    register_view_peer(peer)
}

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
