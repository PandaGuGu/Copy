package com.cakecake.rust;

import android.app.NativeActivity;
import android.content.Context;
import android.content.Intent;
import android.graphics.Bitmap;
import android.graphics.Color;
import android.media.MediaMetadataRetriever;
import android.net.Uri;
import android.os.Build;
import android.text.InputType;
import android.util.Log;
import android.view.KeyEvent;
import android.view.ViewGroup;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.widget.EditText;
import android.widget.TextView;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;

/**
 * 薄壳：接管系统返回键（onBackPressed 默认会 finish 掉整个 Activity）。
 *
 * NativeActivity 的窗口没有输入连接，软键盘在「无输入框」模式下只会吐数字键，
 * 中文候选词根本进不来。这里挂一个 1x1、透明的 EditText 当真正的输入法宿主，
 * Rust 侧通过 beginInput / inputText / endInput 读写它。
 */
public class MainBack extends NativeActivity {
    private static final String TAG = "cakecake-ime";
    private static final int REQ_PICK_IMAGE = 2001;
    private static final int REQ_PICK_VIDEO = 2002;
    private EditText input;
    /** 键盘右下角「发送」键按过一次（Rust 侧 takeInputAction 取走） */
    private volatile boolean inputAction = false;
    /** 相册选择结果：null=还没选，""=取消了，其它=缓存文件绝对路径 */
    private static String pickedPath = null;
    /** 视频选择结果（投稿用），语义同上 */
    private static String pickedVideoPath = null;

    @Override
    public void onBackPressed() {
        nativeBack();
    }

    private static native void nativeBack();

    private EditText ensureInput() {
        if (input != null) {
            return input;
        }
        EditText e = new EditText(this);
        e.setInputType(InputType.TYPE_CLASS_TEXT);
        e.setSingleLine(true);
        // 聊天/评论这类输入把键盘右下角变成「发送」，按下就交给 Rust 侧发送；
        // NO_EXTRACT_UI 是横屏的关键：否则海拔低的横屏窗口会被输入法接管成
        // 全屏编辑页，而我们这块 1x1 的隐藏输入框在那一页上什么都看不见。
        e.setImeOptions(EditorInfo.IME_ACTION_SEND | EditorInfo.IME_FLAG_NO_EXTRACT_UI);
        e.setOnEditorActionListener(new TextView.OnEditorActionListener() {
            @Override
            public boolean onEditorAction(TextView v, int actionId, KeyEvent event) {
                boolean enter = event != null
                        && event.getKeyCode() == KeyEvent.KEYCODE_ENTER
                        && event.getAction() == KeyEvent.ACTION_DOWN;
                if (actionId == EditorInfo.IME_ACTION_SEND
                        || actionId == EditorInfo.IME_ACTION_DONE
                        || enter) {
                    inputAction = true;
                    return true;
                }
                return false;
            }
        });
        e.setBackgroundColor(Color.TRANSPARENT);
        e.setTextColor(Color.TRANSPARENT);
        e.setCursorVisible(false);
        e.setAlpha(0.0f);
        e.setFocusable(true);
        e.setFocusableInTouchMode(true);
        ViewGroup.LayoutParams lp = new ViewGroup.LayoutParams(1, 1);
        try {
            // 直接挂到 DecorView：NativeActivity 的 addContentView 在部分 ROM 上不可用
            ViewGroup decor = (ViewGroup) getWindow().getDecorView();
            decor.addView(e, lp);
            input = e;
        } catch (Throwable t) {
            Log.e(TAG, "addView failed", t);
            return null;
        }
        return input;
    }

    /** 拉起原生键盘，把已有草稿写进去（必须切回 UI 线程操作视图） */
    public void beginInput(final String text) {
        beginInputInternal(text, false);
    }

    /** 多行输入（文章正文这类）：回车换行，键盘右下角不再是「发送」 */
    public void beginInputMulti(final String text) {
        beginInputInternal(text, true);
    }

