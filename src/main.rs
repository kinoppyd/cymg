use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process;

use image::{ImageBuffer, RgbImage};
use rayon::prelude::*;

#[derive(Clone, Copy)]
enum ColorBoost {
    None,
    Red,
    Green,
    Blue,
}

#[derive(Clone, Copy)]
struct EffectSettings {
    intensity: f32,
    glitch_shift: i32,
    glitch_rate: f32,
    scanline: f32,
    scanline_step: u32,
    vignette: f32,
    brightness: f32,
    contrast: f32,
    saturation: f32,
    noise: f32,
    bloom: f32,
    seed: u64,
    color_boost: ColorBoost,
}

impl Default for EffectSettings {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            glitch_shift: 9,
            glitch_rate: 0.18,
            scanline: 0.62,
            scanline_step: 2,
            vignette: 0.45,
            brightness: 0.0,
            contrast: 1.08,
            saturation: 1.15,
            noise: 0.08,
            bloom: 0.20,
            seed: 2_077,
            color_boost: ColorBoost::None,
        }
    }
}

struct CliArgs {
    input_path: PathBuf,
    settings: EffectSettings,
}

enum ParseOutcome {
    Run(CliArgs),
    HelpShown,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = match parse_args()? {
        ParseOutcome::Run(args) => args,
        ParseOutcome::HelpShown => return Ok(()),
    };

    let output_path = build_output_path(&args.input_path)?;
    let src = image::open(&args.input_path)?.to_rgb8();
    let dst = apply_cyberpunk_effect(&src, args.settings);

    dst.save(&output_path)?;
    println!("{}", output_path.display());

    Ok(())
}

fn parse_args() -> Result<ParseOutcome, Box<dyn Error>> {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_else(|| OsString::from("cymg"));
    let bin = Path::new(&program).display().to_string();
    let usage = format!("usage: {bin} [options] <image-path>\ntry: {bin} --help");

    let mut input_path: Option<PathBuf> = None;
    let mut settings = EffectSettings::default();

    while let Some(arg) = args.next() {
        let arg = arg.to_string_lossy().into_owned();

        match arg.as_str() {
            "-h" | "--help" => {
                print_help(&bin);
                return Ok(ParseOutcome::HelpShown);
            }
            "--red" => {
                if !matches!(settings.color_boost, ColorBoost::None) {
                    return Err(format!("only one color flag can be specified\n{usage}").into());
                }
                settings.color_boost = ColorBoost::Red;
            }
            "--green" => {
                if !matches!(settings.color_boost, ColorBoost::None) {
                    return Err(format!("only one color flag can be specified\n{usage}").into());
                }
                settings.color_boost = ColorBoost::Green;
            }
            "--blue" => {
                if !matches!(settings.color_boost, ColorBoost::None) {
                    return Err(format!("only one color flag can be specified\n{usage}").into());
                }
                settings.color_boost = ColorBoost::Blue;
            }
            "--intensity" => {
                settings.intensity = parse_f32_arg(&mut args, "--intensity", 0.0, 2.0, &usage)?;
            }
            "--glitch-shift" => {
                settings.glitch_shift = parse_i32_arg(&mut args, "--glitch-shift", 0, 128, &usage)?;
            }
            "--glitch-rate" => {
                settings.glitch_rate = parse_f32_arg(&mut args, "--glitch-rate", 0.0, 1.0, &usage)?;
            }
            "--scanline" => {
                settings.scanline = parse_f32_arg(&mut args, "--scanline", 0.0, 1.0, &usage)?;
            }
            "--scanline-step" => {
                settings.scanline_step =
                    parse_u32_arg(&mut args, "--scanline-step", 1, 32, &usage)?;
            }
            "--vignette" => {
                settings.vignette = parse_f32_arg(&mut args, "--vignette", 0.0, 1.0, &usage)?;
            }
            "--brightness" => {
                settings.brightness = parse_f32_arg(&mut args, "--brightness", -1.0, 1.0, &usage)?;
            }
            "--contrast" => {
                settings.contrast = parse_f32_arg(&mut args, "--contrast", 0.0, 3.0, &usage)?;
            }
            "--saturation" => {
                settings.saturation = parse_f32_arg(&mut args, "--saturation", 0.0, 3.0, &usage)?;
            }
            "--noise" => {
                settings.noise = parse_f32_arg(&mut args, "--noise", 0.0, 1.0, &usage)?;
            }
            "--bloom" => {
                settings.bloom = parse_f32_arg(&mut args, "--bloom", 0.0, 1.0, &usage)?;
            }
            "--seed" => {
                settings.seed = parse_u64_arg(&mut args, "--seed", &usage)?;
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown option: {arg}\n{usage}").into());
            }
            _ => {
                if input_path.is_some() {
                    return Err(usage.into());
                }
                input_path = Some(PathBuf::from(arg));
            }
        }
    }

    let input_path = input_path.ok_or(usage)?;
    Ok(ParseOutcome::Run(CliArgs {
        input_path,
        settings,
    }))
}

