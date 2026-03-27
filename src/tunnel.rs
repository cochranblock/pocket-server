// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! Cloudflare quick tunnel — spawns `cloudflared` as a child process
//! to expose the local server to the internet without configuration.
//! `cloudflared tunnel --url http://localhost:PORT` gives a free
//! *.trycloudflare.com URL that routes to the phone.

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Spawn cloudflared and print the public URL when it appears.
pub async fn start(port: u16) {
    let url = format!("http://localhost:{}", port);

    let mut child = match Command::new("cloudflared")
        .args(["tunnel", "--url", &url])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[tunnel] failed to start cloudflared: {}", e);
            eprintln!("[tunnel] install: https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/");
            return;
        }
    };

    // cloudflared prints the URL to stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            // The public URL line looks like:
            // ... | https://foo-bar-baz.trycloudflare.com
            if let Some(pos) = line.find("https://") {
                let url = &line[pos..];
                // Trim trailing whitespace or pipe chars
                let url = url.split_whitespace().next().unwrap_or(url);
                let pad = url.len() + 4;
                eprintln!();
                eprintln!("  ┌{}┐", "─".repeat(pad));
                eprintln!("  │  {}  │", url);
                eprintln!("  └{}┘", "─".repeat(pad));
                eprintln!();
                // Keep reading to prevent buffer fill, but stop printing
                while let Ok(Some(_)) = lines.next_line().await {}
                break;
            }
        }
    }

    // Wait for the child so it doesn't become a zombie
    let _ = child.wait().await;
}
