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

/// `None` bila format tidak dikenali (hanya JPEG dan PNG).
pub fn process(bytes: &[u8]) -> Option<Artwork> {
    let img = image::load_from_memory(bytes).ok()?;
    let small = DynamicImage::ImageRgb8(img.thumbnail(MAX_SIDE, MAX_SIDE).to_rgb8());
    let mut jpeg = Cursor::new(Vec::new());
    small.write_to(&mut jpeg, ImageFormat::Jpeg).ok()?;

    let avg = img.thumbnail(1, 1).to_rgb8();
    let [r, g, b] = avg.get_pixel(0, 0).0;
    Some(Artwork {
        data_url: format!(
            "data:image/jpeg;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(jpeg.into_inner())
        ),
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
