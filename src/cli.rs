use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(crate) enum ColorBoost {
    None,
    Red,
    Green,
    Blue,
}

#[derive(Clone, Copy)]
pub(crate) struct EffectSettings {
    pub(crate) intensity: f32,
    pub(crate) glitch_shift: i32,
    pub(crate) glitch_rate: f32,
    pub(crate) scanline: f32,
    pub(crate) scanline_step: u32,
    pub(crate) vignette: f32,
    pub(crate) brightness: f32,
    pub(crate) contrast: f32,
    pub(crate) saturation: f32,
    pub(crate) noise: f32,
    pub(crate) bloom: f32,
    pub(crate) seed: u64,
    pub(crate) color_boost: ColorBoost,
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

pub(crate) struct CliArgs {
    pub(crate) input_path: PathBuf,
    pub(crate) settings: EffectSettings,
    pub(crate) random_mode: bool,
}

pub(crate) enum ParseOutcome {
    Run(CliArgs),
    HelpShown,
}

pub(crate) fn parse_args() -> Result<ParseOutcome, Box<dyn Error>> {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_else(|| OsString::from("cymg"));
    let bin = Path::new(&program).display().to_string();
    let usage = format!("usage: {bin} [options] <image-path>\ntry: {bin} --help");

    let mut input_path: Option<PathBuf> = None;
    let mut settings = EffectSettings::default();
    let mut random_mode = false;

    while let Some(arg) = args.next() {
        let arg = arg.to_string_lossy().into_owned();

        match arg.as_str() {
            "-h" | "--help" => {
                print_help(&bin);
                return Ok(ParseOutcome::HelpShown);
            }
            "--red" => set_color_boost(&mut settings, ColorBoost::Red, &usage)?,
            "--green" => set_color_boost(&mut settings, ColorBoost::Green, &usage)?,
            "--blue" => set_color_boost(&mut settings, ColorBoost::Blue, &usage)?,
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
            "--random" => {
                random_mode = true;
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
        random_mode,
    }))
}

fn set_color_boost(
    settings: &mut EffectSettings,
    boost: ColorBoost,
    usage: &str,
) -> Result<(), Box<dyn Error>> {
    if !matches!(settings.color_boost, ColorBoost::None) {
        return Err(format!("only one color flag can be specified\n{usage}").into());
    }

    settings.color_boost = boost;
    Ok(())
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
    println!(
        "  --random                   Generate 10 randomized variants (keeps selected color mode)"
    );
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
    println!("  {bin} --green --random input.jpg");
}
