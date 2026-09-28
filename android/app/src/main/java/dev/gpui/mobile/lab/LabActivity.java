package dev.gpui.mobile.lab;

import android.app.Activity;
import android.graphics.Rect;
import android.os.Build;
import android.os.Bundle;
import android.text.Editable;
import android.text.InputType;
import android.text.Selection;
import android.text.TextWatcher;
import android.util.Log;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.view.ViewGroup;
import android.view.inputmethod.BaseInputConnection;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputConnection;
import android.view.inputmethod.InputConnectionWrapper;
import android.view.inputmethod.InputMethodManager;
import android.widget.EditText;

/**
 * Host for the GPUI render thread (gpui_mobile::android::host).
 *
 * A plain Activity owning a SurfaceView: the Rust side keeps one process-lived
 * render thread and GPUI App, so recreating this Activity only swaps the surface.
 * The window uses adjustResize, so the SurfaceView (and GPUI's viewport) shrinks
 * above the software keyboard.
 *
 * The IME proxy is adapted from gpui-mobile's GpuiInputActivity: an invisible
 * EditText whose InputConnection forwards composing/commit/delete to Rust.
 */
public class LabActivity extends Activity implements SurfaceHolder.Callback {
    private static final String TAG = "GPUI_MOBILE_LAB";

    static {
        System.loadLibrary("gpui_mobile_lab");
    }

    private SurfaceView surface;
    private InputProxy input;
    private boolean keyboardVisible;

    @Override
    protected void onCreate(Bundle state) {
        super.onCreate(state);
        surface = new SurfaceView(this);
        surface.getHolder().addCallback(this);
        surface.setFocusable(true);
        surface.setFocusableInTouchMode(true);
        surface.setOnTouchListener((view, event) -> forwardMotion(event));
        setContentView(surface);

        // The keyboard shrinks the content via adjustResize; report its height so the
        // diagnostics screen can show it next to GPUI's own viewport.
        final View root = getWindow().getDecorView();
        root.getViewTreeObserver().addOnGlobalLayoutListener(() -> {
            Rect visible = new Rect();
            root.getWindowVisibleDisplayFrame(visible);
            int hidden = root.getHeight() - visible.bottom;
            boolean shown = hidden > root.getHeight() / 5;
            if (shown != keyboardVisible) {
                keyboardVisible = shown;
                Log.i(TAG, "software keyboard " + (shown ? "shown" : "hidden") + " (" + hidden + " px)");
            }
            nativeKeyboardInsets(shown, shown ? hidden : 0);
        });

        nativeOnCreate(this, Build.VERSION.SDK_INT);
    }

    @Override protected void onResume() { super.onResume(); nativeResumed(); }
    @Override protected void onPause() { nativePaused(); super.onPause(); }

    // ── SurfaceHolder.Callback ─────────────────────────────────────────────

    @Override public void surfaceCreated(SurfaceHolder holder) {
        nativeSurfaceChanged(holder.getSurface(), getResources().getDisplayMetrics().density);
    }

