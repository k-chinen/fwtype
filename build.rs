use std::env;
use std::process::Command;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

fn main() {
    // --- Gitの短縮ハッシュ ---
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // --- ビルド日時（UTC, RFC3339）---
    let build_dt = match env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
    {
        Some(epoch) => OffsetDateTime::from_unix_timestamp(epoch).unwrap_or_else(|_| OffsetDateTime::now_utc()),
        None => OffsetDateTime::now_utc(),
    };
    let build_date = build_dt.format(&Rfc3339).unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
}
