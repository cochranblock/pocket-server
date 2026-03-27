// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! JNI bridge — Android calls into Rust to start/stop the server
//! and poll stats for the dashboard Activity.

use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

use crate::stats::Stats;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();
static STATS: OnceLock<std::sync::Arc<Stats>> = OnceLock::new();

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
    })
}

/// Called from Java: PocketServer.startServer(siteName, port, siteDir)
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_cochranblock_pocketserver_PocketServer_startServer(
    mut env: JNIEnv,
    _class: JClass,
    site_name: JString,
    port: jni::sys::jint,
    site_dir: JString,
) {
    let site_name: String = env.get_string(&site_name).unwrap().into();
    let port = port as u16;
    let dir_str: String = env.get_string(&site_dir).unwrap().into();
    let site_dir = if dir_str.is_empty() {
        None
    } else {
        Some(PathBuf::from(dir_str))
    };

    let stats = std::sync::Arc::new(Stats::new());
    let _ = STATS.set(stats.clone());

    let rt = get_runtime();
    rt.spawn(async move {
        let state = crate::server::AppState {
            stats,
            site_name,
            hostname: "pocket-server".into(),
            site_dir,
        };
        let app = crate::server::build_router(state);
        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
}

/// Called from Java: PocketServer.getStats() -> JSON string
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_cochranblock_pocketserver_PocketServer_getStats(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let json = if let Some(stats) = STATS.get() {
        stats.to_json()
    } else {
        r#"{"uptime":"0h 0m","requests":0,"bytes_served":"0 B","power_w":0.0,"monthly_cost":"$0.00"}"#.to_string()
    };
    env.new_string(&json).unwrap().into_raw()
}