    @Override public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        nativeSurfaceChanged(holder.getSurface(), getResources().getDisplayMetrics().density);
    }

    @Override public void surfaceDestroyed(SurfaceHolder holder) {
        // Blocks until the render thread has let go of the surface.
        nativeSurfaceDestroyed();
    }

    // ── Input ──────────────────────────────────────────────────────────────

    private boolean forwardMotion(MotionEvent event) {
        int count = event.getPointerCount();
        int[] ids = new int[count];
        float[] xs = new float[count];
        float[] ys = new float[count];
        for (int i = 0; i < count; i++) {
            ids[i] = event.getPointerId(i);
            xs[i] = event.getX(i);
            ys[i] = event.getY(i);
        }
        return nativeMotion(event.getActionMasked(), event.getActionIndex(), ids, xs, ys);
    }

    @Override
    public boolean dispatchKeyEvent(KeyEvent event) {
        int code = event.getKeyCode();
        if (code == KeyEvent.KEYCODE_VOLUME_UP || code == KeyEvent.KEYCODE_VOLUME_DOWN
                || code == KeyEvent.KEYCODE_VOLUME_MUTE) {
            return super.dispatchKeyEvent(event);
        }
        // Back reaches GPUI as "escape": overlays close first, then the app pops its
        // screen stack, and at the catalog root it calls moveTaskToBack().
        if (code == KeyEvent.KEYCODE_BACK) {
            if (event.getAction() == KeyEvent.ACTION_DOWN || event.getAction() == KeyEvent.ACTION_UP) {
                nativeKey(code, event.getAction(), event.getMetaState());
            }
            return true;
        }
        // While the IME proxy is focused it turns printable keys into commits; only
        // navigation and shortcut keys go to GPUI directly.
        if (input != null && input.hasFocus() && !isNavigationOrShortcut(event)) {
            return super.dispatchKeyEvent(event);
        }
        if (event.getAction() == KeyEvent.ACTION_DOWN || event.getAction() == KeyEvent.ACTION_UP) {
            nativeKey(code, event.getAction(), event.getMetaState());
            return true;
        }
        return super.dispatchKeyEvent(event);
    }

    private static boolean isNavigationOrShortcut(KeyEvent event) {
        if (event.isCtrlPressed() || event.isAltPressed() || event.isMetaPressed()) return true;
        switch (event.getKeyCode()) {
            case KeyEvent.KEYCODE_DPAD_LEFT: case KeyEvent.KEYCODE_DPAD_RIGHT:
            case KeyEvent.KEYCODE_DPAD_UP: case KeyEvent.KEYCODE_DPAD_DOWN:
            case KeyEvent.KEYCODE_MOVE_HOME: case KeyEvent.KEYCODE_MOVE_END:
            case KeyEvent.KEYCODE_PAGE_UP: case KeyEvent.KEYCODE_PAGE_DOWN:
            case KeyEvent.KEYCODE_TAB: case KeyEvent.KEYCODE_ESCAPE:
            case KeyEvent.KEYCODE_FORWARD_DEL:
                return true;
            default:
                return false;
        }
    }

    // ── IME bridge (called from Rust on the render thread) ─────────────────

    public void gpuiShowKeyboard(int keyboardType, long session) {
        runOnUiThread(() -> {
            if (input == null) {
                input = new InputProxy();
                input.setAlpha(0f);
                input.setPadding(0, 0, 0, 0);
                addContentView(input, new ViewGroup.LayoutParams(1, 1));
            }
            input.reset(session);
            int type = InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_MULTI_LINE;
            switch (keyboardType) {
                case 1: type = InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_VARIATION_EMAIL_ADDRESS; break;
                case 2: type = InputType.TYPE_CLASS_PHONE; break;
                case 3: type = InputType.TYPE_CLASS_NUMBER; break;
                case 4: type = InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_VARIATION_URI; break;
                case 5: type = InputType.TYPE_CLASS_NUMBER | InputType.TYPE_NUMBER_FLAG_DECIMAL; break;
            }
            input.setInputType(type);
            input.setImeOptions(EditorInfo.IME_FLAG_NO_EXTRACT_UI);
            input.requestFocus();
            InputMethodManager imm = (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);
            imm.restartInput(input);
            imm.showSoftInput(input, InputMethodManager.SHOW_IMPLICIT);
        });
    }

    public void gpuiHideKeyboard(long session) {
        runOnUiThread(() -> {
            if (input == null) return;
            input.reset(session);
            InputMethodManager imm = (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);
            imm.hideSoftInputFromWindow(input.getWindowToken(), 0);
            input.clearFocus();
            surface.requestFocus();
        });
    }

    public void gpuiResetComposition(long session) {
        runOnUiThread(() -> {
            if (input == null) return;
            input.reset(session);
            ((InputMethodManager) getSystemService(INPUT_METHOD_SERVICE)).restartInput(input);
        });
    }

    private final class InputProxy extends EditText {
        private long session;
        private int depth;
        private boolean marked;

        InputProxy() {
            super(LabActivity.this);
            addTextChangedListener(new TextWatcher() {
                public void beforeTextChanged(CharSequence s, int start, int count, int after) {}
                public void onTextChanged(CharSequence s, int start, int before, int count) {}
                public void afterTextChanged(Editable text) {
                    // Hardware keyboards edit the widget directly, outside its
                    // InputConnection. IME mutations are batched by depth below.
                    if (depth == 0) { depth++; endEdit(); }
                }
            });
        }

        @Override public boolean onKeyDown(int code, KeyEvent event) {
            if (code == KeyEvent.KEYCODE_DEL && getText().length() == 0 && !marked) {
                nativeIme(session, 3, "", 1, 0);
                return true;
            }
            return super.onKeyDown(code, event);
        }

        void reset(long nextSession) {
            depth++;
            getText().clear();
            marked = false;
            session = nextSession;
            depth = 0;
        }

        private void endEdit() {
            if (--depth != 0) return;
            Editable text = getText();
            boolean composing = BaseInputConnection.getComposingSpanStart(text) >= 0;
            if (composing || marked || text.length() > 0) {
                nativeIme(session, composing ? 0 : 1, text.toString(),
                        Math.max(0, Selection.getSelectionStart(text)),
                        Math.max(0, Selection.getSelectionEnd(text)));
                marked = composing;
                if (!composing) {
                    depth++;
                    text.clear();
                    depth--;
                }
            }
        }

        @Override public boolean onKeyPreIme(int code, KeyEvent event) {
            if (code == KeyEvent.KEYCODE_BACK && event.getAction() == KeyEvent.ACTION_UP) {
                nativeIme(session, 4, "", 0, 0);
            }
            return super.onKeyPreIme(code, event);
        }

        @Override public InputConnection onCreateInputConnection(EditorInfo info) {
            InputConnection connection = super.onCreateInputConnection(info);
            if (connection == null) return null;
            final long connectionSession = session;
            return new InputConnectionWrapper(connection, false) {
                @Override public boolean beginBatchEdit() {
                    if (connectionSession != session) return false;
                    depth++;
                    return super.beginBatchEdit();
                }
                @Override public boolean endBatchEdit() {
                    if (connectionSession != session) return false;
                    boolean result = super.endBatchEdit();
                    if (depth > 0) endEdit();
                    return result;
                }
                @Override public boolean setComposingText(CharSequence text, int cursor) {
                    if (connectionSession != session) return false;
                    depth++;
                    try { return super.setComposingText(text, cursor); }
                    finally { endEdit(); }
                }
                @Override public boolean setComposingRegion(int start, int end) {
                    if (connectionSession != session) return false;
                    depth++;
                    try { return super.setComposingRegion(start, end); }
                    finally { endEdit(); }
                }
                @Override public boolean finishComposingText() {
                    if (connectionSession != session) return false;
                    depth++;
                    try { return super.finishComposingText(); }
                    finally { endEdit(); }
                }
                @Override public boolean commitText(CharSequence text, int cursor) {
                    if (connectionSession != session) return false;
                    depth++;
                    try { return super.commitText(text, cursor); }
                    finally { endEdit(); }
                }
                @Override public boolean setSelection(int start, int end) {
                    if (connectionSession != session) return false;
                    depth++;
                    try { return super.setSelection(start, end); }
                    finally { endEdit(); }
                }
                @Override public boolean deleteSurroundingText(int before, int after) {
                    if (connectionSession != session) return false;
                    if (getText().length() == 0 && !marked) {
                        nativeIme(session, 2, "", before, after);
                        return true;
                    }
                    depth++;
                    try { return super.deleteSurroundingText(before, after); }
                    finally { endEdit(); }
                }
                @Override public boolean deleteSurroundingTextInCodePoints(int before, int after) {
                    if (connectionSession != session) return false;
                    if (getText().length() == 0 && !marked) {
                        nativeIme(session, 3, "", before, after);
                        return true;
                    }
                    depth++;
                    try { return super.deleteSurroundingTextInCodePoints(before, after); }
                    finally { endEdit(); }
                }
                @Override public boolean sendKeyEvent(KeyEvent event) {
                    if (connectionSession != session) return false;
                    if (event.getKeyCode() == KeyEvent.KEYCODE_DEL) {
                        if (event.getAction() == KeyEvent.ACTION_DOWN) deleteSurroundingText(1, 0);
                        return true;
                    }
                    if (event.getKeyCode() == KeyEvent.KEYCODE_ENTER) {
                        if (event.getAction() == KeyEvent.ACTION_DOWN) commitText("\n", 1);
                        return true;
                    }
                    return super.sendKeyEvent(event);
                }
                @Override public boolean performEditorAction(int action) {
                    if (connectionSession != session) return false;
                    if (action == EditorInfo.IME_ACTION_DONE) {
                        finishComposingText();
                        nativeIme(session, 4, "", 0, 0);
                        return true;
                    }
                    return commitText("\n", 1);
                }
            };
        }
    }

    // ── Native (src/host.rs) ───────────────────────────────────────────────

    private static native void nativeOnCreate(Activity activity, int apiLevel);
    private static native void nativeSurfaceChanged(android.view.Surface surface, float scale);
    private static native void nativeSurfaceDestroyed();
    private static native void nativeResumed();
    private static native void nativePaused();
    private static native void nativeKeyboardInsets(boolean visible, int heightPx);
    private static native boolean nativeMotion(int action, int actionIndex, int[] ids, float[] xs, float[] ys);
    private static native void nativeKey(int keyCode, int action, int metaState);
    private static native void nativeIme(long session, int kind, String text, int start, int end);
}
