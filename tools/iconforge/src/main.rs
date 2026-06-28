//! Renders the "Clean My Shit" icon: a blue rounded tile (drawn via resvg) with
//! an openly-licensed pile-of-poo emoji composited on top. Exports PNG (1024 +
//! 256) and a multi-resolution Windows .ico.
//!
//! Emoji source is the 2nd CLI arg: `fluent` (MS Fluent 3D, MIT) |
//! `noto` (Google Noto, Apache-2.0) | `twemoji` (CC-BY-4.0). Default: fluent.

use std::path::{Path, PathBuf};

use resvg::tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};
use resvg::usvg;

const BG_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
  <defs><linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0" stop-color="#4F8DF5"/><stop offset="1" stop-color="#1D4ED8"/>
  </linearGradient></defs>
  <rect x="64" y="64" width="896" height="896" rx="200" fill="url(#bg)"/>
  <g fill="#FFFFFF">
    <path d="M214,286 Q214,318 246,318 Q214,318 214,350 Q214,318 182,318 Q214,318 214,286 Z" opacity="0.92"/>
    <path d="M820,250 Q820,288 858,288 Q820,288 820,326 Q820,288 782,288 Q820,288 820,250 Z" opacity="0.9"/>
    <path d="M694,168 Q694,188 714,188 Q694,188 694,208 Q694,188 674,188 Q694,188 694,168 Z" opacity="0.8"/>
  </g>
</svg>"##;

const FLUENT_PNG: &[u8] = include_bytes!("../poo.png");
const NOTO_SVG: &str = include_str!("../poo-noto.svg");
const TWEMOJI_SVG: &str = include_str!("../poo-twemoji.svg");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets"));
    let source = std::env::args().nth(2).unwrap_or_else(|| "fluent".into());
    std::fs::create_dir_all(&out)?;

    // High-res emoji, rendered/decoded once.
    let emoji = match source.as_str() {
        "noto" => render_svg(NOTO_SVG, 1024),
        "twemoji" => render_svg(TWEMOJI_SVG, 1024),
        _ => fluent_pixmap()?, // MS Fluent 3D — upscaled from 256 with Lanczos
    };

    write_png(&out.join("icon-1024.png"), 1024, &emoji)?;
    write_png(&out.join("icon.png"), 256, &emoji)?;
    write_ico(&out.join("icon.ico"), &emoji)?;

    println!("icons written to {} (source: {source})", out.display());
    Ok(())
}

/// Decode the Fluent 3D poo (256px) and Lanczos-upscale it to 1024 for the
/// sharpest result the source allows.
fn fluent_pixmap() -> Result<Pixmap, Box<dyn std::error::Error>> {
    let img = image::load_from_memory_with_format(FLUENT_PNG, image::ImageFormat::Png)?.to_rgba8();
    let up = image::imageops::resize(&img, 1024, 1024, image::imageops::FilterType::Lanczos3);
    Ok(image_to_pixmap(&up))
}

/// Convert a straight-alpha RGBA image into a premultiplied tiny_skia pixmap.
fn image_to_pixmap(img: &image::RgbaImage) -> Pixmap {
    let (w, h) = img.dimensions();
    let mut pm = Pixmap::new(w, h).expect("alloc");
    let dst = pm.data_mut();
    for (i, px) in img.pixels().enumerate() {
        let [r, g, b, a] = px.0;
        let af = a as u16;
        dst[i * 4] = (r as u16 * af / 255) as u8;
        dst[i * 4 + 1] = (g as u16 * af / 255) as u8;
        dst[i * 4 + 2] = (b as u16 * af / 255) as u8;
        dst[i * 4 + 3] = a;
    }
    pm
}

/// Render an SVG string into a square `size`×`size` pixmap, fit to the box.
fn render_svg(svg: &str, size: u32) -> Pixmap {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).expect("svg parse");
    let mut pm = Pixmap::new(size, size).expect("alloc");
    let s = tree.size();
    let scale = size as f32 / s.width().max(s.height());
    let tx = (size as f32 - s.width() * scale) / 2.0;
    let ty = (size as f32 - s.height() * scale) / 2.0;
    resvg::render(
        &tree,
        Transform::from_row(scale, 0.0, 0.0, scale, tx, ty),
        &mut pm.as_mut(),
    );
    pm
}

/// Compose the tile + emoji at the requested size, centered on the emoji's
/// actual opaque content (ignoring transparent padding in the source).
fn compose(size: u32, emoji: &Pixmap) -> Pixmap {
    let mut bg = render_svg(BG_SVG, size);

    let (bx, by, bw, bh) = content_bbox(emoji);
    let box_px = size as f32 * 0.72;
    let scale = box_px / bw.max(bh) as f32;
    // Place the content bbox center at the tile center.
    let tx = size as f32 / 2.0 - (bx as f32 + bw as f32 / 2.0) * scale;
    let ty = size as f32 / 2.0 - (by as f32 + bh as f32 / 2.0) * scale;

    bg.draw_pixmap(
        0,
        0,
        emoji.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Bicubic,
            ..Default::default()
        },
        Transform::from_row(scale, 0.0, 0.0, scale, tx, ty),
        None,
    );
    bg
}

/// Bounding box (x, y, w, h) of the non-transparent pixels in `pm`.
fn content_bbox(pm: &Pixmap) -> (u32, u32, u32, u32) {
    let (w, h) = (pm.width(), pm.height());
    let data = pm.data();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0u32, 0u32);
    let mut found = false;
    for y in 0..h {
        for x in 0..w {
            if data[((y * w + x) * 4 + 3) as usize] > 10 {
                found = true;
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    if !found {
        return (0, 0, w, h);
    }
    (x0, y0, x1 - x0 + 1, y1 - y0 + 1)
}

fn write_png(path: &Path, size: u32, emoji: &Pixmap) -> Result<(), Box<dyn std::error::Error>> {
    let png = compose(size, emoji)
        .encode_png()
        .map_err(|e| format!("png encode: {e}"))?;
    std::fs::write(path, png)?;
    Ok(())
}

fn write_ico(path: &Path, emoji: &Pixmap) -> Result<(), Box<dyn std::error::Error>> {
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
    for size in [16u32, 32, 48, 64, 128, 256] {
        let rgba = unpremultiply(compose(size, emoji).data());
        let img = ico::IconImage::from_rgba_data(size, size, rgba);
        dir.add_entry(ico::IconDirEntry::encode(&img)?);
    }
    dir.write(std::fs::File::create(path)?)?;
    Ok(())
}

fn unpremultiply(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    for px in data.chunks_exact(4) {
        let a = px[3];
        if a == 0 {
            out.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let f = 255.0 / a as f32;
            out.push((px[0] as f32 * f).round().min(255.0) as u8);
            out.push((px[1] as f32 * f).round().min(255.0) as u8);
            out.push((px[2] as f32 * f).round().min(255.0) as u8);
            out.push(a);
        }
    }
    out
}
