// Derived from AccessKit
// Copyright 2025 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from the Flutter engine.
// Copyright 2013 The Flutter Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

package org.linebender.android;

import android.content.Context;
import android.graphics.Rect;
import android.os.Bundle;
import android.view.Choreographer;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.view.accessibility.AccessibilityEvent;
import android.view.accessibility.AccessibilityNodeInfo;
import android.view.accessibility.AccessibilityNodeInfo.AccessibilityAction;
import android.view.accessibility.AccessibilityNodeProvider;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputConnection;
import android.view.inputmethod.InputMethodManager;

public abstract class RustView extends SurfaceView implements SurfaceHolder.Callback, Choreographer.FrameCallback {
    final long mViewPeer;
    final InputMethodManager mInputMethodManager;

    protected abstract long newViewPeer(Context context);

    public RustView(Context context) {
        super(context);
        mViewPeer = newViewPeer(context);
        getHolder().addCallback(this);
        mInputMethodManager = (InputMethodManager)context.getSystemService(Context.INPUT_METHOD_SERVICE);
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

    @Override
    public boolean onHoverEvent(MotionEvent event) {
        switch (event.getAction()) {
            case MotionEvent.ACTION_HOVER_ENTER:
            case MotionEvent.ACTION_HOVER_MOVE:
                int newId = getVirtualViewAtPointNative(mViewPeer, event.getX(), event.getY());
                if (newId != hoverId) {
                    if (newId != AccessibilityNodeProvider.HOST_VIEW_ID) {
                        sendEventInternal(this, newId, AccessibilityEvent.TYPE_VIEW_HOVER_ENTER);
                    }
                    if (hoverId != AccessibilityNodeProvider.HOST_VIEW_ID) {
                        sendEventInternal(this, hoverId, AccessibilityEvent.TYPE_VIEW_HOVER_EXIT);
                    }
                    hoverId = newId;
                }
                break;
            case MotionEvent.ACTION_HOVER_EXIT:
                if (hoverId != AccessibilityNodeProvider.HOST_VIEW_ID) {
                    sendEventInternal(this, hoverId, AccessibilityEvent.TYPE_VIEW_HOVER_EXIT);
                    hoverId = AccessibilityNodeProvider.HOST_VIEW_ID;
                }
                break;
        }
        return true;
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

    private final Runnable mDelayedCallback = new Runnable() {
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

    private int accessibilityFocus = AccessibilityNodeProvider.HOST_VIEW_ID;
    private int hoverId = AccessibilityNodeProvider.HOST_VIEW_ID;

    private static AccessibilityEvent newEvent(View host, int virtualViewId, int type) {
        AccessibilityEvent e = AccessibilityEvent.obtain(type);
        e.setPackageName(host.getContext().getPackageName());
        if (virtualViewId == AccessibilityNodeProvider.HOST_VIEW_ID) {
            e.setSource(host);
        } else {
            e.setSource(host, virtualViewId);
        }
        return e;
    }

    private static void sendCompletedEvent(View host, AccessibilityEvent e) {
        host.getParent().requestSendAccessibilityEvent(host, e);
    }

    private static void sendEventInternal(View host, int virtualViewId, int type) {
        AccessibilityEvent e = newEvent(host, virtualViewId, type);
        if (type == AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED) {
            e.setContentChangeTypes(AccessibilityEvent.CONTENT_CHANGE_TYPE_SUBTREE);
        }
        sendCompletedEvent(host, e);
    }

    public static void sendEvent(final View host, final int virtualViewId, final int type) {
        sendEventInternal(host, virtualViewId, type);
    }

    private static void sendTextChangedInternal(
            View host, int virtualViewId, String oldValue, String newValue) {
        int i;
        for (i = 0; i < oldValue.length() && i < newValue.length(); ++i) {
            if (oldValue.charAt(i) != newValue.charAt(i)) {
                break;
            }
        }
        if (i >= oldValue.length() && i >= newValue.length()) {
            return; // Text did not change
        }
        AccessibilityEvent e =
                newEvent(host, virtualViewId, AccessibilityEvent.TYPE_VIEW_TEXT_CHANGED);
        e.setBeforeText(oldValue);
        e.getText().add(newValue);
        int firstDifference = i;
        e.setFromIndex(firstDifference);
        int oldIndex = oldValue.length() - 1;
        int newIndex = newValue.length() - 1;
        while (oldIndex >= firstDifference && newIndex >= firstDifference) {
            if (oldValue.charAt(oldIndex) != newValue.charAt(newIndex)) {
                break;
            }
            --oldIndex;
            --newIndex;
        }
        e.setRemovedCount(oldIndex - firstDifference + 1);
        e.setAddedCount(newIndex - firstDifference + 1);
        sendCompletedEvent(host, e);
    }

    public static void sendTextChanged(
            final View host,
            final int virtualViewId,
            final String oldValue,
            final String newValue) {
        sendTextChangedInternal(host, virtualViewId, oldValue, newValue);
    }

    private static void sendTextSelectionChangedInternal(
            View host, int virtualViewId, String text, int start, int end) {
        AccessibilityEvent e =
                newEvent(host, virtualViewId, AccessibilityEvent.TYPE_VIEW_TEXT_SELECTION_CHANGED);
        e.getText().add(text);
        e.setFromIndex(start);
        e.setToIndex(end);
        e.setItemCount(text.length());
        sendCompletedEvent(host, e);
    }

    public static void sendTextSelectionChanged(
            final View host,
            final int virtualViewId,
            final String text,
            final int start,
            final int end) {
        sendTextSelectionChangedInternal(host, virtualViewId, text, start, end);
    }

    private static void sendTextTraversedInternal(
            View host,
            int virtualViewId,
            int granularity,
            boolean forward,
            int segmentStart,
            int segmentEnd) {
        AccessibilityEvent e =
                newEvent(
                        host,
                        virtualViewId,
                        AccessibilityEvent.TYPE_VIEW_TEXT_TRAVERSED_AT_MOVEMENT_GRANULARITY);
        e.setMovementGranularity(granularity);
        e.setAction(
                forward
                        ? AccessibilityNodeInfo.ACTION_NEXT_AT_MOVEMENT_GRANULARITY
                        : AccessibilityNodeInfo.ACTION_PREVIOUS_AT_MOVEMENT_GRANULARITY);
        e.setFromIndex(segmentStart);
        e.setToIndex(segmentEnd);
        sendCompletedEvent(host, e);
    }

    public static void sendTextTraversed(
            final View host,
            final int virtualViewId,
            final int granularity,
            final boolean forward,
            final int segmentStart,
            final int segmentEnd) {
        sendTextTraversedInternal(
                host,
                virtualViewId,
                granularity,
                forward,
                segmentStart,
                segmentEnd);
    }

    private native boolean populateAccessibilityNodeInfoNative(
            long peer,
            int screenX,
            int screenY,
            int virtualViewId,
            AccessibilityNodeInfo nodeInfo);

    private native int getInputFocusNative(long peer);

    private native int getVirtualViewAtPointNative(long peer, float x, float y);

    private native boolean performAccessibilityActionNative(long peer, int virtualViewId, int action);

    private native boolean accessibilitySetTextSelectionNative(long peer, int virtualViewId, int anchor, int focus);

    private native boolean accessibilityCollapseTextSelectionNative(long peer, int virtualViewId);

    private native boolean accessibilityTraverseTextNative(
            long peer,
            int virtualViewId,
            int granularity,
            boolean forward,
            boolean extendSelection);

    @Override
    public AccessibilityNodeProvider getAccessibilityNodeProvider() {
        return new AccessibilityNodeProvider() {
            @Override
            public AccessibilityNodeInfo createAccessibilityNodeInfo(int virtualViewId) {
                int[] location = new int[2];
                getLocationOnScreen(location);
                AccessibilityNodeInfo nodeInfo;
                if (virtualViewId == HOST_VIEW_ID) {
                    nodeInfo = AccessibilityNodeInfo.obtain(RustView.this);
                } else {
                    nodeInfo = AccessibilityNodeInfo.obtain(RustView.this, virtualViewId);
                }
                nodeInfo.setPackageName(getContext().getPackageName());
                nodeInfo.setVisibleToUser(true);
                if (!populateAccessibilityNodeInfoNative(
                        mViewPeer, location[0], location[1], virtualViewId, nodeInfo)) {
                    nodeInfo.recycle();
                    return null;
                }
                if (virtualViewId == accessibilityFocus) {
                    nodeInfo.setAccessibilityFocused(true);
                    nodeInfo.addAction(AccessibilityAction.ACTION_CLEAR_ACCESSIBILITY_FOCUS);
                } else {
                    nodeInfo.setAccessibilityFocused(false);
                    nodeInfo.addAction(AccessibilityAction.ACTION_ACCESSIBILITY_FOCUS);
                }
                return nodeInfo;
            }

            @Override
            public boolean performAction(int virtualViewId, int action, Bundle arguments) {
                switch (action) {
                    case AccessibilityNodeInfo.ACTION_ACCESSIBILITY_FOCUS:
                        accessibilityFocus = virtualViewId;
                        invalidate();
                        sendEventInternal(
                                RustView.this,
                                virtualViewId,
                                AccessibilityEvent.TYPE_VIEW_ACCESSIBILITY_FOCUSED);
                        return true;
                    case AccessibilityNodeInfo.ACTION_CLEAR_ACCESSIBILITY_FOCUS:
                        if (accessibilityFocus == virtualViewId) {
                            accessibilityFocus = AccessibilityNodeProvider.HOST_VIEW_ID;
                        }
                        invalidate();
                        sendEventInternal(
                                RustView.this,
                                virtualViewId,
                                AccessibilityEvent.TYPE_VIEW_ACCESSIBILITY_FOCUS_CLEARED);
                        return true;
                    case AccessibilityNodeInfo.ACTION_SET_SELECTION:
                        if (!(arguments != null
                                && arguments.containsKey(
                                        AccessibilityNodeInfo.ACTION_ARGUMENT_SELECTION_START_INT)
                                && arguments.containsKey(
                                        AccessibilityNodeInfo.ACTION_ARGUMENT_SELECTION_END_INT))) {
                            return accessibilityCollapseTextSelectionNative(
                                    mViewPeer, virtualViewId);
                        }
                        int anchor =
                                arguments.getInt(
                                        AccessibilityNodeInfo.ACTION_ARGUMENT_SELECTION_START_INT);
                        int focus =
                                arguments.getInt(
                                        AccessibilityNodeInfo.ACTION_ARGUMENT_SELECTION_END_INT);
                        return accessibilitySetTextSelectionNative(
                                mViewPeer, virtualViewId, anchor, focus);
                    case AccessibilityNodeInfo.ACTION_NEXT_AT_MOVEMENT_GRANULARITY:
                    case AccessibilityNodeInfo.ACTION_PREVIOUS_AT_MOVEMENT_GRANULARITY:
                        int granularity =
                                arguments.getInt(
                                        AccessibilityNodeInfo
                                                .ACTION_ARGUMENT_MOVEMENT_GRANULARITY_INT);
                        boolean forward =
                                (action
                                        == AccessibilityNodeInfo
                                                .ACTION_NEXT_AT_MOVEMENT_GRANULARITY);
                        boolean extendSelection =
                                arguments.getBoolean(
                                        AccessibilityNodeInfo
                                                .ACTION_ARGUMENT_EXTEND_SELECTION_BOOLEAN);
                        return accessibilityTraverseTextNative(
                                mViewPeer,
                                virtualViewId,
                                granularity,
                                forward,
                                extendSelection);
                }
                if (!performAccessibilityActionNative(mViewPeer, virtualViewId, action)) {
                    return false;
                }
                switch (action) {
                    case AccessibilityNodeInfo.ACTION_CLICK:
                        sendEventInternal(
                                RustView.this, virtualViewId, AccessibilityEvent.TYPE_VIEW_CLICKED);
                        break;
                }
                return true;
            }

            @Override
            public AccessibilityNodeInfo findFocus(int focusType) {
                switch (focusType) {
                    case AccessibilityNodeInfo.FOCUS_ACCESSIBILITY:
                        {
                            AccessibilityNodeInfo result =
                                    createAccessibilityNodeInfo(accessibilityFocus);
                            if (result != null && result.isAccessibilityFocused()) {
                                return result;
                            }
                            break;
                        }
                    case AccessibilityNodeInfo.FOCUS_INPUT:
                        {
                            AccessibilityNodeInfo result =
                                    createAccessibilityNodeInfo(getInputFocusNative(mViewPeer));
                            if (result != null && result.isFocused()) {
                                return result;
                            }
                            break;
                        }
                }
                return null;
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

    native boolean deleteSurroundingTextInCodePointsNative(long peer, int beforeLength, int afterLength);

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
