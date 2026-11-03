package com.cakecake.rust;

import android.app.NativeActivity;

/** 薄壳：接管系统返回键（onBackPressed 默认会 finish 掉整个 Activity）。 */
public class MainBack extends NativeActivity {
    @Override
    public void onBackPressed() {
        nativeBack();
    }

    private static native void nativeBack();
}
