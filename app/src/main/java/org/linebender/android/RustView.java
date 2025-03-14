package org.linebender.android;

import android.content.Context;
import android.graphics.Rect;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;

public abstract class RustView extends SurfaceView implements SurfaceHolder.Callback {
    private final long mViewPeer;

    protected abstract long newViewPeer(Context context);

    public RustView(Context context) {
        super(context);
        mViewPeer = newViewPeer(context);
        getHolder().addCallback(this);
    }

    private native int[] onMeasureNative(long peer, int widthSpec, int heightSpec);

    @Override
    protected void onMeasure(int widthSpec, int heightSpec) {
        int[] result = onMeasureNative(mViewPeer, widthSpec, heightSpec);
        if (result != null) {
            setMeasuredDimension(result[0], result[1]);
        } else {
            super.onMeasure(widthSpec, heightSpec);
        }
    }

    private native void onLayoutNative(
            long peer, boolean changed, int left, int top, int right, int bottom);

    @Override
    protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
        onLayoutNative(mViewPeer, changed, left, top, right, bottom);
        super.onLayout(changed, left, top, right, bottom);
    }

    private native void onSizeChangedNative(long peer, int w, int h, int oldw, int oldh);

    @Override
    protected void onSizeChanged(int w, int h, int oldw, int oldh) {
        onSizeChangedNative(mViewPeer, w, h, oldw, oldh);
        super.onSizeChanged(w, h, oldw, oldh);
    }

    private native boolean onKeyDownNative(long peer, int keyCode, KeyEvent event);

    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        return onKeyDownNative(mViewPeer, keyCode, event) || super.onKeyDown(keyCode, event);
    }

    private native boolean onKeyUpNative(long peer, int keyCode, KeyEvent event);

    @Override
    public boolean onKeyUp(int keyCode, KeyEvent event) {
        return onKeyUpNative(mViewPeer, keyCode, event) || super.onKeyUp(keyCode, event);
    }

    private native boolean onTrackballEventNative(long peer, MotionEvent event);

    @Override
    public boolean onTrackballEvent(MotionEvent event) {
        return onTrackballEventNative(mViewPeer, event) || super.onTrackballEvent(event);
    }

    private native boolean onTouchEventNative(long peer, MotionEvent event);

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        return onTouchEventNative(mViewPeer, event) || super.onTouchEvent(event);
    }

    private native boolean onGenericMotionEventNative(long peer, MotionEvent event);

    @Override
    public boolean onGenericMotionEvent(MotionEvent event) {
        return onGenericMotionEventNative(mViewPeer, event) || super.onGenericMotionEvent(event);
    }

    private native boolean onHoverEventNative(long peer, MotionEvent event);

    @Override
    public boolean onHoverEvent(MotionEvent event) {
        return onHoverEventNative(mViewPeer, event) || super.onHoverEvent(event);
    }

    private native void onFocusChangedNative(
            long peer, boolean gainFocus, int direction, Rect previouslyFocusedRect);

    @Override
    protected void onFocusChanged(boolean gainFocus, int direction, Rect previouslyFocusedRect) {
        super.onFocusChanged(gainFocus, direction, previouslyFocusedRect);
        onFocusChangedNative(mViewPeer, gainFocus, direction, previouslyFocusedRect);
    }

    private native void onWindowFocusChangedNative(long peer, boolean hasWindowFocus);

    @Override
    public void onWindowFocusChanged(boolean hasWindowFocus) {
        super.onWindowFocusChanged(hasWindowFocus);
        onWindowFocusChangedNative(mViewPeer, hasWindowFocus);
    }

    private native void onAttachedToWindowNative(long peer);

    @Override
    protected void onAttachedToWindow() {
        super.onAttachedToWindow();
        onAttachedToWindowNative(mViewPeer);
    }

    private native void onDetachedFromWindowNative(long peer);

    @Override
    protected void onDetachedFromWindow() {
        super.onDetachedFromWindow();
        onDetachedFromWindowNative(mViewPeer);
    }

    private native void onWindowVisibilityChangedNative(long peer, int visibility);

    @Override
    protected void onWindowVisibilityChanged(int visibility) {
        super.onWindowVisibilityChanged(visibility);
        onWindowVisibilityChangedNative(mViewPeer, visibility);
    }

    private native void surfaceCreatedNative(long peer, SurfaceHolder holder);

    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        surfaceCreatedNative(mViewPeer, holder);
    }

    private native void surfaceChangedNative(
            long peer, SurfaceHolder holder, int format, int width, int height);

    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        surfaceChangedNative(mViewPeer, holder, format, width, height);
    }

    private native void surfaceDestroyedNative(long peer, SurfaceHolder holder);

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        surfaceDestroyedNative(mViewPeer, holder);
    }
}