    private void beginInputInternal(final String text, final boolean multi) {
        inputAction = false;
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                final EditText e = ensureInput();
                if (e == null) {
                    return;
                }
                try {
                    // 同一个宿主 EditText 在单行/多行之间切换，切换后要重设
                    // inputType 才会生效（否则多行框里回车仍然是「发送」）
                    e.setSingleLine(!multi);
                    e.setInputType(multi
                            ? (InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_MULTI_LINE)
                            : InputType.TYPE_CLASS_TEXT);
                    e.setImeOptions(multi
                            ? (EditorInfo.IME_ACTION_NONE | EditorInfo.IME_FLAG_NO_EXTRACT_UI)
                            : (EditorInfo.IME_ACTION_SEND | EditorInfo.IME_FLAG_NO_EXTRACT_UI));
                    e.setText(text);
                    e.setSelection(e.getText().length());
                    e.requestFocus();
                    // addView 之后还没 layout，立刻 showSoftInput 会返回 false；
                    // 丢到消息队列，等这一帧布局完成再把键盘绑到 EditText 上
                    e.post(new Runnable() {
                        @Override
                        public void run() {
                            try {
                                e.requestFocus();
                                InputMethodManager imm =
                                        (InputMethodManager) getSystemService(
                                                Context.INPUT_METHOD_SERVICE);
                                // SHOW_IMPLICIT 在横屏全屏下会返回 true 但键盘不出现
                                // （dumpsys 里 mInputShown=true 却看不见），必须用
                                // SHOW_FORCED 直连输入法；再不行就 toggle 兜底。
                                boolean ok = imm.showSoftInput(e, InputMethodManager.SHOW_FORCED);
                                if (!ok) {
                                    Log.w(TAG, "showSoftInput(SHOW_FORCED) false, toggling");
                                    imm.toggleSoftInput(InputMethodManager.SHOW_FORCED, 0);
                                }
                            } catch (Throwable t) {
                                Log.w(TAG, "showSoftInput failed", t);
                            }
                        }
                    });
                } catch (Throwable t) {
                    Log.e(TAG, "beginInput failed", t);
                }
            }
        });
    }

    public void endInput() {
        inputAction = false;
        final EditText e = input;
        if (e == null) {
            return;
        }
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                try {
                    InputMethodManager imm =
                            (InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE);
                    imm.hideSoftInputFromWindow(e.getWindowToken(), 0);
                    e.clearFocus();
                } catch (Throwable t) {
                    Log.w(TAG, "endInput failed", t);
                }
            }
        });
    }

    public String inputText() {
        if (input == null || input.getText() == null) {
            return "";
        }
        return input.getText().toString();
    }

    /** 取一次「键盘发送键」标记（取完清零） */
    public boolean takeInputAction() {
        boolean v = inputAction;
        inputAction = false;
        return v;
    }

    public void clearInput() {
        final EditText e = input;
        if (e == null) {
            return;
        }
        inputAction = false;
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                e.setText("");
            }
        });
    }

    /** 打开系统相册选一张图（选完拷到 cacheDir，Rust 侧直接按路径上传） */
    public void pickImage() {
        pickedPath = null;
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                try {
                    Intent i = new Intent(Intent.ACTION_OPEN_DOCUMENT);
                    i.addCategory(Intent.CATEGORY_OPENABLE);
                    i.setType("image/*");
                    startActivityForResult(i, REQ_PICK_IMAGE);
                } catch (Throwable t) {
                    try {
                        Intent p = new Intent(Intent.ACTION_PICK);
                        p.setType("image/*");
                        startActivityForResult(p, REQ_PICK_IMAGE);
                    } catch (Throwable t2) {
                        Log.e(TAG, "pickImage failed", t2);
                        pickedPath = "";
                    }
                }
            }
        });
    }

    /** 投稿页选中的视频：时长(ms)、宽、高，返回 "dur,w,h"；取不到返回 "" */
    public String videoMeta(String path) {
        MediaMetadataRetriever r = new MediaMetadataRetriever();
        try {
            r.setDataSource(path);
            String dur = r.extractMetadata(MediaMetadataRetriever.METADATA_KEY_DURATION);
            String w = r.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_WIDTH);
            String h = r.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_HEIGHT);
            return (dur == null ? "0" : dur) + "," + (w == null ? "0" : w) + "," + (h == null ? "0" : h);
        } catch (Throwable t) {
            Log.w(TAG, "videoMeta failed", t);
            return "";
        } finally {
            try { r.release(); } catch (Throwable ignored) {}
        }
    }

    /** 取首帧缩略图（RGBA8888，固定 w x h），取不到返回 null */
    public byte[] videoFrameRgba(String path, int w, int h) {
        MediaMetadataRetriever r = new MediaMetadataRetriever();
        try {
            r.setDataSource(path);
            Bitmap raw = (Build.VERSION.SDK_INT >= 27)
                    ? r.getScaledFrameAtTime(0, MediaMetadataRetriever.OPTION_CLOSEST_SYNC,
                            w * 2, h * 2)
                    : r.getFrameAtTime(0);
            if (raw == null) {
                return null;
            }
            Bitmap bmp = Bitmap.createScaledBitmap(raw, w, h, true);
            int[] px = new int[w * h];
            bmp.getPixels(px, 0, w, 0, 0, w, h);
            byte[] out = new byte[w * h * 4];
            for (int i = 0; i < px.length; i++) {
                int c = px[i];
                out[i * 4] = (byte) ((c >> 16) & 0xff);
                out[i * 4 + 1] = (byte) ((c >> 8) & 0xff);
                out[i * 4 + 2] = (byte) (c & 0xff);
                out[i * 4 + 3] = (byte) ((c >>> 24) & 0xff);
            }
            if (bmp != raw) {
                bmp.recycle();
            }
            raw.recycle();
            return out;
        } catch (Throwable t) {
            Log.w(TAG, "videoFrame failed", t);
            return null;
        } finally {
            try { r.release(); } catch (Throwable ignored) {}
        }
    }

    /** 打开系统文件选择器挑一个视频（投稿用） */
    public void pickVideo() {
        pickedVideoPath = null;
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                try {
                    Intent i = new Intent(Intent.ACTION_OPEN_DOCUMENT);
                    i.addCategory(Intent.CATEGORY_OPENABLE);
                    i.setType("video/*");
                    startActivityForResult(i, REQ_PICK_VIDEO);
                } catch (Throwable t) {
                    try {
                        Intent p = new Intent(Intent.ACTION_GET_CONTENT);
                        p.setType("video/*");
                        startActivityForResult(p, REQ_PICK_VIDEO);
                    } catch (Throwable t2) {
                        Log.e(TAG, "pickVideo failed", t2);
                        pickedVideoPath = "";
                    }
                }
            }
        });
    }

    /** 取一次视频选择结果：null=还没选，""=取消/失败，其它=缓存文件路径 */
    public String takePickedVideo() {
        String p = pickedVideoPath;
        pickedVideoPath = null;
        return p;
    }

    /** 取一次选择结果：null=还没选，""=取消/失败，其它=缓存文件路径 */
    public String takePickedImage() {
        String p = pickedPath;
        pickedPath = null;
        return p;
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        boolean video = requestCode == REQ_PICK_VIDEO;
        if (!video && requestCode != REQ_PICK_IMAGE) {
            return;
        }
        if (video) {
            pickedVideoPath = "";
        } else {
            pickedPath = "";
        }
        if (resultCode != RESULT_OK || data == null || data.getData() == null) {
            return;
        }
        String path = copyToCache(data.getData(), video ? "video" : "comment",
                video ? ".mp4" : ".jpg");
        if (video) {
            pickedVideoPath = path;
            Log.e(TAG, "picked video -> " + path);
        } else {
            pickedPath = path;
            Log.e(TAG, "picked image -> " + path);
        }
    }

    /** 把选中的 Uri 拷进 cacheDir，返回绝对路径（失败返回 ""） */
    private String copyToCache(Uri uri, String prefix, String ext) {
        try {
            InputStream in = getContentResolver().openInputStream(uri);
            if (in == null) {
                return "";
            }
            File dir = new File(getCacheDir(), "picked");
            if (!dir.exists() && !dir.mkdirs()) {
                in.close();
                return "";
            }
            File out = new File(dir, prefix + "_" + System.currentTimeMillis() + ext);
            FileOutputStream fos = new FileOutputStream(out);
            byte[] buf = new byte[8192];
            int n;
            while ((n = in.read(buf)) != -1) {
                fos.write(buf, 0, n);
            }
            fos.close();
            in.close();
            return out.getAbsolutePath();
        } catch (Throwable t) {
            Log.e(TAG, "copy picked file failed", t);
            return "";
        }
    }
}
