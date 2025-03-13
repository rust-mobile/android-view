package org.linebender.android.viewdemo;

import android.content.Context;

import org.linebender.android.RustView;

public final class DemoView extends RustView {
    @Override
    protected native long newNative(Context context);

    public DemoView(Context context) {
        super(context);
    }
}