fn parse_f32_arg(
    args: &mut env::ArgsOs,
    flag: &str,
    min: f32,
    max: f32,
    usage: &str,
) -> Result<f32, Box<dyn Error>> {
    let value = next_arg_value(args, flag, usage)?;
    let parsed = value
        .parse::<f32>()
        .map_err(|_| format!("invalid value for {flag}: {value}\n{usage}"))?;
    if parsed < min || parsed > max {
        return Err(
            format!("value out of range for {flag}: {value} (expected {min}..{max})").into(),
        );
    }
    Ok(parsed)
}

fn parse_i32_arg(
    args: &mut env::ArgsOs,
    flag: &str,
    min: i32,
    max: i32,
    usage: &str,
) -> Result<i32, Box<dyn Error>> {
    let value = next_arg_value(args, flag, usage)?;
    let parsed = value
        .parse::<i32>()
        .map_err(|_| format!("invalid value for {flag}: {value}\n{usage}"))?;
    if parsed < min || parsed > max {
        return Err(
            format!("value out of range for {flag}: {value} (expected {min}..{max})").into(),
        );
    }
    Ok(parsed)
}

fn parse_u32_arg(
    args: &mut env::ArgsOs,
    flag: &str,
    min: u32,
    max: u32,
    usage: &str,
) -> Result<u32, Box<dyn Error>> {
    let value = next_arg_value(args, flag, usage)?;
    let parsed = value
        .parse::<u32>()
        .map_err(|_| format!("invalid value for {flag}: {value}\n{usage}"))?;
    if parsed < min || parsed > max {
        return Err(
            format!("value out of range for {flag}: {value} (expected {min}..{max})").into(),
        );
    }
    Ok(parsed)
}

fn parse_u64_arg(args: &mut env::ArgsOs, flag: &str, usage: &str) -> Result<u64, Box<dyn Error>> {
    let value = next_arg_value(args, flag, usage)?;
    let parsed = value
        .parse::<u64>()
        .map_err(|_| format!("invalid value for {flag}: {value}\n{usage}"))?;
    Ok(parsed)
}

fn next_arg_value(
    args: &mut env::ArgsOs,
    flag: &str,
    usage: &str,
) -> Result<String, Box<dyn Error>> {
    let value = args
        .next()
        .ok_or_else(|| format!("missing value for {flag}\n{usage}"))?;
    Ok(value.to_string_lossy().into_owned())
}

