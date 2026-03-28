// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! CLI entry point — run the server on any machine for dev/demo.
//! On Android this binary isn't used; the JNI bridge starts the server instead.

use std::path::PathBuf;

/// f21=parse_args
fn f21() -> (String, u16, Option<PathBuf>, bool) {
    let mut name = "Pocket Server".to_string();
    let mut port: u16 = 8080;
    let mut site_dir: Option<PathBuf> = None;
    let mut tunnel = false;

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--name" | "-n" => {
                i += 1;
                if i < args.len() { name = args[i].clone(); }
            }
            "--port" | "-p" => {
                i += 1;
                if i < args.len() { port = args[i].parse().unwrap_or(8080); }
            }
            "--site-dir" | "-d" => {
                i += 1;
                if i < args.len() { site_dir = Some(PathBuf::from(&args[i])); }
            }
            "--tunnel" | "-t" => {
                tunnel = true;
            }
            "--sbom" => {
                print!("{}", pocket_server::govdocs::generate_spdx());
                std::process::exit(0);
            }
            "--help" | "-h" => {
                eprintln!("pocket-server — your website lives on your phone");
                eprintln!();
                eprintln!("  --name, -n <name>      Site name (default: Pocket Server)");
                eprintln!("  --port, -p <port>      Port to bind (default: 8080)");
                eprintln!("  --site-dir, -d <path>  Directory with site files to serve");
                eprintln!("  --tunnel, -t           Start Cloudflare quick tunnel");
                eprintln!("  --sbom                 Print SPDX SBOM and exit");
                eprintln!("  --help, -h             This message");
                std::process::exit(0);
            }
            _ => {
                eprintln!("unknown arg: {}", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }
    (name, port, site_dir, tunnel)
}

#[tokio::main]
async fn main() {
    let (name, port, site_dir, tunnel) = f21();

    let dir_label = site_dir
        .as_ref()
        .map(|d| d.display().to_string())
        .unwrap_or_else(|| "(default landing page)".into());

    // Validate site dir if specified
    if let Some(ref d) = site_dir
        && !d.exists()
    {
        eprintln!("warning: site-dir does not exist: {}", d.display());
        eprintln!("  creating it now...");
        if let Err(e) = std::fs::create_dir_all(d) {
            eprintln!("error: cannot create site-dir: {}", e);
            std::process::exit(1);
        }
    }

    eprintln!("pocket-server v{}", env!("CARGO_PKG_VERSION"));
    eprintln!("  name:     {}", name);
    eprintln!("  port:     {}", port);
    eprintln!("  site-dir: {}", dir_label);
    eprintln!("  site:      http://127.0.0.1:{}/", port);
    eprintln!("  dashboard: http://127.0.0.1:{}/dashboard", port);
    eprintln!();

    if tunnel {
        tokio::spawn(pocket_server::tunnel::f20(port));
    }

    pocket_server::server::f9(name, "pocket-server".into(), port, site_dir).await;
}
