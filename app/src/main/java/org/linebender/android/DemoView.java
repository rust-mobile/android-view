package org.linebender.android;

import android.content.Context;

public final class DemoView extends RustView {
    @Override
    protected native long newNative(Context context);

    public DemoView(Context context) {
        super(context);
    }
}
