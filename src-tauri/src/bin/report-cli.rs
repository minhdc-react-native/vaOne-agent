use anyhow::{Context, Result};
use std::{env, fs};

use vaone_plugin::pdf::models::PdfTemplate;
use vaone_plugin::pdf::renderer::render_page;

fn main() -> Result<()> {
    // report-cli config.json output.pdf

    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage:");
        eprintln!("report-cli <report.json> <output.pdf>");
        std::process::exit(1);
    }

    let report_path = &args[1];
    let data_path = &args[2];
    let output_path = &args[3];

    let json_report =
        fs::read_to_string(report_path).with_context(|| format!("Cannot read {}", report_path))?;

    let json_data =
        fs::read_to_string(data_path).with_context(|| format!("Cannot read {}", data_path))?;

    let doc: PdfTemplate = serde_json::from_str(&json_report).context("Invalid report json")?;

    let data: serde_json::Value = serde_json::from_str(&json_data).context("Invalid data json")?;

    render_page(vec![doc], vec![data], output_path)?;

    println!("Render success: {}", output_path);

    Ok(())
}
