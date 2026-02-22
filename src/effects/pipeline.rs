use image::{ImageBuffer, RgbImage};
use rayon::prelude::*;

use crate::cli::EffectSettings;

use super::bloom::apply_bloom;
use super::color::{
    apply_color_overdrive, apply_highlight_recovery, apply_matrix_tone, apply_saturation,
    color_gains,
};
use super::math::{clamp_to_u8, random_shift, random_unit, sample_rgb};

struct RowContext {
    y_u32: u32,
    y_i32: i32,
    row_glitch: bool,
    red_shift: i32,
    blue_shift: i32,
}

pub(crate) fn apply_cyberpunk_effect(src: &RgbImage, settings: EffectSettings) -> RgbImage {
    let width = src.width();
    let height = src.height();
    if width == 0 || height == 0 {
        return src.clone();
    }

    let row_stride = width as usize * 3;
    let mut processed_buf = vec![0_u8; row_stride * height as usize];
    let (red_gain, green_gain, blue_gain) = color_gains(settings.color_boost);

    processed_buf
        .par_chunks_mut(row_stride)
        .enumerate()
        .for_each(|(y, row)| {
            let row_ctx = build_row_context(settings, y);

            for x in 0..width {
                let x_i32 = x as i32;
                let src_r = sample_rgb(src, x_i32 + row_ctx.red_shift, row_ctx.y_i32);
                let src_g = sample_rgb(src, x_i32, row_ctx.y_i32);
                let src_b = sample_rgb(src, x_i32 + row_ctx.blue_shift, row_ctx.y_i32);

                let mut r = src_r[0] as f32;
                let mut g = src_g[1] as f32;
                let mut b = src_b[2] as f32;

                r *= 0.90;
                g *= 0.68;
                b *= 0.98;

                let r_mix = r * 0.90 + b * 0.18;
                let g_mix = g * 0.95 + r * 0.04;
                let b_mix = b * 1.14 + r * 0.08;
                r = r_mix;
                g = g_mix;
                b = b_mix;

                apply_scanline(x, row_ctx.y_u32, settings, &mut r, &mut g, &mut b);
                apply_vignette(
                    x,
                    row_ctx.y_u32,
                    width,
                    height,
                    settings,
                    &mut r,
                    &mut g,
                    &mut b,
                );

                if row_ctx.row_glitch && row_ctx.y_u32 % 7 == 0 {
                    r *= 1.08;
                    b *= 1.10;
                }

                r *= red_gain;
                g *= green_gain;
                b *= blue_gain;

                apply_saturation(&mut r, &mut g, &mut b, settings.saturation);

                r = (r - 128.0) * settings.contrast + 128.0 + settings.brightness * 255.0;
                g = (g - 128.0) * settings.contrast + 128.0 + settings.brightness * 255.0;
                b = (b - 128.0) * settings.contrast + 128.0 + settings.brightness * 255.0;

                apply_color_overdrive(&mut r, &mut g, &mut b, settings.color_boost);
                apply_matrix_tone(&mut r, &mut g, &mut b, settings.color_boost);
                apply_highlight_recovery(&mut r, &mut g, &mut b, settings.color_boost);
                apply_noise(settings, x, row_ctx.y_u32, &mut r, &mut g, &mut b);

                let base = x as usize * 3;
                row[base] = clamp_to_u8(r);
                row[base + 1] = clamp_to_u8(g);
                row[base + 2] = clamp_to_u8(b);
            }
        });

    let mut processed =
        ImageBuffer::from_raw(width, height, processed_buf).expect("buffer length matches image");

    if settings.bloom > 0.0 {
        processed = apply_bloom(&processed, settings.bloom);
    }

    blend_with_original(src, &processed, settings.intensity)
}

fn build_row_context(settings: EffectSettings, y: usize) -> RowContext {
    let y_u32 = y as u32;
    let y_u64 = y as u64;
    let y_i32 = y as i32;
    let row_glitch = random_unit(settings.seed, 991, y_u64, 3) < settings.glitch_rate;

    let base_red_shift = random_shift(settings.seed, y_u64, settings.glitch_shift, 17);
    let base_blue_shift = random_shift(settings.seed, y_u64, settings.glitch_shift, 23);
    let row_boost = if row_glitch { settings.glitch_shift } else { 0 };

    RowContext {
        y_u32,
        y_i32,
        row_glitch,
        red_shift: base_red_shift + row_boost,
        blue_shift: base_blue_shift - row_boost,
    }
}

