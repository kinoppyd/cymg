use std::error::Error;
use std::fs::File;
use std::path::Path;

use image::codecs::gif::{GifEncoder, Repeat};
use image::imageops::FilterType;
use image::{Delay, Frame, RgbImage, Rgba, RgbaImage};

const KEYFRAME_HOLD_MS: u32 = 1_200;
const FLASH_MS: u32 = 70;
const RETURN_MS: u32 = 150;
const END_HOLD_MS: u32 = 700;
const GIF_ENCODER_SPEED: i32 = 30;
const MAX_GIF_EDGE: u32 = 960;

pub(crate) fn write_flash_animation_gif(
    output_path: &Path,
    frames: &[RgbImage],
) -> Result<(), Box<dyn Error>> {
    if frames.is_empty() {
        return Err("animation requires at least one frame".into());
    }

    let keyframe_rgb = resize_for_gif(&frames[0]);
    let keyframe = rgb_to_rgba(&keyframe_rgb);

    let file = File::create(output_path)?;
    let mut encoder = GifEncoder::new_with_speed(file, GIF_ENCODER_SPEED);
    encoder.set_repeat(Repeat::Infinite)?;

    encoder.encode_frame(to_frame(keyframe.clone(), KEYFRAME_HOLD_MS))?;

    for alt_rgb in &frames[1..] {
        let alt_resized = resize_for_gif(alt_rgb);
        let alt = rgb_to_rgba(&alt_resized);
        encoder.encode_frame(to_frame(alt, FLASH_MS))?;
        encoder.encode_frame(to_frame(keyframe.clone(), RETURN_MS))?;
    }

    encoder.encode_frame(to_frame(keyframe, END_HOLD_MS))?;
    Ok(())
}

pub(crate) fn resize_for_gif(src: &RgbImage) -> RgbImage {
    let (w, h) = src.dimensions();
    let longest = w.max(h);
    if longest <= MAX_GIF_EDGE {
        return src.clone();
    }

    let scale = MAX_GIF_EDGE as f32 / longest as f32;
    let target_w = ((w as f32 * scale).round()).max(1.0) as u32;
    let target_h = ((h as f32 * scale).round()).max(1.0) as u32;
    image::imageops::resize(src, target_w, target_h, FilterType::Triangle)
}

fn rgb_to_rgba(src: &RgbImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    RgbaImage::from_fn(w, h, |x, y| {
        let px = src.get_pixel(x, y).0;
        Rgba([px[0], px[1], px[2], 255])
    })
}

fn to_frame(buf: RgbaImage, duration_ms: u32) -> Frame {
    let delay = Delay::from_numer_denom_ms(duration_ms, 1);
    Frame::from_parts(buf, 0, 0, delay)
}
