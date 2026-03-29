// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

// Thin Swift bridge — calls Rust entry point on a background thread.
// The Rust server handles everything: HTTP, dashboard, stats, upload.

import UIKit
import WebKit

// Rust FFI — linked from libpocket_server.a (staticlib)
@_silgen_name("pocket_server_ios_main")
func pocket_server_ios_main(_ port: UInt16, _ site_dir: UnsafePointer<CChar>?)

@main
class AppDelegate: UIResponder, UIApplicationDelegate {
    var window: UIWindow?

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        let port: UInt16 = 8080

        // Site directory: Documents/site/
        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let siteDir = docs.appendingPathComponent("site")
        try? FileManager.default.createDirectory(at: siteDir, withIntermediateDirectories: true)

        let sitePath = siteDir.path

        // Start Rust server on background thread
        DispatchQueue.global(qos: .userInitiated).async {
            sitePath.withCString { ptr in
                pocket_server_ios_main(port, ptr)
            }
        }

        // WebView dashboard
        window = UIWindow(frame: UIScreen.main.bounds)
        let wv = WKWebView(frame: window!.bounds)
        wv.backgroundColor = UIColor(red: 0.04, green: 0.04, blue: 0.04, alpha: 1)
        wv.isOpaque = false
        wv.scrollView.bounces = false

        let vc = UIViewController()
        vc.view = wv
        window?.rootViewController = vc
        window?.makeKeyAndVisible()

        // Load dashboard after server binds
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            let url = URL(string: "http://127.0.0.1:\(port)/dashboard")!
            wv.load(URLRequest(url: url))
        }

        return true
    }
}
