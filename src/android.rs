// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! JNI bridge — Android calls into Rust to start/stop the server
//! and poll stats for the dashboard Activity.
//! JNI symbols cannot be renamed (Java naming convention).

use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::{EnvUnowned, Outcome};
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

use crate::stats::t1;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();
static STATS: OnceLock<std::sync::Arc<t1>> = OnceLock::new();
static SHUTDOWN: OnceLock<std::sync::Arc<tokio::sync::Notify>> = OnceLock::new();

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
#[allow(deprecated)]
pub extern "system" fn Java_org_cochranblock_pocketserver_PocketServer_startServer<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    site_name: JString<'local>,
    port: jni::sys::jint,
    site_dir: JString<'local>,
) {
    let _: jni::EnvOutcome<'_, (), jni::errors::Error> = unowned_env.with_env(|env| {
        let s1: String = env.get_string(&site_name)?.into();
        let port = port as u16;
        let dir_str: String = env.get_string(&site_dir)?.into();
        let s3 = if dir_str.is_empty() {
            None
        } else {
            Some(PathBuf::from(dir_str))
        };

        let s0 = std::sync::Arc::new(t1::f10());
        let _ = STATS.set(s0.clone());

        let shutdown = std::sync::Arc::new(tokio::sync::Notify::new());
        let _ = SHUTDOWN.set(shutdown.clone());

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
            .with_graceful_shutdown(async move { shutdown.notified().await })
            .await
            .unwrap();
        });
        Ok(())
    });
}

/// Called from Java: PocketServer.stopServer()
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_cochranblock_pocketserver_PocketServer_stopServer(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
) {
    if let Some(shutdown) = SHUTDOWN.get() {
        shutdown.notify_one();
    }
}

/// Called from Java: PocketServer.getStats() -> JSON string
#[unsafe(no_mangle)]
#[allow(deprecated)]
pub extern "system" fn Java_org_cochranblock_pocketserver_PocketServer_getStats<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> jstring {
    let json = if let Some(s0) = STATS.get() {
        s0.f19()
    } else {
        r#"{"uptime":"0h 0m","requests":0,"bytes_served":"0 B","power_w":0.0,"monthly_cost":"$0.00"}"#.to_string()
    };
    let outcome: jni::EnvOutcome<'_, jstring, jni::errors::Error> = unowned_env.with_env(|env| {
        Ok(env.new_string(&json)?.into_raw())
    });
    let outcome = outcome.into_outcome();
    match outcome {
        Outcome::Ok(s) => s,
        _ => std::ptr::null_mut(),
    }
}
