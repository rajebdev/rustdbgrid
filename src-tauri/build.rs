use std::time::SystemTime;

fn main() {
    // Set APP_VERSION environment variable from Cargo.toml
    let version = env!("CARGO_PKG_VERSION");
    println!("cargo::rustc-env=APP_VERSION={}", version);

    // Set APP_YEAR environment variable from current build time
    let year = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .map(|d| {
            // Convert to year (rough calculation: 1970 + seconds / seconds per year)
            let secs = d.as_secs() as f64;
            let years_since_1970 = (secs / (365.25 * 24.0 * 3600.0)) as u32;
            1970 + years_since_1970
        })
        .unwrap_or(2024);

    println!("cargo::rustc-env=APP_YEAR={}", year);

    tauri_build::build()
}
