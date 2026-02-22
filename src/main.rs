mod cli;
mod effects;
mod output;

use std::error::Error;
use std::process;

use cli::{ParseOutcome, parse_args};
use effects::apply_cyberpunk_effect;
use output::build_output_path;

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
