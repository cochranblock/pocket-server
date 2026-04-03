// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! iOS entry point — called from Swift via @_silgen_name FFI.
//! Starts the server on the calling thread (Swift dispatches to background).

use std::ffi::CStr;
use std::path::PathBuf;

/// Called from Swift: pocket_server_ios_main(port, site_dir_cstr)
#[unsafe(no_mangle)]
pub extern "C" fn pocket_server_ios_main(port: u16, site_dir: *const std::ffi::c_char) {
    let s3 = if site_dir.is_null() {
        None
    } else {
        let cstr = unsafe { CStr::from_ptr(site_dir) };
        let path = cstr.to_string_lossy().to_string();
        if path.is_empty() { None } else { Some(PathBuf::from(path)) }
    };

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        crate::server::f9("Pocket Server".into(), "pocket-server".into(), port, s3, true, 50 * 1024 * 1024).await;
    });
}
