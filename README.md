# WIP: Implement an Android view in Rust

This will be a library that can be reused to implement an Android view in Rust. Things that will distinguish it from `android-activity`:

* No C++ code.
* Meant to be usable for both pure-Rust Android applications and embedding Rust widgets in existing applications.
* Event handlers and other view callbacks run directly on the UI thread; there is no sending events to a separate Rust event loop thread.
* This crate intends to stick as close as possible to the Android framework. This will be especially important for text input support.

## Building and running the demos

### Common setup

```bash
export ANDROID_NDK_HOME="path/to/ndk"
export ANDROID_HOME="path/to/sdk"

rustup target add aarch64-linux-android
cargo install cargo-ndk
```

### Simple editor demo

```bash
cargo ndk -t arm64-v8a -o app/src/main/jniLibs/ build -p android-view-demo
./gradlew build
./gradlew installDebug
adb shell am start -n org.linebender.android.viewdemo/.DemoActivity
# To view logs:
adb shell run-as org.linebender.android.viewdemo logcat -v color
```

### Masonry demo

```bash
cargo ndk -t arm64-v8a -o masonry-app/src/main/jniLibs/ build -p android-view-masonry-demo
./gradlew build
./gradlew installDebug
adb shell am start -n org.linebender.android.masonrydemo/.DemoActivity
# To view logs:
adb shell run-as org.linebender.android.masonrydemo logcat -v color
```

## Open questions

* Do we need to be able to handle the view being reattached to a window after it has been detached? If not, then `onDetachedFromWindow` is the logical place to sever the connection between Java and native.
