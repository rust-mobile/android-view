use dpi::PhysicalPosition;
use jni::{
    JNIEnv,
    objects::JObject,
    sys::{jfloat, jint, jlong},
};
use ndk::event::{
    Axis, ButtonState, KeyAction, KeyEventFlags, Keycode, MetaState, MotionAction,
    MotionEventFlags, Source, ToolType,
};
use num_enum::FromPrimitive;
use ui_events::{
    ScrollDelta,
    keyboard::{KeyboardEvent, Modifiers},
    pointer::{ContactGeometry, PointerEvent, PointerId, PointerState, PointerUpdate},
};

use crate::ViewConfiguration;

#[repr(transparent)]
pub struct KeyEvent<'local>(pub JObject<'local>);

impl<'local> KeyEvent<'local> {
    pub fn device_id(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getDeviceId", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn source(&self, env: &mut JNIEnv<'local>) -> Source {
        Source::from_primitive(
            env.call_method(&self.0, "getSource", "()I", &[])
                .unwrap()
                .i()
                .unwrap(),
        )
    }

    pub fn action(&self, env: &mut JNIEnv<'local>) -> KeyAction {
        KeyAction::from_primitive(
            env.call_method(&self.0, "getAction", "()I", &[])
                .unwrap()
                .i()
                .unwrap(),
        )
    }

    pub fn event_time(&self, env: &mut JNIEnv<'local>) -> jlong {
        env.call_method(&self.0, "getEventTime", "()J", &[])
            .unwrap()
            .j()
            .unwrap()
    }

    pub fn down_time(&self, env: &mut JNIEnv<'local>) -> jlong {
        env.call_method(&self.0, "getDownTime", "()J", &[])
            .unwrap()
            .j()
            .unwrap()
    }

    pub fn flags(&self, env: &mut JNIEnv<'local>) -> KeyEventFlags {
        KeyEventFlags(
            env.call_method(&self.0, "getFlags", "()I", &[])
                .unwrap()
                .i()
                .unwrap() as u32,
        )
    }

    pub fn meta_state(&self, env: &mut JNIEnv<'local>) -> MetaState {
        MetaState(
            env.call_method(&self.0, "getMetaState", "()I", &[])
                .unwrap()
                .i()
                .unwrap() as u32,
        )
    }

    pub fn repeat_count(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getRepeatCount", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn key_code(&self, env: &mut JNIEnv<'local>) -> Keycode {
        Keycode::from_primitive(
            env.call_method(&self.0, "getKeyCode", "()I", &[])
                .unwrap()
                .i()
                .unwrap(),
        )
    }

    pub fn scan_code(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getScanCode", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn unicode_char(&self, env: &mut JNIEnv<'local>) -> Option<char> {
        let i = env
            .call_method(&self.0, "getUnicodeChar", "()I", &[])
            .unwrap()
            .i()
            .unwrap();
        if i <= 0 {
            return None;
        }
        char::from_u32(i as _)
    }

    pub fn to_keyboard_event(&self, env: &mut JNIEnv<'local>) -> KeyboardEvent {
        use ui_events::keyboard::{Key, KeyState, NamedKey, android};

        let key_code = self.key_code(env);

        KeyboardEvent {
            state: if self.action(env) == KeyAction::Down {
                KeyState::Down
            } else {
                KeyState::Up
            },
            key: match android::keycode_to_named_key(key_code.into()) {
                NamedKey::Unidentified => {
                    if let Some(c) = self.unicode_char(env) {
                        Key::Character(c.to_string())
                    } else {
                        Key::Named(NamedKey::Unidentified)
                    }
                }
                nk => Key::Named(nk),
            },
            code: android::keycode_to_code(key_code.into()),
            location: android::keycode_to_location(key_code.into()),
            modifiers: meta_state_to_modifiers(self.meta_state(env)),
            repeat: self.repeat_count(env) != 0,
            is_composing: false,
        }
    }
}

#[repr(transparent)]
pub struct MotionEvent<'local>(pub JObject<'local>);

impl<'local> MotionEvent<'local> {
    pub fn device_id(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getDeviceId", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn source(&self, env: &mut JNIEnv<'local>) -> Source {
        Source::from_primitive(
            env.call_method(&self.0, "getSource", "()I", &[])
                .unwrap()
                .i()
                .unwrap(),
        )
    }