fn print_help(bin: &str) {
    println!("cymg - Cyber image CLI");
    println!();
    println!("Usage:");
    println!("  {bin} [options] <image-path>");
    println!();
    println!("Color boost:");
    println!("  --red                 Boost red tone");
    println!("  --green               Boost green tone");
    println!("  --blue                Boost blue tone");
    println!();
    println!("Effect options:");
    println!("  --intensity <0.0-2.0>      Overall effect strength (default: 1.0)");
    println!("  --glitch-shift <0-128>     RGB shift size in pixels (default: 9)");
    println!("  --glitch-rate <0.0-1.0>    Chance of strong glitch rows (default: 0.18)");
    println!("  --scanline <0.0-1.0>       Scanline strength (default: 0.62)");
    println!("  --scanline-step <1-32>     Scanline interval in rows (default: 2)");
    println!("  --vignette <0.0-1.0>       Vignette amount (default: 0.45)");
    println!("  --brightness <-1.0-1.0>    Brightness shift (default: 0.0)");
    println!("  --contrast <0.0-3.0>       Contrast gain (default: 1.08)");
    println!("  --saturation <0.0-3.0>     Saturation gain (default: 1.15)");
    println!("  --noise <0.0-1.0>          Film/noise amount (default: 0.08)");
    println!("  --bloom <0.0-1.0>          Neon bloom amount (default: 0.20)");
    println!("  --seed <u64>               Seed for deterministic glitch/noise (default: 2077)");
    println!("  -h, --help                 Show this help");
    println!();
    println!("Output:");
    println!("  Saves to: <input-stem>.cyber.<input-ext> in the same directory.");
    println!();
    println!("Examples:");
    println!("  {bin} input.jpg");
    println!("  {bin} --red --intensity 1.35 --glitch-shift 14 --glitch-rate 0.28 input.jpg");
    println!("  {bin} --blue --scanline 0.35 --scanline-step 2 --vignette 0.6 input.png");
    println!(
        "  {bin} --brightness -0.10 --contrast 1.25 --saturation 1.4 --noise 0.2 --bloom 0.35 --seed 42 input.webp"
    );
}

fn build_output_path(input_path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let stem = input_path
        .file_stem()
        .ok_or("input image must have a file name")?;
    let ext = input_path
        .extension()
        .ok_or("input image must have an extension")?;

    let mut output_name = OsString::new();
    output_name.push(stem);
    output_name.push(".cyber.");
    output_name.push(ext);

    let parent = input_path.parent().unwrap_or(Path::new("."));
    Ok(parent.join(output_name))
}

