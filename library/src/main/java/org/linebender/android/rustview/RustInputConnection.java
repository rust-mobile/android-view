package org.linebender.android.rustview;

import android.os.Bundle;
import android.os.Handler;
import android.view.KeyEvent;
import android.view.inputmethod.CompletionInfo;
import android.view.inputmethod.CorrectionInfo;
import android.view.inputmethod.ExtractedText;
import android.view.inputmethod.ExtractedTextRequest;
import android.view.inputmethod.InputConnection;
import android.view.inputmethod.InputContentInfo;

class RustInputConnection implements InputConnection {
    private final RustView mView;

    RustInputConnection(RustView view) {
        mView = view;
    }

    private long getViewPeer() {
        return mView.mViewPeer;
    }

    @Override
    public CharSequence getTextBeforeCursor(int n, int flags) {
        return mView.getTextBeforeCursorNative(getViewPeer(), n);
    }

    @Override
    public CharSequence getTextAfterCursor(int n, int flags) {
        return mView.getTextAfterCursorNative(getViewPeer(), n);
    }

    @Override
    public CharSequence getSelectedText(int flags) {
        return mView.getSelectedTextNative(getViewPeer());
    }

    @Override
    public int getCursorCapsMode(int reqModes) {
        return mView.getCursorCapsModeNative(getViewPeer(), reqModes);
    }

    @Override
    public ExtractedText getExtractedText(ExtractedTextRequest request, int flags) {
        return null;
    }

    @Override
    public boolean deleteSurroundingText(int beforeLength, int afterLength) {
        return mView.deleteSurroundingTextNative(getViewPeer(), beforeLength, afterLength);
    }

    @Override
    public boolean deleteSurroundingTextInCodePoints(int beforeLength, int afterLength) {
        return mView.deleteSurroundingTextInCodePointsNative(getViewPeer(), beforeLength, afterLength);
    }

    @Override
    public boolean setComposingText(CharSequence text, int newCursorPosition) {
        return mView.setComposingTextNative(getViewPeer(), text.toString(), newCursorPosition);
    }

    @Override
    public boolean setComposingRegion(int start, int end) {
        return mView.setComposingRegionNative(getViewPeer(), start, end);
    }

    @Override
    public boolean finishComposingText() {
        return mView.finishComposingTextNative(getViewPeer());
    }

    @Override
    public boolean commitText(CharSequence text, int newCursorPosition) {
        return mView.commitTextNative(getViewPeer(), text.toString(), newCursorPosition);
    }

    @Override
    public boolean commitCompletion(CompletionInfo text) {
        return false;
    }

    @Override
    public boolean commitCorrection(CorrectionInfo correctionInfo) {
        return false;
    }

    @Override
    public boolean setSelection(int start, int end) {
        return mView.setSelectionNative(getViewPeer(), start, end);
    }

    @Override
    public boolean performEditorAction(int editorAction) {
        return mView.performEditorActionNative(getViewPeer(), editorAction);
    }

    @Override
    public boolean performContextMenuAction(int id) {
        return mView.performContextMenuActionNative(getViewPeer(), id);
    }

    @Override
    public boolean beginBatchEdit() {
        return mView.beginBatchEditNative(getViewPeer());
    }

    @Override
    public boolean endBatchEdit() {
        return mView.endBatchEditNative(getViewPeer());
    }

    @Override
    public boolean sendKeyEvent(KeyEvent event) {
        return mView.inputConnectionSendKeyEventNative(getViewPeer(), event);
    }

    @Override
    public boolean clearMetaKeyStates(int states) {
        return mView.inputConnectionClearMetaKeyStatesNative(getViewPeer(), states);
    }

    @Override
    public boolean reportFullscreenMode(boolean enabled) {
        return mView.inputConnectionReportFullscreenModeNative(getViewPeer(), enabled);
    }

    @Override
    public boolean performPrivateCommand(String action, Bundle data) {
        return false;
    }

    @Override
    public boolean requestCursorUpdates(int cursorUpdateMode) {
        return mView.requestCursorUpdatesNative(getViewPeer(), cursorUpdateMode);
    }

    @Override
    public Handler getHandler() {
        return null;
    }

    @Override
    public void closeConnection() {
        mView.closeInputConnectionNative(getViewPeer());
    }

    @Override
    public boolean commitContent(InputContentInfo inputContentInfo, int flags, Bundle opts) {
        return false;
    }
}
