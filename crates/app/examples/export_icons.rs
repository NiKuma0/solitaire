//! Writes the app icon as PNG files for platform packaging:
//!
//! - `export_icons android <res dir>`: `<res dir>/mipmap-*/ic_launcher.png`
//! - `export_icons iconset <dir>.iconset`: input for macOS `iconutil`

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

/// Launcher icon sizes per Android screen density.
const ANDROID: [(&str, u32); 5] = [
    ("mipmap-mdpi", 48),
    ("mipmap-hdpi", 72),
    ("mipmap-xhdpi", 96),
    ("mipmap-xxhdpi", 144),
    ("mipmap-xxxhdpi", 192),
];

fn write_png(path: &Path, size: u32) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut encoder = png::Encoder::new(BufWriter::new(File::create(path)?), size, size);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()?
        .write_image_data(&solitaire_app::app_icon::rgba(size))?;
    println!("{}", path.display());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let usage = "usage: export_icons <android|iconset> <dir>";
    let kind = args.next().ok_or(usage)?;
    let out = PathBuf::from(args.next().ok_or(usage)?);
    match kind.as_str() {
        "android" => {
            for (dir, size) in ANDROID {
                write_png(&out.join(dir).join("ic_launcher.png"), size)?;
            }
        }
        "iconset" => {
            for base in [16, 32, 128, 256, 512] {
                write_png(&out.join(format!("icon_{base}x{base}.png")), base)?;
                write_png(&out.join(format!("icon_{base}x{base}@2x.png")), base * 2)?;
            }
        }
        _ => return Err(usage.into()),
    }
    Ok(())
}
