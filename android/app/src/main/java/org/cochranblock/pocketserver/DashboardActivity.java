// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

package org.cochranblock.pocketserver;

import android.app.Activity;
import android.content.Intent;
import android.os.Build;
import android.os.Bundle;
import android.view.View;
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
    private WebView webView;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        // Fullscreen kiosk mode
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        getWindow().getDecorView().setSystemUiVisibility(
            View.SYSTEM_UI_FLAG_FULLSCREEN
            | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
            | View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
        );

        // Start the foreground service
        Intent svc = new Intent(this, ServerService.class);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(svc);
        } else {
            startService(svc);
        }

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
    public void onBackPressed() {
        // Kiosk mode — don't leave
    }
}
