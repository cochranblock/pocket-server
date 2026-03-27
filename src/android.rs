// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! JNI bridge — Android calls into Rust to start/stop the server
//! and poll stats for the dashboard Activity.
//! JNI symbols cannot be renamed (Java naming convention).

use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

use crate::stats::t1;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();
static STATS: OnceLock<std::sync::Arc<t1>> = OnceLock::new();

/// f22=get_runtime
fn f22() -> &'static Runtime {
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
    let s1: String = env.get_string(&site_name).unwrap().into();
    let port = port as u16;
    let dir_str: String = env.get_string(&site_dir).unwrap().into();
    let s3 = if dir_str.is_empty() {
        None
    } else {
        Some(PathBuf::from(dir_str))
    };

    let s0 = std::sync::Arc::new(t1::f10());
    let _ = STATS.set(s0.clone());

    let rt = f22();
    rt.spawn(async move {
        let state = crate::server::t0 {
            s0,
            s1,
            s2: "pocket-server".into(),
            s3,
        };
        let app = crate::server::f8(state);
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
    let json = if let Some(s0) = STATS.get() {
        s0.f19()
    } else {
        r#"{"uptime":"0h 0m","requests":0,"bytes_served":"0 B","power_w":0.0,"monthly_cost":"$0.00"}"#.to_string()
    };
    env.new_string(&json).unwrap().into_raw()
}
