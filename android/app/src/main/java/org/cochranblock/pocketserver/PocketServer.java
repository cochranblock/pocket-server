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

    /**
     * Start the server.
     * @param siteName display name for the site
     * @param port     TCP port to bind
     * @param siteDir  path to site files on storage, or "" for default landing page
     */
    public static native void startServer(String siteName, int port, String siteDir);
    public static native String getStats();
}
