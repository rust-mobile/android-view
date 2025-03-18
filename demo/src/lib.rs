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
        sys::{JNI_VERSION_1_6, JavaVM as RawJavaVM, jint, jlong},
    },
    ndk::native_window::NativeWindow,
    *,
};
use anyhow::Result;
use log::LevelFilter;
use std::ffi::c_void;
use std::num::NonZeroUsize;
use std::time::Instant;
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
            antialiasing_support: vello::AaSupport::all(),
            num_init_threads: NonZeroUsize::new(1),
        },
    )
    .expect("Couldn't create renderer")
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

    access_adapter: accesskit_android::Adapter,
}

fn build_text_input_node(
    render_surface: &Option<RenderSurface>,
    editor: &mut text::Editor,
    update: &mut TreeUpdate,
) -> Node {
    let mut node = Node::new(Role::MultilineTextInput);
    if let Some(surface) = &render_surface {
        node.set_bounds(accesskit::Rect {
            x0: 0.0,
            y0: 0.0,
            x1: surface.config.width as _,
            y1: surface.config.height as _,
        });
    }
    editor.accessibility(update, &mut node);
    node
}

impl DemoViewPeer {
    fn build_text_input_node(&mut self, update: &mut TreeUpdate) -> Node {
        build_text_input_node(&self.render_surface, &mut self.editor, update)
    }

    fn build_initial_access_tree(&mut self) -> TreeUpdate {
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
        let view_class = env.get_object_class(&view.0).unwrap();
        let render_surface = &self.render_surface;
        let editor = &mut self.editor;
        self.access_adapter.update_if_active(
            || {
                let mut update = TreeUpdate {
                    nodes: vec![],
                    tree: None,
                    focus: TEXT_INPUT_ID,
                };
                let node = build_text_input_node(render_surface, editor, &mut update);
                update.nodes.push((TEXT_INPUT_ID, node));
                update
            },
            env,
            &view_class,
            &view.0,
        );

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
        if self.last_drawn_generation != self.editor.generation() {
            // Empty the scene of objects to draw. You could create a new Scene each time, but in this case
            // the same Scene is reused so that the underlying memory allocation can also be reused.
            self.scene.reset();

            self.last_drawn_generation = self.editor.draw(&mut self.scene);
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

    fn on_focus_changed<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &View<'local>,
        gain_focus: bool,
        _direction: jint,
        _previously_focused_rect: Option<&Rect<'local>>,
    ) {
        if self.render_surface.is_none() {
            return;
        }
        self.update_cursor_state(env, view, gain_focus);
        view.post_frame_callback(env);
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
        let editor = self.editor.editor();
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
        view.post_frame_callback(env);
        self.schedule_next_blink(env, view);
    }
}

impl ActivationHandler for DemoViewPeer {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(self.build_initial_access_tree())
    }
}

impl ActionHandlerWithAndroidContext for DemoViewPeer {
    fn do_action<'local>(
        &mut self,
        env: &mut JNIEnv<'local>,
        view: &JObject<'local>,
        req: ActionRequest,
    ) {
        // TODO: Is there a way to refactor android-view's wrappers so we don't
        // have to clone the local reference here?
        let view = View(env.new_local_ref(view).unwrap());
        if req.target == TEXT_INPUT_ID {
            self.editor.handle_accesskit_action_request(&req);
            if self.last_drawn_generation != self.editor.generation()
                && self.render_surface.is_some()
            {
                view.post_frame_callback(env);
            }
        }
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

    let vm = unsafe { JavaVM::from_raw(vm) }.unwrap();
    let mut env = vm.get_env().unwrap();
    register_view_class(
        &mut env,
        "org/linebender/android/viewdemo/DemoView",
        new_view_peer,
    );
    JNI_VERSION_1_6
}