fn apply_scanline(x: u32, y: u32, settings: EffectSettings, r: &mut f32, g: &mut f32, b: &mut f32) {
    if settings.scanline <= 0.0 {
        return;
    }

    let depth = settings.scanline.clamp(0.0, 1.0);
    let phase = y % settings.scanline_step;

    let mut scanline_factor = if phase == 0 {
        1.0 - depth * 1.10
    } else if phase == 1 {
        1.0 + depth * 0.98
    } else {
        1.0 + depth * 0.35
    };

    let aperture = if x % 3 == 0 {
        1.0 + depth * 0.14
    } else {
        1.0 - depth * 0.07
    };
    scanline_factor *= aperture;

    if phase == 0 {
        *r *= scanline_factor * (1.0 + depth * 0.05);
        *g *= scanline_factor * (1.0 - depth * 0.06);
        *b *= scanline_factor * (1.0 + depth * 0.07);
        *r -= depth * 18.0;
        *g -= depth * 24.0;
        *b -= depth * 16.0;
    } else if phase == 1 {
        *r *= scanline_factor * (1.0 + depth * 0.10);
        *g *= scanline_factor * (1.0 + depth * 0.04);
        *b *= scanline_factor * (1.0 + depth * 0.13);
        *r += depth * 12.0;
        *g += depth * 10.0;
        *b += depth * 15.0;
    } else {
        *r *= scanline_factor;
        *g *= scanline_factor;
        *b *= scanline_factor;
    }
}

fn apply_vignette(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    settings: EffectSettings,
    r: &mut f32,
    g: &mut f32,
    b: &mut f32,
) {
    if settings.vignette <= 0.0 {
        return;
    }

    let nx = if width > 1 {
        (x as f32 / (width - 1) as f32) * 2.0 - 1.0
    } else {
        0.0
    };
    let ny = if height > 1 {
        (y as f32 / (height - 1) as f32) * 2.0 - 1.0
    } else {
        0.0
    };

    let dist = ((nx * nx + ny * ny).sqrt() / 1.414_213_5).min(1.0);
    let vignette_factor = 1.0 - dist.powf(1.8) * settings.vignette;

    *r *= vignette_factor;
    *g *= vignette_factor * 0.97;
    *b *= vignette_factor * 1.03;
}

fn apply_noise(settings: EffectSettings, x: u32, y: u32, r: &mut f32, g: &mut f32, b: &mut f32) {
    if settings.noise <= 0.0 {
        return;
    }

    let noise_amp = settings.noise * 36.0;
    *r += (random_unit(settings.seed, x as u64, y as u64, 101) * 2.0 - 1.0) * noise_amp;
    *g += (random_unit(settings.seed, x as u64, y as u64, 211) * 2.0 - 1.0) * noise_amp;
    *b += (random_unit(settings.seed, x as u64, y as u64, 307) * 2.0 - 1.0) * noise_amp;
}

fn blend_with_original(src: &RgbImage, cyber: &RgbImage, intensity: f32) -> RgbImage {
    if (intensity - 1.0).abs() < f32::EPSILON {
        return cyber.clone();
    }

    let width = src.width();
    let height = src.height();
    if width == 0 || height == 0 {
        return cyber.clone();
    }

    let src_raw = src.as_raw();
    let cyber_raw = cyber.as_raw();
    let mut out_buf = vec![0_u8; src_raw.len()];

    out_buf
        .par_chunks_mut(3)
        .enumerate()
        .for_each(|(idx, pix)| {
            let base = idx * 3;
            let r =
                src_raw[base] as f32 + (cyber_raw[base] as f32 - src_raw[base] as f32) * intensity;
            let g = src_raw[base + 1] as f32
                + (cyber_raw[base + 1] as f32 - src_raw[base + 1] as f32) * intensity;
            let b = src_raw[base + 2] as f32
                + (cyber_raw[base + 2] as f32 - src_raw[base + 2] as f32) * intensity;

            pix[0] = clamp_to_u8(r);
            pix[1] = clamp_to_u8(g);
            pix[2] = clamp_to_u8(b);
        });

    ImageBuffer::from_raw(width, height, out_buf).expect("buffer length matches image")
}
