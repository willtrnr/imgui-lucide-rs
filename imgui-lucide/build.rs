pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    cargo_emit::rerun_if_changed!("build.rs");
    cargo_emit::rerun_if_env_changed!("LUCIDE_CDN_BASE", "LUCIDE_VERSION");

    let config = {
        let mut c = imgui_lucide_build::Config::try_from_env()?;
        if let Ok(v) = std::env::var("LUCIDE_CDN_BASE") {
            c = c.with_cdn_base(v);
        }
        if let Ok(v) = std::env::var("LUCIDE_VERSION") {
            c = c.with_version(v);
        }
        c
    };

    imgui_lucide_build::download_and_generate(&config)?;

    Ok(())
}
