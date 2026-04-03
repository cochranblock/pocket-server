// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! Govdocs — compliance docs baked into the binary.
//! Routes: /govdocs, /govdocs/sbom, /govdocs/capability, /govdocs/security
//! The binary IS the compliance artifact.

use axum::response::{Html, IntoResponse};

const CAPABILITY_MD: &str = include_str!("../govdocs/capability.md");
const SECURITY_MD: &str = include_str!("../govdocs/security.md");
const CARGO_TOML: &str = include_str!("../Cargo.toml");
const CARGO_LOCK: &str = include_str!("../Cargo.lock");

const CRT_STYLE: &str = r#"
*{margin:0;padding:0;box-sizing:border-box}
body{background:#0a0a0a;color:#e0e0e0;font-family:'SF Mono',ui-monospace,monospace;padding:2rem;max-width:80ch;margin:0 auto;line-height:1.6}
h1{color:#00d4aa;font-size:1.5rem;margin-bottom:1rem;border-bottom:1px solid #222;padding-bottom:0.5rem}
h2{color:#00d4aa;font-size:1.1rem;margin-top:1.5rem;margin-bottom:0.5rem}
h3{color:#888;font-size:0.95rem;margin-top:1rem;margin-bottom:0.3rem}
a{color:#00d4aa;text-decoration:none}
a:hover{text-decoration:underline}
p{margin-bottom:0.5rem}
table{border-collapse:collapse;width:100%;margin:0.5rem 0}
th,td{border:1px solid #333;padding:0.4rem 0.8rem;text-align:left;font-size:0.85rem}
th{background:#111;color:#00d4aa}
code{background:#111;padding:0.1rem 0.3rem;border-radius:2px;font-size:0.85rem}
pre{background:#111;padding:1rem;border-radius:4px;overflow-x:auto;margin:0.5rem 0;font-size:0.8rem}
ul,ol{padding-left:1.5rem;margin-bottom:0.5rem}
li{margin-bottom:0.2rem}
.nav{margin-bottom:2rem;font-size:0.85rem}
.nav a{margin-right:1.5rem}
"#;

fn nav() -> &'static str {
    r#"<div class="nav"><a href="/govdocs">Index</a><a href="/govdocs/sbom">SBOM</a><a href="/govdocs/capability">Capability</a><a href="/govdocs/security">Security</a></div>"#
}

/// Minimal markdown-to-HTML for headings, tables, lists, paragraphs.
/// Not a full parser — handles the subset used in govdocs files.
fn md_to_html(md: &str) -> String {
    let mut out = String::with_capacity(md.len() * 2);
    let mut in_table = false;
    let mut in_list = false;
    let mut in_pre = false;

    for line in md.lines() {
        // Code blocks
        if line.starts_with("```") {
            if in_pre {
                out.push_str("</pre>\n");
                in_pre = false;
            } else {
                out.push_str("<pre>");
                in_pre = true;
            }
            continue;
        }
        if in_pre {
            out.push_str(&line.replace('<', "&lt;").replace('>', "&gt;"));
            out.push('\n');
            continue;
        }

        let trimmed = line.trim();

        // Close table if we leave table rows
        if in_table && !trimmed.starts_with('|') {
            out.push_str("</table>\n");
            in_table = false;
        }

        // Close list if we leave list items
        if in_list && !trimmed.starts_with("- ") && !trimmed.starts_with("* ") {
            out.push_str("</ul>\n");
            in_list = false;
        }

        if trimmed.is_empty() {
            continue;
        } else if let Some(h) = trimmed.strip_prefix("### ") {
            out.push_str(&format!("<h3>{}</h3>\n", h));
        } else if let Some(h) = trimmed.strip_prefix("## ") {
            out.push_str(&format!("<h2>{}</h2>\n", h));
        } else if let Some(h) = trimmed.strip_prefix("# ") {
            out.push_str(&format!("<h1>{}</h1>\n", h));
        } else if trimmed.starts_with('|') {
            // Table row
            if !in_table {
                out.push_str("<table>\n");
                in_table = true;
            }
            // Skip separator rows (|---|---|)
            if trimmed.contains("---") {
                continue;
            }
            let cells: Vec<&str> = trimmed
                .split('|')
                .filter(|c| !c.is_empty())
                .map(|c| c.trim())
                .collect();
            let tag = if !out.contains("<tr>") || out.ends_with("<table>\n") {
                "th"
            } else {
                "td"
            };
            out.push_str("<tr>");
            for cell in &cells {
                out.push_str(&format!("<{tag}>{cell}</{tag}>"));
            }
            out.push_str("</tr>\n");
        } else if let Some(item) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            if !in_list {
                out.push_str("<ul>\n");
                in_list = true;
            }
            // Bold handling
            let item = item.replace("**", "");
            out.push_str(&format!("<li>{}</li>\n", item));
        } else {
            out.push_str(&format!("<p>{}</p>\n", trimmed));
        }
    }
    if in_table { out.push_str("</table>\n"); }
    if in_list { out.push_str("</ul>\n"); }
    if in_pre { out.push_str("</pre>\n"); }
    out
}

/// Parse Cargo.lock to extract package names and versions.
fn parse_lock_packages() -> Vec<(String, String)> {
    let mut pkgs = Vec::new();
    let mut name = String::new();
    let mut version = String::new();

    for line in CARGO_LOCK.lines() {
        if line == "[[package]]" {
            if !name.is_empty() && name != "pocket-server" {
                pkgs.push((name.clone(), version.clone()));
            }
            name.clear();
            version.clear();
        } else if let Some(n) = line.strip_prefix("name = \"") {
            name = n.trim_end_matches('"').to_string();
        } else if let Some(v) = line.strip_prefix("version = \"") {
            version = v.trim_end_matches('"').to_string();
        }
    }
    if !name.is_empty() && name != "pocket-server" {
        pkgs.push((name, version));
    }
    pkgs
}

/// f23=govdocs_index — GET /govdocs
pub async fn f23() -> impl IntoResponse {
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Govdocs — Pocket Server</title><style>{style}</style></head><body>
{nav}
<h1>Compliance Documents — Pocket Server</h1>
<p>pocket-server v{ver}</p>
<p>The binary IS the compliance artifact. These docs are baked into the executable at compile time.</p>
<h2>Documents</h2>
<ul>
<li><a href="/govdocs/sbom">Software Bill of Materials</a> — live dependency list from Cargo.lock</li>
<li><a href="/govdocs/capability">Capability Statement</a> — what this product does, vendor info, NAICS</li>
<li><a href="/govdocs/security">Security Posture</a> — architecture, access controls, supply chain</li>
</ul>
<h2>Machine-Readable</h2>
<p>SPDX SBOM: <code>pocket-server --sbom</code> or <code>curl {host}/govdocs/sbom?format=spdx</code></p>
</body></html>"#,
        style = CRT_STYLE,
        nav = nav(),
        ver = env!("CARGO_PKG_VERSION"),
        host = "http://localhost:8080",
    );
    Html(html)
}

/// f24=govdocs_sbom — GET /govdocs/sbom
pub async fn f24(req: axum::extract::Request) -> impl IntoResponse {
    // Check for ?format=spdx query param
    if let Some(query) = req.uri().query()
        && query.contains("format=spdx")
    {
        return (
            [("content-type", "text/plain")],
            generate_spdx(),
        ).into_response();
    }

    let pkgs = parse_lock_packages();
    let mut table = String::from("<table><tr><th>#</th><th>Package</th><th>Version</th></tr>\n");
    for (i, (name, ver)) in pkgs.iter().enumerate() {
        table.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            i + 1, name, ver
        ));
    }
    table.push_str("</table>");

    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>SBOM — Pocket Server</title><style>{style}</style></head><body>
{nav}
<h1>Software Bill of Materials</h1>
<p>pocket-server v{ver} — generated live from baked Cargo.lock</p>
<p>Direct deps: axum, tokio, tower-http. Total transitive: {count}.</p>
<p>Machine-readable: <a href="/govdocs/sbom?format=spdx">SPDX format</a></p>
<h2>All Packages</h2>
{table}
<h2>Build Profile</h2>
<pre>{cargo_toml}</pre>
</body></html>"#,
        style = CRT_STYLE,
        nav = nav(),
        ver = env!("CARGO_PKG_VERSION"),
        count = pkgs.len(),
        table = table,
        cargo_toml = CARGO_TOML.replace('<', "&lt;").replace('>', "&gt;"),
    );
    Html(html).into_response()
}

/// f25=govdocs_capability — GET /govdocs/capability
pub async fn f25() -> impl IntoResponse {
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Capability — Pocket Server</title><style>{style}</style></head><body>
{nav}
{body}
</body></html>"#,
        style = CRT_STYLE,
        nav = nav(),
        body = md_to_html(CAPABILITY_MD),
    );
    Html(html)
}

/// f26=govdocs_security — GET /govdocs/security
pub async fn f26() -> impl IntoResponse {
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Security — Pocket Server</title><style>{style}</style></head><body>
{nav}
{body}
</body></html>"#,
        style = CRT_STYLE,
        nav = nav(),
        body = md_to_html(SECURITY_MD),
    );
    Html(html)
}

/// Generate SPDX 2.3 SBOM document.
pub fn generate_spdx() -> String {
    let pkgs = parse_lock_packages();
    let mut out = String::new();

    out.push_str("SPDXVersion: SPDX-2.3\n");
    out.push_str("DataLicense: CC0-1.0\n");
    out.push_str("SPDXID: SPDXRef-DOCUMENT\n");
    out.push_str(&format!("DocumentName: pocket-server-{}\n", env!("CARGO_PKG_VERSION")));
    out.push_str("DocumentNamespace: https://cochranblock.org/spdx/pocket-server\n");
    out.push_str("Creator: Tool: pocket-server-builtin-sbom\n");
    out.push_str("Creator: Organization: CochranBlock\n");
    out.push_str(&format!("Created: {}\n", created_timestamp()));
    out.push('\n');

    // Root package
    out.push_str("PackageName: pocket-server\n");
    out.push_str("SPDXID: SPDXRef-Package-pocket-server\n");
    out.push_str(&format!("PackageVersion: {}\n", env!("CARGO_PKG_VERSION")));
    out.push_str("PackageDownloadLocation: https://github.com/cochranblock/pocket-server\n");
    out.push_str("PackageLicenseConcluded: Unlicense\n");
    out.push_str("PackageLicenseDeclared: Unlicense\n");
    out.push_str("PackageCopyrightText: NOASSERTION\n");
    out.push('\n');

    // Dependencies
    for (name, ver) in &pkgs {
        let spdx_id = name.replace('-', "_");
        out.push_str(&format!("PackageName: {}\n", name));
        out.push_str(&format!("SPDXID: SPDXRef-Package-{}\n", spdx_id));
        out.push_str(&format!("PackageVersion: {}\n", ver));
        out.push_str(&format!(
            "PackageDownloadLocation: https://crates.io/crates/{}/{}\n",
            name, ver
        ));
        out.push_str("PackageLicenseConcluded: NOASSERTION\n");
        out.push_str("PackageLicenseDeclared: NOASSERTION\n");
        out.push_str("PackageCopyrightText: NOASSERTION\n");
        out.push_str(&format!(
            "Relationship: SPDXRef-Package-pocket-server DEPENDS_ON SPDXRef-Package-{}\n",
            spdx_id
        ));
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lock_packages_returns_deps() {
        let pkgs = parse_lock_packages();
        assert!(!pkgs.is_empty());
        // Should not include self
        assert!(!pkgs.iter().any(|(n, _)| n == "pocket-server"));
        // Should include axum (direct dep)
        assert!(pkgs.iter().any(|(n, _)| n == "axum"));
    }

    #[test]
    fn spdx_output_valid() {
        let spdx = generate_spdx();
        assert!(spdx.starts_with("SPDXVersion: SPDX-2.3"));
        assert!(spdx.contains("DataLicense: CC0-1.0"));
        assert!(spdx.contains("PackageName: pocket-server"));
        assert!(spdx.contains("PackageLicenseConcluded: Unlicense"));
        assert!(spdx.contains("DEPENDS_ON"));
        assert!(spdx.contains("cochranblock.org"));
    }

    #[test]
    fn md_to_html_headings() {
        let html = md_to_html("# Title\n## Subtitle\n### Sub-sub");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<h2>Subtitle</h2>"));
        assert!(html.contains("<h3>Sub-sub</h3>"));
    }

    #[test]
    fn md_to_html_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = md_to_html(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>A</th>"));
        assert!(html.contains("<td>1</td>"));
    }

    #[test]
    fn md_to_html_list() {
        let md = "- item one\n- item two";
        let html = md_to_html(md);
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>item one</li>"));
    }
}

fn created_timestamp() -> String {
    // Use build-time env or fallback
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // ISO 8601 approximation from unix timestamp
    let days = secs / 86400;
    let years = 1970 + days / 365;
    let rem_days = days % 365;
    let month = rem_days / 30 + 1;
    let day = rem_days % 30 + 1;
    let hour = (secs % 86400) / 3600;
    let min = (secs % 3600) / 60;
    let sec = secs % 60;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        years, month, day, hour, min, sec
    )
}
