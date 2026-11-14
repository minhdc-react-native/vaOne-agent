use anyhow::Result;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage:");
        eprintln!("report <report.json> <data.json> <output.pdf>");
        std::process::exit(1);
    }

    report_cli::render(&args[1], &args[2], &args[3])?;

    println!("Render success");

    Ok(())
}
