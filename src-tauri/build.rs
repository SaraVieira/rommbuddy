fn main() {
    // Load .env from the repo root and forward ScreenScraper credentials to env!() macros
    let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env");
    if env_path.exists() {
        dotenvy::from_path(&env_path).ok();
    }

    for key in [
        "SCREENSCRAPER_DEV_ID",
        "SCREENSCRAPER_DEV_PASSWORD",
        "SCREENSCRAPER_SOFT_NAME",
    ] {
        if let Ok(val) = std::env::var(key) {
            println!("cargo:rustc-env={key}={val}");
        }
    }

    tauri_build::build();
}
