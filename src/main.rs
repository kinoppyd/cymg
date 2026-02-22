mod cli;
mod effects;
mod output;

use std::cmp;
use std::error::Error;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use cli::{CliArgs, EffectSettings, ParseOutcome, parse_args};
use effects::apply_cyberpunk_effect;
use image::RgbImage;
use output::{build_output_path, build_output_path_with_index};

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

    let src = image::open(&args.input_path)?.to_rgb8();
    if args.random_mode {
        run_random_batch(&args, &src)?;
        return Ok(());
    }

    let output_path = build_output_path(&args.input_path)?;
    let dst = apply_cyberpunk_effect(&src, args.settings);
    dst.save(&output_path)?;
    println!("{}", output_path.display());

    Ok(())
}

fn run_random_batch(args: &CliArgs, src: &RgbImage) -> Result<(), Box<dyn Error>> {
    let mut rng = SplitMix64::new(seed_from_time() ^ args.settings.seed ^ (src.width() as u64));
    for index in 0..10 {
        let randomized = randomize_settings(args.settings, &mut rng);
        let dst = apply_cyberpunk_effect(src, randomized);
        let output_path = build_output_path_with_index(&args.input_path, index)?;
        dst.save(&output_path)?;
        println!("{}", output_path.display());
    }

    Ok(())
}

fn randomize_settings(base: EffectSettings, rng: &mut SplitMix64) -> EffectSettings {
    EffectSettings {
        intensity: rng.f32_range(0.65, 1.85),
        glitch_shift: rng.i32_range_inclusive(2, 30),
        glitch_rate: rng.f32_range(0.06, 0.52),
        scanline: rng.f32_range(0.22, 0.92),
        scanline_step: rng.u32_range_inclusive(1, 6),
        vignette: rng.f32_range(0.20, 0.75),
        brightness: rng.f32_range(-0.16, 0.18),
        contrast: rng.f32_range(0.95, 1.70),
        saturation: rng.f32_range(0.90, 2.15),
        noise: rng.f32_range(0.02, 0.32),
        bloom: rng.f32_range(0.05, 0.62),
        seed: rng.next_u64(),
        // Keep the initially selected color mode fixed, as requested.
        color_boost: base.color_boost,
    }
}

fn seed_from_time() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_nanos() as u64,
        Err(_) => 0xA5A5_5A5A_D3C1_F00D,
    }
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_f32_0_1(&mut self) -> f32 {
        let v = (self.next_u64() >> 40) as u32;
        v as f32 / 16_777_215.0
    }

    fn f32_range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32_0_1()
    }

    fn i32_range_inclusive(&mut self, min: i32, max: i32) -> i32 {
        let span = cmp::max(1, max - min + 1) as u64;
        min + (self.next_u64() % span) as i32
    }

    fn u32_range_inclusive(&mut self, min: u32, max: u32) -> u32 {
        let span = max.saturating_sub(min).saturating_add(1) as u64;
        min + (self.next_u64() % span) as u32
    }
}
