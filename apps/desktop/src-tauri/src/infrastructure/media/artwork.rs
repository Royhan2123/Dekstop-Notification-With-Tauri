//! Mengecilkan gambar sampul (<= 256 px) dan menghitung warna dominannya.
use std::io::Cursor;

use base64::Engine;
use image::{DynamicImage, ImageFormat};

const MAX_SIDE: u32 = 256;

pub struct Artwork {
    /// `data:image/jpeg;base64,...`
    pub data_url: String,
    /// Rata-rata warna, "#rrggbb".
    pub color: String,
}

/// Gambar dikecilkan sampai sisi terpanjang `max_side` lalu dijadikan data URL JPEG.
pub fn data_url(bytes: &[u8], max_side: u32) -> Option<String> {
    shrink(&image::load_from_memory(bytes).ok()?, max_side)
}

fn shrink(img: &DynamicImage, max_side: u32) -> Option<String> {
    let small = DynamicImage::ImageRgb8(img.thumbnail(max_side, max_side).to_rgb8());
    let mut jpeg = Cursor::new(Vec::new());
    small.write_to(&mut jpeg, ImageFormat::Jpeg).ok()?;
    Some(format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(jpeg.into_inner())
    ))
}

const FETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);
const MAX_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024;

/// Unduh sampul dari URL https (batas waktu 3 dtk, maks 2 MB) lalu olah. `None` bila gagal.
/// URL berasal dari ekstensi/pemutar yang tidak sepenuhnya dipercaya, jadi hanya https.
pub fn fetch(url: &str) -> Option<Artwork> {
    use std::io::Read;
    if !url.starts_with("https://") {
        return None;
    }
    let resp = reqwest::blocking::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .build()
        .ok()?
        .get(url)
        .send()
        .ok()?
        .error_for_status()
        .ok()?;
    let mut bytes = Vec::new();
    resp.take(MAX_DOWNLOAD_BYTES).read_to_end(&mut bytes).ok()?;
    process(&bytes)
}

/// `None` bila format tidak dikenali (hanya JPEG dan PNG).
pub fn process(bytes: &[u8]) -> Option<Artwork> {
    let img = image::load_from_memory(bytes).ok()?;
    let avg = img.thumbnail(1, 1).to_rgb8();
    let [r, g, b] = avg.get_pixel(0, 0).0;
    Some(Artwork {
        data_url: shrink(&img, MAX_SIDE)?,
        color: format!("#{r:02x}{g:02x}{b:02x}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(w, h, image::Rgb(rgb)));
        let mut out = Cursor::new(Vec::new());
        img.write_to(&mut out, ImageFormat::Png).unwrap();
        out.into_inner()
    }

    #[test]
    fn gambar_besar_dikecilkan_dan_warna_dominan_dihitung() {
        let art = process(&png(1000, 500, [255, 0, 0])).unwrap();
        assert_eq!(art.color, "#ff0000");
        assert!(art.data_url.starts_with("data:image/jpeg;base64,"));
        let b64 = art.data_url.trim_start_matches("data:image/jpeg;base64,");
        let jpeg = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap();
        let out = image::load_from_memory(&jpeg).unwrap();
        assert_eq!((out.width(), out.height()), (256, 128));
    }

    #[test]
    fn data_rusak_menghasilkan_none() {
        assert!(process(b"bukan gambar").is_none());
    }
}
