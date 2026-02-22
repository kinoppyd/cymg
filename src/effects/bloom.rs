use image::{ImageBuffer, RgbImage};
use rayon::prelude::*;

use super::math::clamp_to_u8;

pub(super) fn apply_bloom(src: &RgbImage, bloom: f32) -> RgbImage {
    let width = src.width();
    let height = src.height();
    if width == 0 || height == 0 {
        return src.clone();
    }

    let len = (width as usize) * (height as usize);
    let src_raw = src.as_raw();
    let mut bright_map = vec![[0.0_f32; 3]; len];
    let threshold = 150.0 - bloom * 40.0;

    bright_map
        .par_iter_mut()
        .enumerate()
        .for_each(|(idx, out)| {
            let base = idx * 3;
            let r = src_raw[base] as f32;
            let g = src_raw[base + 1] as f32;
            let b = src_raw[base + 2] as f32;
            let lum = r * 0.2126 + g * 0.7152 + b * 0.0722;
            let k = ((lum - threshold) / (255.0 - threshold)).clamp(0.0, 1.0);
            *out = [r * k, g * k, b * k];
        });

    let passes = if bloom < 0.34 {
        1
    } else if bloom < 0.67 {
        2
    } else {
        3
    };

    for _ in 0..passes {
        bright_map = blur3x3_separable(&bright_map, width, height);
    }

    let gain = bloom * 0.85;
    let mut out_buf = vec![0_u8; src_raw.len()];
    out_buf
        .par_chunks_mut(3)
        .enumerate()
        .for_each(|(idx, pix)| {
            let base = idx * 3;
            let glow = bright_map[idx];
            let r = src_raw[base] as f32 + glow[0] * gain;
            let g = src_raw[base + 1] as f32 + glow[1] * gain;
            let b = src_raw[base + 2] as f32 + glow[2] * gain;
            pix[0] = clamp_to_u8(r);
            pix[1] = clamp_to_u8(g);
            pix[2] = clamp_to_u8(b);
        });

    ImageBuffer::from_raw(width, height, out_buf).expect("buffer length matches image")
}

fn blur3x3_separable(input: &[[f32; 3]], width: u32, height: u32) -> Vec<[f32; 3]> {
    let w = width as usize;
    let h = height as usize;
    if w == 0 || h == 0 {
        return Vec::new();
    }

    let mut horizontal = vec![[0.0_f32; 3]; input.len()];
    horizontal
        .par_chunks_mut(w)
        .enumerate()
        .for_each(|(y, row)| {
            let base = y * w;
            for x in 0..w {
                let xl = x.saturating_sub(1);
                let xr = (x + 1).min(w - 1);
                let p0 = input[base + xl];
                let p1 = input[base + x];
                let p2 = input[base + xr];
                row[x] = [
                    (p0[0] + p1[0] + p2[0]) / 3.0,
                    (p0[1] + p1[1] + p2[1]) / 3.0,
                    (p0[2] + p1[2] + p2[2]) / 3.0,
                ];
            }
        });

    let mut out = vec![[0.0_f32; 3]; input.len()];
    out.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let yu = y.saturating_sub(1);
        let yd = (y + 1).min(h - 1);
        for (x, pix) in row.iter_mut().enumerate() {
            let p0 = horizontal[yu * w + x];
            let p1 = horizontal[y * w + x];
            let p2 = horizontal[yd * w + x];
            *pix = [
                (p0[0] + p1[0] + p2[0]) / 3.0,
                (p0[1] + p1[1] + p2[1]) / 3.0,
                (p0[2] + p1[2] + p2[2]) / 3.0,
            ];
        }
    });

    out
}
