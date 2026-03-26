// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

package org.cochranblock.pocketserver;

/**
 * JNI bridge to the Rust server. Loads libpocket_server.so
 * and exposes start/stats to the dashboard Activity.
 */
public class PocketServer {
    static {
        System.loadLibrary("pocket_server");
    }

    public static native void startServer(String siteName, int port);
    public static native String getStats();
}
