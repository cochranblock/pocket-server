// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

package org.cochranblock.pocketserver;

import android.Manifest;
import android.app.Activity;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.os.Build;
import android.os.Bundle;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.view.WindowManager;
import android.webkit.WebView;
import android.webkit.WebViewClient;

/**
 * Kiosk dashboard — fullscreen WebView pointed at the local Rust server.
 * Shows live stats (uptime, requests, bytes, power draw, monthly cost).
 * Starts ServerService on launch so the server outlives the Activity.
 */
public class DashboardActivity extends Activity {

    private static final int PORT = 8080;
    private static final int NOTIF_PERM_CODE = 1;
    private WebView webView;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        // Keep screen on
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);

        // Fullscreen immersive — API 30+ WindowInsetsController
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            getWindow().setDecorFitsSystemWindows(false);
            WindowInsetsController wic = getWindow().getInsetsController();
            if (wic != null) {
                wic.hide(WindowInsets.Type.systemBars());
                wic.setSystemBarsBehavior(
                    WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
                );
            }
        } else {
            getWindow().getDecorView().setSystemUiVisibility(
                android.view.View.SYSTEM_UI_FLAG_FULLSCREEN
                | android.view.View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                | android.view.View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
            );
        }

        // Request notification permission (API 33+)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            if (checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS)
                    != PackageManager.PERMISSION_GRANTED) {
                requestPermissions(
                    new String[]{Manifest.permission.POST_NOTIFICATIONS}, NOTIF_PERM_CODE
                );
            }
        }

        // Start the foreground service
        Intent svc = new Intent(this, ServerService.class);
        startForegroundService(svc);

        // WebView pointed at the dashboard
        webView = new WebView(this);
        webView.getSettings().setJavaScriptEnabled(true);
        webView.getSettings().setDomStorageEnabled(true);
        webView.setBackgroundColor(0xFF0A0A0A);
        webView.setWebViewClient(new WebViewClient());
        setContentView(webView);

        // Small delay so the server has time to bind
        webView.postDelayed(() -> {
            webView.loadUrl("http://127.0.0.1:" + PORT + "/dashboard");
        }, 500);
    }

    @Override
    protected void onResume() {
        super.onResume();
        // Re-apply immersive on resume
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            WindowInsetsController wic = getWindow().getInsetsController();
            if (wic != null) {
                wic.hide(WindowInsets.Type.systemBars());
            }
        }
    }
}
