# WIP: Implement an Android view in Rust

*This doesn't work yet.*

This will be a reusable library that can be reused to implement an Android view in Rust. Things that will distinguish it from `android-activity`:

* No C++ code.
* Meant to be usable for both pure-Rust Android applications and embedding Rust widgets in existing applications.
* Event handlers and other view callbacks run directly on the UI thread; there is no sending events to a separate Rust event loop thread.
* This crate intends to stick as close as possible to the Android framework. This will be especially important for text input support.

So far, glue code has been written for implementing several Android framework methods (on both `View` and `SurfaceHolder.Callback`) in Rust, but it's not fully wired up yet, and the plumbing for `InputConnection` still needs to be done.
