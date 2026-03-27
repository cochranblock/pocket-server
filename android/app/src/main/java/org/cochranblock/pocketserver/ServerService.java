// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

package org.cochranblock.pocketserver;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.app.Service;
import android.content.Intent;
import android.os.Build;
import android.os.IBinder;
import android.os.PowerManager;

/**
 * Foreground service that keeps the Rust web server alive.
 * Acquires a partial wake lock so the CPU doesn't sleep while serving.
 * Notification shows "Pocket Server — serving".
 */
public class ServerService extends Service {

    private static final int NOTIFICATION_ID = 1;
    private static final String CHANNEL_ID = "pocket_server";
    private static final int PORT = 8080;
    private static final String SITE_NAME = "Pocket Server";

    private PowerManager.WakeLock wakeLock;
    private boolean serverStarted = false;

    @Override
    public void onCreate() {
        super.onCreate();
        createNotificationChannel();
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        startForeground(NOTIFICATION_ID, buildNotification());

        if (!serverStarted) {
            serverStarted = true;

            // Wake lock — keep CPU alive
            PowerManager pm = (PowerManager) getSystemService(POWER_SERVICE);
            wakeLock = pm.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "PocketServer::Server");
            wakeLock.acquire();

            // Site directory: /sdcard/pocket-server/site/
            // Empty string = default landing page
            String siteDir = getExternalFilesDir(null) + "/site";
            java.io.File dir = new java.io.File(siteDir);
            if (!dir.exists() || !dir.isDirectory() || dir.list().length == 0) {
                siteDir = "";
            }

            PocketServer.startServer(SITE_NAME, PORT, siteDir);
        }

        return START_STICKY;
    }

    @Override
    public void onDestroy() {
        if (wakeLock != null && wakeLock.isHeld()) {
            wakeLock.release();
        }
        super.onDestroy();
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }

    private void createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            NotificationChannel channel = new NotificationChannel(
                CHANNEL_ID, "Pocket Server", NotificationManager.IMPORTANCE_LOW
            );
            channel.setDescription("Server status");
            NotificationManager nm = getSystemService(NotificationManager.class);
            nm.createNotificationChannel(channel);
        }
    }

    private Notification buildNotification() {
        Intent tapIntent = new Intent(this, DashboardActivity.class);
        PendingIntent pending = PendingIntent.getActivity(
            this, 0, tapIntent, PendingIntent.FLAG_IMMUTABLE
        );

        Notification.Builder builder;
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            builder = new Notification.Builder(this, CHANNEL_ID);
        } else {
            builder = new Notification.Builder(this);
        }

        return builder
            .setContentTitle("Pocket Server")
            .setContentText("Serving on port " + PORT)
            .setSmallIcon(android.R.drawable.ic_menu_compass)
            .setContentIntent(pending)
            .setOngoing(true)
            .build();
    }
}
