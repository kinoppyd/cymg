use std::error::Error;
use std::fs::File;
use std::path::Path;

use image::codecs::gif::{GifEncoder, Repeat};
use image::imageops::FilterType;
use image::{Delay, Frame, RgbImage, Rgba, RgbaImage};

const KEYFRAME_HOLD_MS: u32 = 1_200;
const FLASH_MS: u32 = 70;
const RETURN_MS: u32 = 150;
const GIF_ENCODER_SPEED: i32 = 30;
const MAX_GIF_EDGE: u32 = 960;

pub(crate) fn write_flash_animation_gif(
    output_path: &Path,
    frames: &[RgbImage],
) -> Result<(), Box<dyn Error>> {
    if frames.is_empty() {
        return Err("animation requires at least one frame".into());
    }

    let resized_frames: Vec<RgbaImage> = frames
        .iter()
        .map(|frame| rgb_to_rgba(&resize_for_gif(frame)))
        .collect();
    let first_keyframe = resized_frames[0].clone();

    let file = File::create(output_path)?;
    let mut encoder = GifEncoder::new_with_speed(file, GIF_ENCODER_SPEED);
    encoder.set_repeat(Repeat::Infinite)?;

    encoder.encode_frame(to_frame(first_keyframe.clone(), KEYFRAME_HOLD_MS))?;

    if resized_frames.len() > 4 {
        for alt in &resized_frames[1..4] {
            encoder.encode_frame(to_frame(alt.clone(), FLASH_MS))?;
            encoder.encode_frame(to_frame(first_keyframe.clone(), RETURN_MS))?;
        }

        let second_keyframe = resized_frames[4].clone();
        encoder.encode_frame(to_frame(second_keyframe.clone(), KEYFRAME_HOLD_MS))?;

        let tail = &resized_frames[5..];
        for (i, alt) in tail.iter().enumerate() {
            encoder.encode_frame(to_frame(alt.clone(), FLASH_MS))?;
            if i + 1 != tail.len() {
                encoder.encode_frame(to_frame(second_keyframe.clone(), RETURN_MS))?;
            }
        }
    } else {
        let tail = &resized_frames[1..];
        for (i, alt) in tail.iter().enumerate() {
            encoder.encode_frame(to_frame(alt.clone(), FLASH_MS))?;
            if i + 1 != tail.len() {
                encoder.encode_frame(to_frame(first_keyframe.clone(), RETURN_MS))?;
            }
        }
    }
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
