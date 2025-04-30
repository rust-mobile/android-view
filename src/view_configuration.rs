//! A simple representation of Android `ViewConfiguration`.

use jni::{JNIEnv, errors::Error, objects::JObject};

/// A representation of Android `ViewConfiguration`.
///
/// This is a plain struct, and only contains non-deprecated values
/// available on API level 33 and below.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ViewConfiguration {
    /// Milliseconds between the first tap's up event and the
    /// subsequent tap's down event to detect a double tap.
    pub double_tap_timeout: i32,
    /// Milliseconds for a press to become a long press.
    pub long_press_timeout: i32,
    /// Milliseconds between a tap's up event and a subsequent
    /// tap's down event such that the subsequent tap is counted
    /// part of the same multi-tap sequence.
    pub multi_press_timeout: i32,
    /// Maximum pixels between a tap and a subsequent tap
    /// such that the subsequent tap can be counted as part of
    /// the same multi-tap sequence.
    pub scaled_double_tap_slop: i32,
    /// Scaling factor for the horizontal scroll axis value during
    /// `MotionAction::Scroll` for the number of pixels to scroll.
    pub scaled_horizontal_scroll_factor: f32,
    /// Maximum velocity, in pixels per second, to initiate a fling.
    pub scaled_maximum_fling_velocity: i32,
    /// Minimum velocity, in pixels per second, to initiate a fling.
    pub scaled_minimum_fling_velocity: i32,
    /// Minimum distance in pixels between touches before a gesture
    /// can be interpreted as scaling.
    pub scaled_minimum_scaling_span: i32,
    /// Pixels a touch can travel before a touch can be interpreted
    /// as a paging gesture.
    pub scaled_paging_touch_slop: i32,
    /// Perpendicular size of the scroll bar in pixels.
    pub scaled_scroll_bar_size: i32,
    /// Scaling factor for the vertical scroll axis value during
    /// `MotionAction::Scroll` for the number of pixels to scroll.
    pub scaled_vertical_scroll_factor: f32,
    /// Millisecond duration of the fade for inactive scrollbars.
    pub scroll_bar_fade_duration: i32,
    /// Milliseconds before inactive scrollbars begin to fade out.
    pub scroll_default_delay: i32,
    /// Friction factor for fling scrolls.
    pub scroll_friction: f32,
    /// Milliseconds to wait before deciding if a stationary touch
    /// is for a tap or for a tap.
    pub tap_timeout: i32,
    /// `true` when menus should display keyboard shortcut hints.
    pub should_show_menu_shortcuts_when_keyboard_present: bool,
}

impl ViewConfiguration {
    pub fn new<'local>(view: &JObject<'local>, env: &mut JNIEnv<'local>) -> Self {
        Self::try_new(view, env).unwrap_or_default()
    }

    fn try_new<'local>(view: &JObject<'local>, env: &mut JNIEnv<'local>) -> Result<Self, Error> {
        const CL: &str = "android/view/ViewConfiguration";

        let context = env
            .call_method(view, "getContext", "()Landroid/content/Context;", &[])?
            .l()?;

        let vc = env
            .call_static_method(
                CL,
                "get",
                "(Landroid/content/Context;)Landroid/view/ViewConfiguration;",
                &[(&context).into()],
            )?
            .l()?;

        Ok(Self {
            double_tap_timeout: env
                .call_static_method(CL, "getDoubleTapTimeout", "()I", &[])?
                .i()?,
            long_press_timeout: env
                .call_static_method(CL, "getLongPressTimeout", "()I", &[])?
                .i()?,
            multi_press_timeout: env
                .call_static_method(CL, "getMultiPressTimeout", "()I", &[])?
                .i()?,
            scaled_double_tap_slop: env
                .call_method(&vc, "getScaledDoubleTapSlop", "()I", &[])?
                .i()?,
            scaled_horizontal_scroll_factor: env
                .call_method(&vc, "getScaledHorizontalScrollFactor", "()F", &[])?
                .f()?,
            scaled_maximum_fling_velocity: env
                .call_method(&vc, "getScaledMaximumFlingVelocity", "()I", &[])?
                .i()?,
            scaled_minimum_fling_velocity: env
                .call_method(&vc, "getScaledMinimumFlingVelocity", "()I", &[])?
                .i()?,
            scaled_minimum_scaling_span: env
                .call_method(&vc, "getScaledMinimumScalingSpan", "()I", &[])?
                .i()?,
            scaled_paging_touch_slop: env
                .call_method(&vc, "getScaledPagingTouchSlop", "()I", &[])?
                .i()?,
            scaled_scroll_bar_size: env
                .call_method(&vc, "getScaledScrollBarSize", "()I", &[])?
                .i()?,
            scaled_vertical_scroll_factor: env
                .call_method(&vc, "getScaledVerticalScrollFactor", "()F", &[])?
                .f()?,
            scroll_bar_fade_duration: env
                .call_static_method(CL, "getScrollBarFadeDuration", "()I", &[])?
                .i()?,
            scroll_default_delay: env
                .call_static_method(CL, "getScrollDefaultDelay", "()I", &[])?
                .i()?,
            scroll_friction: env
                .call_static_method(CL, "getScrollFriction", "()F", &[])?
                .f()?,
            tap_timeout: env
                .call_static_method(CL, "getTapTimeout", "()I", &[])?
                .i()?,
            should_show_menu_shortcuts_when_keyboard_present: env
                .call_method(
                    &vc,
                    "shouldShowMenuShortcutsWhenKeyboardPresent",
                    "()Z",
                    &[],
                )?
                .z()?,
        })
    }
}
