use inflector::Inflector;
use proc_macro2::Literal;
use quote::{format_ident, quote};
use std::{collections::BTreeMap, env, fs, path::PathBuf};
use syn::parse_quote;

pub type Result<T, E = Box<dyn std::error::Error>> = core::result::Result<T, E>;

#[derive(Clone, Debug)]
pub struct Config {
    cdn_base: String,
    version: String,
    out_dir: PathBuf,
}

impl Config {
    pub fn new(out_dir: impl Into<PathBuf>) -> Self {
        Self {
            cdn_base: "https://unpkg.com".to_owned(),
            version: "latest".to_owned(),
            out_dir: out_dir.into(),
        }
    }

    pub fn try_from_env() -> Result<Self, env::VarError> {
        env::var_os("OUT_DIR")
            .ok_or(env::VarError::NotPresent)
            .map(Self::new)
    }

    pub fn with_cdn_base(self, cdn_base: impl Into<String>) -> Self {
        let mut slf = self;
        slf.cdn_base = cdn_base.into();
        slf
    }

    pub fn with_version(self, version: impl Into<String>) -> Self {
        let mut slf = self;
        slf.version = version.into();
        slf
    }
}

pub fn download_and_generate(config: &Config) -> Result<()> {
    let font_path = {
        let p = config.out_dir.join("lucide.ttf");

        let _ = std::io::copy(
            &mut reqwest::blocking::get(format!(
                "{}/lucide-static@{}/font/lucide.ttf",
                config.cdn_base, config.version
            ))?,
            &mut fs::File::create(&p)?,
        )?;

        Literal::string(p.to_str().unwrap())
    };

    let codepoints = {
        reqwest::blocking::get(format!(
            "{}/lucide-static@{}/font/codepoints.json",
            config.cdn_base, config.version
        ))?
        .json::<BTreeMap<String, u32>>()?
    };

    let range_start = Literal::u32_suffixed(codepoints.values().min().copied().unwrap_or(0));
    let range_end = Literal::u32_suffixed(codepoints.values().max().copied().unwrap_or(0));

    let glyphs = codepoints.into_iter().filter_map(|(k, v)| {
        let c = Literal::string(&char::from_u32(v)?.to_string());
        let ident = format_ident!("{}", k.to_screaming_snake_case());
        Some(quote! {
            pub const #ident: &str = #c;
        })
    });

    fs::write(
        config.out_dir.join("lucide.gen.rs"),
        prettyplease::unparse(&parse_quote! {
            const LUCIDE_TTF: &[u8] = include_bytes!(#font_path);

            const LUCIDE_TTF_RANGE: [u32; 3] = [#range_start, #range_end, 0u32];

            pub fn font_source(size: f32) -> imgui::FontSource<'static> {
                imgui::FontSource::TtfData {
                    data: LUCIDE_TTF,
                    size_pixels: size,
                    config: Some(imgui::FontConfig {
                        size_pixels: size,
                        pixel_snap_h: true,
                        glyph_offset: [0., (size / 5.).round()],
                        glyph_ranges: imgui::FontGlyphRanges::from_slice(&LUCIDE_TTF_RANGE),
                        glyph_min_advance_x: size,
                        name: Some("lucide.ttf".to_owned()),
                        ..Default::default()
                    }),
                }
            }

            #[allow(unused)]
            pub mod icons {
                #(#glyphs)*
            }
        }),
    )?;

    Ok(())
}