fn apply_cyberpunk_effect(src: &RgbImage, settings: EffectSettings) -> RgbImage {
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
            let y_u32 = y as u32;
            let y_i32 = y as i32;
            let row_glitch = random_unit(settings.seed, 991, y as u64, 3) < settings.glitch_rate;

            let base_red_shift = random_shift(settings.seed, y as u64, settings.glitch_shift, 17);
            let base_blue_shift = random_shift(settings.seed, y as u64, settings.glitch_shift, 23);
            let row_boost = if row_glitch { settings.glitch_shift } else { 0 };
            let red_shift = base_red_shift + row_boost;
            let blue_shift = base_blue_shift - row_boost;

            for x in 0..width {
                let x_i32 = x as i32;
                let src_r = sample_rgb(src, x_i32 + red_shift, y_i32);
                let src_g = sample_rgb(src, x_i32, y_i32);
                let src_b = sample_rgb(src, x_i32 + blue_shift, y_i32);

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

                if settings.scanline > 0.0 {
                    let depth = settings.scanline.clamp(0.0, 1.0);
                    let phase = y_u32 % settings.scanline_step;
                    let mut scanline_factor = if phase == 0 {
                        1.0 - depth * 1.10
                    } else if phase == 1 {
                        1.0 + depth * 0.98
                    } else {
                        1.0 + depth * 0.35
                    };

                    // Slight aperture-style variation makes scanlines stand out on photo textures.
                    let aperture = if x % 3 == 0 {
                        1.0 + depth * 0.14
                    } else {
                        1.0 - depth * 0.07
                    };
                    scanline_factor *= aperture;

                    if phase == 0 {
                        // Dark rows: stronger attenuation + subtle magenta tint.
                        r *= scanline_factor * (1.0 + depth * 0.05);
                        g *= scanline_factor * (1.0 - depth * 0.06);
                        b *= scanline_factor * (1.0 + depth * 0.07);
                        r -= depth * 18.0;
                        g -= depth * 24.0;
                        b -= depth * 16.0;
                    } else if phase == 1 {
                        // Bright rows: lift highlights so scanlines remain obvious.
                        r *= scanline_factor * (1.0 + depth * 0.10);
                        g *= scanline_factor * (1.0 + depth * 0.04);
                        b *= scanline_factor * (1.0 + depth * 0.13);
                        r += depth * 12.0;
                        g += depth * 10.0;
                        b += depth * 15.0;
                    } else {
                        r *= scanline_factor;
                        g *= scanline_factor;
                        b *= scanline_factor;
                    }
                }

                if settings.vignette > 0.0 {
                    let nx = if width > 1 {
                        (x as f32 / (width - 1) as f32) * 2.0 - 1.0
                    } else {
                        0.0
                    };
                    let ny = if height > 1 {
                        (y_u32 as f32 / (height - 1) as f32) * 2.0 - 1.0
                    } else {
                        0.0
                    };
                    let dist = ((nx * nx + ny * ny).sqrt() / 1.414_213_5).min(1.0);
                    let vignette_factor = 1.0 - dist.powf(1.8) * settings.vignette;
                    r *= vignette_factor;
                    g *= vignette_factor * 0.97;
                    b *= vignette_factor * 1.03;
                }

                if row_glitch && y_u32 % 7 == 0 {
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

                if settings.noise > 0.0 {
                    let noise_amp = settings.noise * 36.0;
                    r += (random_unit(settings.seed, x as u64, y as u64, 101) * 2.0 - 1.0)
                        * noise_amp;
                    g += (random_unit(settings.seed, x as u64, y as u64, 211) * 2.0 - 1.0)
                        * noise_amp;
                    b += (random_unit(settings.seed, x as u64, y as u64, 307) * 2.0 - 1.0)
                        * noise_amp;
                }

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

fn color_gains(boost: ColorBoost) -> (f32, f32, f32) {
    match boost {
        ColorBoost::None => (1.0, 1.0, 1.0),
        ColorBoost::Red => (1.28, 0.94, 0.94),
        ColorBoost::Green => (1.05, 1.85, 1.08),
        ColorBoost::Blue => (1.05, 1.12, 1.95),
    }
}

fn apply_color_overdrive(r: &mut f32, g: &mut f32, b: &mut f32, boost: ColorBoost) {
    match boost {
        ColorBoost::None | ColorBoost::Red => {}
        ColorBoost::Green => {
            *r = *r * 1.10 + 18.0;
            *g = *g * 1.58 + 44.0;
            *b = *b * 1.16 + 20.0;

            let lum = *r * 0.2126 + *g * 0.7152 + *b * 0.0722;
            let shoulder = ((lum - 120.0) / 110.0).clamp(0.0, 1.0);
            *r += shoulder * 20.0;
            *g += shoulder * 56.0;
            *b += shoulder * 26.0;
        }
        ColorBoost::Blue => {
            *r = *r * 1.10 + 16.0;
            *g = *g * 1.18 + 22.0;
            *b = *b * 1.62 + 48.0;

            let lum = *r * 0.2126 + *g * 0.7152 + *b * 0.0722;
            let shoulder = ((lum - 118.0) / 105.0).clamp(0.0, 1.0);
            *r += shoulder * 18.0;
            *g += shoulder * 24.0;
            *b += shoulder * 62.0;
        }
    }
}

fn apply_matrix_tone(r: &mut f32, g: &mut f32, b: &mut f32, boost: ColorBoost) {
    let lum = *r * 0.2126 + *g * 0.7152 + *b * 0.0722;
    let (strength, tint_r, tint_g, tint_b, lift) = match boost {
        ColorBoost::Green => (0.44, 0.33, 1.30, 0.64, 19.0),
        ColorBoost::Blue => (0.44, 0.36, 0.98, 1.32, 19.0),
        ColorBoost::None | ColorBoost::Red => (0.22, 0.36, 1.12, 0.70, 9.0),
    };

    let target_r = lum * tint_r + lift * 0.25;
    let target_g = lum * tint_g + lift;
    let target_b = lum * tint_b + lift * 0.45;

    *r = *r * (1.0 - strength) + target_r * strength;
    *g = *g * (1.0 - strength) + target_g * strength;
    *b = *b * (1.0 - strength) + target_b * strength;

    let shadow = ((112.0 - lum) / 112.0).clamp(0.0, 1.0) * strength;
    *r *= 1.0 - shadow * 0.25;
    *g *= 1.0 + shadow * 0.18;
    *b *= 1.0 + shadow * 0.08;
}

fn apply_highlight_recovery(r: &mut f32, g: &mut f32, b: &mut f32, boost: ColorBoost) {
    let max_c = (*r).max(*g).max(*b);
    let min_c = (*r).min(*g).min(*b);
    let lum = *r * 0.2126 + *g * 0.7152 + *b * 0.0722;
    let clipping = ((max_c - 225.0) / 60.0).clamp(0.0, 1.0);
    if clipping <= 0.0 {
        return;
    }

    // White-ish highlights need stronger color restoration than already saturated highlights.
    let whiteness = (1.0 - ((max_c - min_c) / 90.0).clamp(0.0, 1.0)).powf(0.75);
    let amount = (clipping * (0.70 + 0.30 * whiteness)).clamp(0.0, 1.0);
    let compress = 1.0 - amount * 0.55;

    compress_highlights(r, 215.0, compress);
    compress_highlights(g, 215.0, compress);
    compress_highlights(b, 215.0, compress);

    let (tint_r, tint_g, tint_b, offset) = match boost {
        ColorBoost::Green => (0.42, 1.00, 0.68, 18.0),
        ColorBoost::Blue => (0.46, 0.90, 1.00, 18.0),
        ColorBoost::None | ColorBoost::Red => (0.44, 0.98, 0.74, 14.0),
    };
    let tint_rv = lum * tint_r + offset * 0.32;
    let tint_gv = lum * tint_g + offset;
    let tint_bv = lum * tint_b + offset * 0.55;

    *r = *r * (1.0 - amount) + tint_rv * amount;
    *g = *g * (1.0 - amount) + tint_gv * amount;
    *b = *b * (1.0 - amount) + tint_bv * amount;
}

fn compress_highlights(channel: &mut f32, start: f32, factor: f32) {
    if *channel <= start {
        return;
    }
    *channel = start + (*channel - start) * factor;
}

fn apply_saturation(r: &mut f32, g: &mut f32, b: &mut f32, saturation: f32) {
    let lum = *r * 0.2126 + *g * 0.7152 + *b * 0.0722;
    *r = lum + (*r - lum) * saturation;
    *g = lum + (*g - lum) * saturation;
    *b = lum + (*b - lum) * saturation;
}

fn apply_bloom(src: &RgbImage, bloom: f32) -> RgbImage {
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

fn random_shift(seed: u64, y: u64, max_shift: i32, salt: u64) -> i32 {
    if max_shift <= 0 {
        return 0;
    }
    let span = (max_shift as u64) * 2 + 1;
    let n = hash64(seed ^ (y.wrapping_mul(0x9E37_79B9_7F4A_7C15)) ^ salt) % span;
    n as i32 - max_shift
}

fn random_unit(seed: u64, x: u64, y: u64, salt: u64) -> f32 {
    let bits = hash64(
        seed ^ (x.wrapping_mul(0x9E37_79B9_7F4A_7C15))
            ^ (y.wrapping_mul(0xBF58_476D_1CE4_E5B9))
            ^ salt,
    );
    let value = (bits >> 40) as u32;
    value as f32 / 16_777_215.0
}

fn hash64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn sample_rgb(img: &RgbImage, x: i32, y: i32) -> [u8; 3] {
    let max_x = img.width().saturating_sub(1) as i32;
    let max_y = img.height().saturating_sub(1) as i32;

    let xx = x.clamp(0, max_x) as u32;
    let yy = y.clamp(0, max_y) as u32;
    img.get_pixel(xx, yy).0
}

fn clamp_to_u8(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}
