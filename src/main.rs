mod utils;
mod tests;
use clap::Parser;
use std::env;

use crate::utils::utils::convert;

#[derive(Parser, Debug)]
#[command(name = "AdventureSmartCLI")]
struct Args {
    #[arg(short, long)]
    file: String,
    #[arg(short, long)]
    dataset: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let dir = env::current_dir().expect("Не удалось получить текущую директорию").join("output");
    let path = dir.to_str().expect("Путь содержит недопустимые символы");

    convert(&args.file, path, &args.dataset).map_err(|e| e.to_string())?;

    Ok(())
}