    pub fn action(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getAction", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn action_button(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getActionButton", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn action_masked(&self, env: &mut JNIEnv<'local>) -> MotionAction {
        MotionAction::from_primitive(
            env.call_method(&self.0, "getActionMasked", "()I", &[])
                .unwrap()
                .i()
                .unwrap(),
        )
    }

    pub fn action_index(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getActionIndex", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn button_state(&self, env: &mut JNIEnv<'local>) -> ButtonState {
        ButtonState(
            env.call_method(&self.0, "getButtonState", "()I", &[])
                .unwrap()
                .i()
                .unwrap() as u32,
        )
    }

    pub fn event_time(&self, env: &mut JNIEnv<'local>) -> jlong {
        env.call_method(&self.0, "getEventTime", "()J", &[])
            .unwrap()
            .j()
            .unwrap()
    }

    pub fn event_time_nanos(&self, env: &mut JNIEnv<'local>) -> jlong {
        env.call_method(&self.0, "getEventTimeNanos", "()J", &[])
            .unwrap()
            .j()
            .unwrap()
    }

    pub fn historical_event_time_nanos(&self, env: &mut JNIEnv<'local>, pos: i32) -> jlong {
        env.call_method(
            &self.0,
            "getHistoricalEventTimeNanos",
            "(I)J",
            &[pos.into()],
        )
        .unwrap()
        .j()
        .unwrap()
    }

    pub fn down_time(&self, env: &mut JNIEnv<'local>) -> jlong {
        env.call_method(&self.0, "getDownTime", "()J", &[])
            .unwrap()
            .j()
            .unwrap()
    }

    pub fn flags(&self, env: &mut JNIEnv<'local>) -> MotionEventFlags {
        MotionEventFlags(
            env.call_method(&self.0, "getFlags", "()I", &[])
                .unwrap()
                .i()
                .unwrap() as u32,
        )
    }

    pub fn meta_state(&self, env: &mut JNIEnv<'local>) -> MetaState {
        MetaState(
            env.call_method(&self.0, "getMetaState", "()I", &[])
                .unwrap()
                .i()
                .unwrap() as u32,
        )
    }

    pub fn pointer_count(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getPointerCount", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn pointer_id(&self, env: &mut JNIEnv<'local>, pointer_index: jint) -> jint {
        env.call_method(&self.0, "getPointerId", "(I)I", &[pointer_index.into()])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn tool_type(&self, env: &mut JNIEnv<'local>, pointer_index: jint) -> ToolType {
        ToolType::from(
            env.call_method(&self.0, "getToolType", "(I)I", &[pointer_index.into()])
                .unwrap()
                .i()
                .unwrap(),
        )
    }

    pub fn x(&self, env: &mut JNIEnv<'local>) -> jfloat {
        env.call_method(&self.0, "getX", "()F", &[])
            .unwrap()
            .f()
            .unwrap()
    }

    pub fn x_at(&self, env: &mut JNIEnv<'local>, pointer_index: jint) -> jfloat {
        env.call_method(&self.0, "getX", "(I)F", &[pointer_index.into()])
            .unwrap()
            .f()
            .unwrap()
    }

    pub fn y(&self, env: &mut JNIEnv<'local>) -> jfloat {
        env.call_method(&self.0, "getY", "()F", &[])
            .unwrap()
            .f()
            .unwrap()
    }

    pub fn y_at(&self, env: &mut JNIEnv<'local>, pointer_index: jint) -> jfloat {
        env.call_method(&self.0, "getY", "(I)F", &[pointer_index.into()])
            .unwrap()
            .f()
            .unwrap()
    }

    pub fn pressure(&self, env: &mut JNIEnv<'local>) -> jfloat {
        env.call_method(&self.0, "getPressure", "()F", &[])
            .unwrap()
            .f()
            .unwrap()
    }

    pub fn history_size(&self, env: &mut JNIEnv<'local>) -> jint {
        env.call_method(&self.0, "getHistorySize", "()I", &[])
            .unwrap()
            .i()
            .unwrap()
    }

    pub fn historical_axis(
        &self,
        env: &mut JNIEnv<'local>,
        axis: Axis,
        pointer_index: i32,
        pos: i32,
    ) -> jfloat {
        env.call_method(
            &self.0,
            "getHistoricalAxisValue",
            "(III)F",
            &[i32::from(axis).into(), pointer_index.into(), pos.into()],
        )
        .unwrap()
        .f()
        .unwrap()
    }

    pub fn axis(&self, env: &mut JNIEnv<'local>, axis: Axis, pointer_index: jint) -> jfloat {
        env.call_method(
            &self.0,
            "getAxisValue",
            "(II)F",
            &[i32::from(axis).into(), pointer_index.into()],
        )
        .unwrap()
        .f()
        .unwrap()
    }

    pub fn to_pointer_event(
        &self,
        env: &mut JNIEnv<'local>,
        vc: &ViewConfiguration,
    ) -> Option<PointerEvent> {
        use ui_events::pointer::{
            PersistentDeviceId, PointerButton, PointerButtons, PointerId, PointerInfo,
            PointerOrientation, PointerState, PointerType, PointerUpdate,
        };

        let time = self.event_time_nanos(env) as u64;
        let action = self.action_masked(env);

        let action_index = self.action_index(env);
        let tool_type = self.tool_type(env, action_index);
        if tool_type == ToolType::Palm {
            // I don't think we have any useful way of handling this.
            return None;
        }
        let pointer = PointerInfo {
            pointer_id: match self.pointer_id(env, action_index) {
                n if n < 0 => None,
                n => PointerId::new(n as u64 + 1),
            },
            persistent_device_id: PersistentDeviceId::new(self.device_id(env) as u64),
            pointer_type: match tool_type {
                ToolType::Mouse => PointerType::Mouse,
                ToolType::Finger => PointerType::Touch,
                ToolType::Stylus | ToolType::Eraser => PointerType::Pen,
                _ => PointerType::Unknown,
            },
        };
        let buttons = {
            let mut pb = PointerButtons::default();
            let bs = self.button_state(env);
            if bs.primary() {
                pb |= PointerButton::Primary;
            }
            if bs.stylus_primary() {
                pb |= if tool_type == ToolType::Eraser {
                    PointerButton::PenEraser
                } else {
                    PointerButton::Primary
                };
            }
            if bs.secondary() || bs.stylus_secondary() {
                pb |= PointerButton::Secondary;
            }
            if bs.teriary() {
                pb |= PointerButton::Auxiliary;
            }
            if bs.back() {
                pb |= PointerButton::X1;
            }
            if bs.forward() {
                pb |= PointerButton::X2;
            }
            // TODO: verify this behavior.
            if tool_type == ToolType::Eraser && self.axis(env, Axis::Pressure, action_index) > 0.0 {
                pb |= PointerButton::PenEraser;
            }
            pb
        };
        let modifiers = meta_state_to_modifiers(self.meta_state(env));
        let orientation = if matches!(tool_type, ToolType::Stylus | ToolType::Eraser) {
            use core::f32::consts::FRAC_PI_2;
            let axis_orientation = self.axis(env, Axis::Orientation, action_index);
            let axis_tilt = self.axis(env, Axis::Tilt, action_index);
            let altitude = FRAC_PI_2 - axis_tilt;
            let azimuth = (-axis_orientation + 3.0 * FRAC_PI_2).rem_euclid(4.0 * FRAC_PI_2);
            PointerOrientation { altitude, azimuth }
        } else {
            Default::default()
        };
        let contact_geometry = if pointer.pointer_type == PointerType::Touch {
            let height = self.axis(env, Axis::TouchMajor, action_index) as f64;
            let width = self.axis(env, Axis::TouchMinor, action_index) as f64;
            (height > 0.0 && width > 0.0)
                .then_some(ContactGeometry { width, height })
                .unwrap_or_default()
        } else {
            Default::default()
        };
        let state = PointerState {
            time,
            position: PhysicalPosition::<f64> {
                x: self.axis(env, Axis::X, action_index) as f64,
                y: self.axis(env, Axis::Y, action_index) as f64,
            },
            buttons,
            // `TapCounter` will attach an appropriate count.
            count: 0,
            modifiers,
            contact_geometry,
            orientation,
            pressure: self.axis(env, Axis::Pressure, action_index) * 0.5,
            tangential_pressure: 0.0,
        };

        let button = {
            // Button constants from <https://developer.android.com/reference/android/view/MotionEvent>.
            const BUTTON_PRIMARY: jint = 0b1;
            const BUTTON_STYLUS_PRIMARY: jint = 0b100000;
            const BUTTON_SECONDARY: jint = 0b10;
            const BUTTON_STYLUS_SECONDARY: jint = 0b1000000;
            const BUTTON_TERTIARY: jint = 0b100;
            const BUTTON_BACK: jint = 0b1000;
            const BUTTON_FORWARD: jint = 0b10000;
            match self.action_button(env) {
                BUTTON_PRIMARY | BUTTON_STYLUS_PRIMARY => Some(PointerButton::Primary),
                BUTTON_SECONDARY | BUTTON_STYLUS_SECONDARY => Some(PointerButton::Secondary),
                BUTTON_TERTIARY => Some(PointerButton::Auxiliary),
                BUTTON_BACK => Some(PointerButton::X1),
                BUTTON_FORWARD => Some(PointerButton::X2),
                _ => (tool_type == ToolType::Eraser).then_some(PointerButton::PenEraser),
            }
        };

        Some(match action {
            MotionAction::Down | MotionAction::PointerDown => PointerEvent::Down {
                pointer,
                state,
                button,
            },
            MotionAction::Up | MotionAction::PointerUp => PointerEvent::Up {
                pointer,
                state,
                button,
            },
            MotionAction::Move | MotionAction::HoverMove => {
                let hsz = self.history_size(env);
                let mut coalesced: Vec<PointerState> = vec![state.clone(); hsz as usize];
                for pos in 0..hsz {
                    let i = pos as usize;
                    coalesced[i].time = self.historical_event_time_nanos(env, pos) as u64;
                    coalesced[i].position = PhysicalPosition::<f64> {
                        x: self.historical_axis(env, Axis::X, action_index, pos) as f64,
                        y: self.historical_axis(env, Axis::Y, action_index, pos) as f64,
                    };
                    coalesced[i].contact_geometry = if pointer.pointer_type == PointerType::Touch {
                        let height =
                            self.historical_axis(env, Axis::TouchMajor, action_index, pos) as f64;
                        let width =
                            self.historical_axis(env, Axis::TouchMinor, action_index, pos) as f64;
                        (height > 0.0 && width > 0.0)
                            .then_some(ContactGeometry { width, height })
                            .unwrap_or_default()
                    } else {
                        Default::default()
                    };
                    coalesced[i].pressure =
                        self.historical_axis(env, Axis::Pressure, action_index, pos) * 0.5;
                    coalesced[i].orientation =
                        if matches!(tool_type, ToolType::Stylus | ToolType::Eraser) {
                            use core::f32::consts::FRAC_PI_2;
                            let axis_orientation =
                                self.historical_axis(env, Axis::Orientation, action_index, pos);
                            let axis_tilt =
                                self.historical_axis(env, Axis::Tilt, action_index, pos);
                            let altitude = FRAC_PI_2 - axis_tilt;
                            let azimuth =
                                (-axis_orientation + 3.0 * FRAC_PI_2).rem_euclid(4.0 * FRAC_PI_2);
                            PointerOrientation { altitude, azimuth }
                        } else {
                            Default::default()
                        };
                }

                PointerEvent::Move(PointerUpdate {
                    pointer,
                    current: state,
                    coalesced,
                    // TODO: map predicted events
                    predicted: vec![],
                })
            }
            MotionAction::Cancel => PointerEvent::Cancel(pointer),
            MotionAction::HoverEnter => PointerEvent::Enter(pointer),
            MotionAction::HoverExit => PointerEvent::Leave(pointer),
            MotionAction::Scroll => PointerEvent::Scroll {
                pointer,
                delta: ScrollDelta::PixelDelta(PhysicalPosition::<f64> {
                    x: (self.axis(env, Axis::Hscroll, action_index)
                        * vc.scaled_horizontal_scroll_factor) as f64,
                    y: (self.axis(env, Axis::Vscroll, action_index)
                        * vc.scaled_vertical_scroll_factor) as f64,
                }),
                state,
            },
            _ => {
                // Other current `MotionAction` values relate to gamepad/joystick buttons;
                // ui-events doesn't currently have types for these, so consider them unhandled.
                return None;
            }
        })
    }
}

/// Convert `MetaState` to `Modifiers`.
fn meta_state_to_modifiers(s: MetaState) -> Modifiers {
    let mut m = Modifiers::default();
    if s.caps_lock_on() {
        m |= Modifiers::CAPS_LOCK;
    }
    if s.scroll_lock_on() {
        m |= Modifiers::SCROLL_LOCK;
    }
    if s.num_lock_on() {
        m |= Modifiers::NUM_LOCK;
    }
    if s.sym_on() {
        m |= Modifiers::SYMBOL;
    }
    if s.shift_on() {
        m |= Modifiers::SHIFT;
    }
    if s.alt_on() {
        m |= Modifiers::ALT;
    }
    if s.ctrl_on() {
        m |= Modifiers::CONTROL;
    }
    if s.function_on() {
        m |= Modifiers::FN;
    }
    if s.meta_on() {
        m |= Modifiers::META;
    }
    m
}

/// State related to detecting taps for tap counting.
#[derive(Clone, Debug)]
struct TapState {
    /// Pointer ID associated, used for attaching counts
    /// to `PointerEvent::Move` and `PointerEvent::Up`.
    /// Ignored for tap counting, because pointer id can
    /// change between taps in a multi-tap.
    pointer_id: Option<PointerId>,
    /// Nanosecond timestamp when the tap went Down.
    down_time: u64,
    /// Nanosecond timestamp when the tap went Up.
    ///
    /// Resets to `down_time` when tap goes Down.
    up_time: u64,
    /// The local tap count as of the last Down phase.
    count: u8,
    /// x coordinate.
    x: f64,
    /// y coordinate.
    y: f64,
}

/// Track and apply tap counts for `PointerEvent`.
#[derive(Default)]
pub struct TapCounter {
    /// The `ViewConfiguration` which configures tap counting.
    pub vc: ViewConfiguration,
    /// Recent taps which can be used for tap counting.
    taps: Vec<TapState>,
}

impl TapCounter {
    /// Make a new `TapCounter` with `ViewConfiguration` from your view.
    pub fn new(vc: ViewConfiguration) -> Self {
        Self { vc, taps: vec![] }
    }

    /// Enhance a `PointerEvent` with `count`.
    ///
    pub fn attach_count(&mut self, e: PointerEvent) -> PointerEvent {
        match e {
            PointerEvent::Down {
                button,
                pointer,
                state,
            } => {
                let e = if let Some(i) =
                    self.taps.iter().position(|TapState { x, y, up_time, .. }| {
                        let dx = (x - state.position.x).abs();
                        let dy = (y - state.position.y).abs();
                        (dx * dx + dy * dy).sqrt() < self.vc.scaled_double_tap_slop as f64
                            && (up_time + self.vc.multi_press_timeout.max(400) as u64 * 1000000)
                                > state.time
                    }) {
                    let count = self.taps[i].count + 1;
                    self.taps[i].count = count;
                    self.taps[i].pointer_id = pointer.pointer_id;
                    self.taps[i].down_time = state.time;
                    self.taps[i].up_time = state.time;
                    self.taps[i].x = state.position.x;
                    self.taps[i].y = state.position.y;

                    PointerEvent::Down {
                        button,
                        pointer,
                        state: PointerState { count, ..state },
                    }
                } else {
                    let s = TapState {
                        pointer_id: pointer.pointer_id,
                        down_time: state.time,
                        up_time: state.time,
                        count: 1,
                        x: state.position.x,
                        y: state.position.y,
                    };
                    self.taps.push(s);
                    PointerEvent::Down {
                        button,
                        pointer,
                        state: PointerState { count: 1, ..state },
                    }
                };
                self.clear_expired(state.time);
                e
            }
            PointerEvent::Up {
                button,
                pointer,
                ref state,
            } => {
                if let Some(i) = self
                    .taps
                    .iter()
                    .position(|TapState { pointer_id, .. }| *pointer_id == pointer.pointer_id)
                {
                    self.taps[i].up_time = state.time;
                    PointerEvent::Up {
                        button,
                        pointer,
                        state: PointerState {
                            count: self.taps[i].count,
                            ..state.clone()
                        },
                    }
                } else {
                    e.clone()
                }
            }
            PointerEvent::Move(PointerUpdate {
                pointer,
                ref current,
                ref coalesced,
                ref predicted,
            }) => {
                if let Some(TapState { count, .. }) = self
                    .taps
                    .iter()
                    .find(
                        |TapState {
                             pointer_id,
                             down_time,
                             up_time,
                             ..
                         }| {
                            *pointer_id == pointer.pointer_id && down_time == up_time
                        },
                    )
                    .cloned()
                {
                    PointerEvent::Move(PointerUpdate {
                        pointer,
                        current: PointerState {
                            count,
                            ..current.clone()
                        },
                        coalesced: coalesced
                            .iter()
                            .cloned()
                            .map(|u| PointerState { count, ..u })
                            .collect(),
                        predicted: predicted
                            .iter()
                            .cloned()
                            .map(|u| PointerState { count, ..u })
                            .collect(),
                    })
                } else {
                    e
                }
            }
            PointerEvent::Cancel(p) | PointerEvent::Leave(p) => {
                self.taps
                    .retain(|TapState { pointer_id, .. }| *pointer_id != p.pointer_id);
                e.clone()
            }
            PointerEvent::Enter(..) | PointerEvent::Scroll { .. } => e.clone(),
        }
    }

    /// Clear expired taps.
    ///
    /// `t` is the time of the last received event.
    /// All events have the same time base on Android, so this is valid here.
    fn clear_expired(&mut self, t: u64) {
        self.taps.retain(
            |TapState {
                 down_time, up_time, ..
             }| {
                down_time == up_time
                    || (up_time + self.vc.multi_press_timeout.max(400) as u64 * 1000000) > t
            },
        );
    }
}
