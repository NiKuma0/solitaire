//! Writes the app icon as Android launcher PNGs:
//! `cargo run -p solitaire-app --example export_icons -- <res dir>`
//! creates `<res dir>/mipmap-*/ic_launcher.png`.

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

/// Launcher icon sizes per Android screen density.
const DENSITIES: [(&str, u32); 5] = [
    ("mdpi", 48),
    ("hdpi", 72),
    ("xhdpi", 96),
    ("xxhdpi", 144),
    ("xxxhdpi", 192),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let res = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("usage: export_icons <res dir>")?,
    );
    for (density, size) in DENSITIES {
        let dir = res.join(format!("mipmap-{density}"));
        fs::create_dir_all(&dir)?;
        let path = dir.join("ic_launcher.png");
        let mut encoder = png::Encoder::new(BufWriter::new(File::create(&path)?), size, size);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()?
            .write_image_data(&solitaire_app::app_icon::rgba(size))?;
        println!("{}", path.display());
    }
    Ok(())
}
