package org.linebender.android.masonrydemo;

import android.content.Context;

import org.linebender.android.rustview.RustView;

public final class DemoView extends RustView {
    @Override
    protected native long newViewPeer(Context context);

    public DemoView(Context context) {
        super(context);
    }
}
