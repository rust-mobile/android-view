package org.linebender.android.rustview;

import android.content.Context;
import android.graphics.Rect;
import android.os.Bundle;
import android.view.Choreographer;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.accessibility.AccessibilityNodeInfo;
import android.view.accessibility.AccessibilityNodeProvider;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputConnection;
import android.view.inputmethod.InputMethodManager;

public abstract class RustView extends SurfaceView
        implements SurfaceHolder.Callback, Choreographer.FrameCallback {
    final long mViewPeer;
    final InputMethodManager mInputMethodManager;

    protected abstract long newViewPeer(Context context);

    public RustView(Context context) {
        super(context);
        mViewPeer = newViewPeer(context);
        getHolder().addCallback(this);
        mInputMethodManager =
                (InputMethodManager) context.getSystemService(Context.INPUT_METHOD_SERVICE);
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

    void postFrameCallback() {
        Choreographer c = Choreographer.getInstance();
        c.removeFrameCallback(this);
        c.postFrameCallback(this);
    }

    void removeFrameCallback() {
        Choreographer.getInstance().removeFrameCallback(this);
    }

    private native void doFrameNative(long peer, long frameTimeNanos);

    @Override
    public void doFrame(long frameTimeNanos) {
        doFrameNative(mViewPeer, frameTimeNanos);
    }

    private native void delayedCallbackNative(long peer);

    private final Runnable mDelayedCallback =
            new Runnable() {
                @Override
                public void run() {
                    delayedCallbackNative(mViewPeer);
                }
            };

    boolean postDelayed(long delayMillis) {
        return postDelayed(mDelayedCallback, delayMillis);
    }

    boolean removeDelayedCallbacks() {
        return removeCallbacks(mDelayedCallback);
    }

    private native boolean hasAccessibilityNodeProviderNative(long peer);

    private native AccessibilityNodeInfo createAccessibilityNodeInfoNative(
            long peer, int virtualViewId);

    private native AccessibilityNodeInfo accessibilityFindFocusNative(long peer, int virtualViewId);

    private native boolean performAccessibilityActionNative(
            long peer, int virtualViewId, int action, Bundle arguments);

    @Override
    public AccessibilityNodeProvider getAccessibilityNodeProvider() {
        if (!hasAccessibilityNodeProviderNative(mViewPeer)) {
            return super.getAccessibilityNodeProvider();
        }
        return new AccessibilityNodeProvider() {
            @Override
            public AccessibilityNodeInfo createAccessibilityNodeInfo(int virtualViewId) {
                return createAccessibilityNodeInfoNative(mViewPeer, virtualViewId);
            }

            @Override
            public AccessibilityNodeInfo findFocus(int focusType) {
                return accessibilityFindFocusNative(mViewPeer, focusType);
            }

            @Override
            public boolean performAction(int virtualViewId, int action, Bundle arguments) {
                return performAccessibilityActionNative(
                        mViewPeer, virtualViewId, action, arguments);
            }
        };
    }

    private native boolean onCreateInputConnectionNative(long peer, EditorInfo outAttrs);

    @Override
    public InputConnection onCreateInputConnection(EditorInfo outAttrs) {
        if (!onCreateInputConnectionNative(mViewPeer, outAttrs)) {
            return null;
        }
        return new RustInputConnection(this);
    }

    native String getTextBeforeCursorNative(long peer, int n);

    native String getTextAfterCursorNative(long peer, int n);

    native String getSelectedTextNative(long peer);

    native int getCursorCapsModeNative(long peer, int reqModes);

    native boolean deleteSurroundingTextNative(long peer, int beforeLength, int afterLength);

    native boolean deleteSurroundingTextInCodePointsNative(
            long peer, int beforeLength, int afterLength);

    native boolean setComposingTextNative(long peer, String text, int newCursorPosition);

    native boolean setComposingRegionNative(long peer, int start, int end);

    native boolean finishComposingTextNative(long peer);

    native boolean commitTextNative(long peer, String text, int newCursorPosition);

    native boolean setSelectionNative(long peer, int start, int end);

    native boolean performEditorActionNative(long peer, int editorAction);

    native boolean performContextMenuActionNative(long peer, int id);

    native boolean beginBatchEditNative(long peer);

    native boolean endBatchEditNative(long peer);

    native boolean inputConnectionSendKeyEventNative(long peer, KeyEvent event);

    native boolean inputConnectionClearMetaKeyStatesNative(long peer, int states);

    native boolean inputConnectionReportFullscreenModeNative(long peer, boolean enabled);

    native boolean requestCursorUpdatesNative(long peer, int cursorUpdateMode);

    native void closeInputConnectionNative(long peer);
}
