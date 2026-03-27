// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! Cloudflare quick tunnel — spawns `cloudflared` as a child process
//! to expose the local server to the internet without configuration.

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// f20=start — spawn cloudflared and print the public URL
pub async fn f20(port: u16) {
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

    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(pos) = line.find("https://") {
                let url = &line[pos..];
                let url = url.split_whitespace().next().unwrap_or(url);
                let pad = url.len() + 4;
                eprintln!();
                eprintln!("  ┌{}┐", "─".repeat(pad));
                eprintln!("  │  {}  │", url);
                eprintln!("  └{}┘", "─".repeat(pad));
                eprintln!();
                while let Ok(Some(_)) = lines.next_line().await {}
                break;
            }
        }
    }

    let _ = child.wait().await;
}
