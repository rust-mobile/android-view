package org.linebender.android;

import android.content.Context;
import android.graphics.Rect;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;

public abstract class RustView extends SurfaceView implements SurfaceHolder.Callback {
    private final long mHandle;

    protected abstract long newNative(Context context);

    public RustView(Context context) {
        super(context);
        mHandle = newNative(context);
        getHolder().addCallback(this);
    }

    private native int[] onMeasureNative(long handle, int widthSpec, int heightSpec);

    @Override
    protected void onMeasure(int widthSpec, int heightSpec) {
        int[] result = onMeasureNative(mHandle, widthSpec, heightSpec);
        if (result != null) {
            setMeasuredDimension(result[0], result[1]);
        } else {
            super.onMeasure(widthSpec, heightSpec);
        }
    }

    private native void onLayoutNative(
            long handle, boolean changed, int left, int top, int right, int bottom);

    @Override
    protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
        onLayoutNative(mHandle, changed, left, top, right, bottom);
        super.onLayout(changed, left, top, right, bottom);
    }

    private native void onSizeChangedNative(long handle, int w, int h, int oldw, int oldh);

    @Override
    protected void onSizeChanged(int w, int h, int oldw, int oldh) {
        onSizeChangedNative(mHandle, w, h, oldw, oldh);
        super.onSizeChanged(w, h, oldw, oldh);
    }

    private native boolean onKeyDownNative(long handle, int keyCode, KeyEvent event);

    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        return onKeyDownNative(mHandle, keyCode, event) || super.onKeyDown(keyCode, event);
    }

    private native boolean onKeyUpNative(long handle, int keyCode, KeyEvent event);

    @Override
    public boolean onKeyUp(int keyCode, KeyEvent event) {
        return onKeyUpNative(mHandle, keyCode, event) || super.onKeyUp(keyCode, event);
    }

    private native boolean onTrackballEventNative(long handle, MotionEvent event);

    @Override
    public boolean onTrackballEvent(MotionEvent event) {
        return onTrackballEventNative(mHandle, event) || super.onTrackballEvent(event);
    }

    private native boolean onTouchEventNative(long handle, MotionEvent event);

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        return onTouchEventNative(mHandle, event) || super.onTouchEvent(event);
    }

    private native boolean onGenericMotionEventNative(long handle, MotionEvent event);

    @Override
    public boolean onGenericMotionEvent(MotionEvent event) {
        return onGenericMotionEventNative(mHandle, event) || super.onGenericMotionEvent(event);
    }

    private native boolean onHoverEventNative(long handle, MotionEvent event);

    @Override
    public boolean onHoverEvent(MotionEvent event) {
        return onHoverEventNative(mHandle, event) || super.onHoverEvent(event);
    }

    private native void onFocusChangedNative(
            long handle, boolean gainFocus, int direction, Rect previouslyFocusedRect);

    @Override
    protected void onFocusChanged(boolean gainFocus, int direction, Rect previouslyFocusedRect) {
        super.onFocusChanged(gainFocus, direction, previouslyFocusedRect);
        onFocusChangedNative(mHandle, gainFocus, direction, previouslyFocusedRect);
    }

    private native void onWindowFocusChangedNative(long handle, boolean hasWindowFocus);

    @Override
    public void onWindowFocusChanged(boolean hasWindowFocus) {
        super.onWindowFocusChanged(hasWindowFocus);
        onWindowFocusChangedNative(mHandle, hasWindowFocus);
    }

    private native void onAttachedToWindowNative(long handle);

    @Override
    protected void onAttachedToWindow() {
        super.onAttachedToWindow();
        onAttachedToWindowNative(mHandle);
    }

    private native void onDetachedFromWindowNative(long handle);

    @Override
    protected void onDetachedFromWindow() {
        super.onDetachedFromWindow();
        onDetachedFromWindowNative(mHandle);
    }

    private native void onWindowVisibilityChangedNative(long handle, int visibility);

    @Override
    protected void onWindowVisibilityChanged(int visibility) {
        super.onWindowVisibilityChanged(visibility);
        onWindowVisibilityChangedNative(mHandle, visibility);
    }

    private native void surfaceCreatedNative(long handle, SurfaceHolder holder);

    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        surfaceCreatedNative(mHandle, holder);
    }

    private native void surfaceChangedNative(
            long handle, SurfaceHolder holder, int format, int width, int height);

    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        surfaceChangedNative(mHandle, holder, format, width, height);
    }

    private native void surfaceDestroyedNative(long handle, SurfaceHolder holder);

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        surfaceDestroyedNative(mHandle, holder);
    }
}
