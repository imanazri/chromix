use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "chromix")]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert a hex color to OKLCH
    Convert(ConvertArgs),
    /// Generate an 11-step OKLCH color scale
    Scale(ScaleArgs),
    /// Export color tokens to files
    Export(ExportArgs),
}

#[derive(Args)]
pub struct ColorArg {
    /// Base color as hex (#rgb, #rrggbb, #rrggbbaa; # optional)
    #[arg(value_name = "COLOR")]
    pub color: String,
}

#[derive(Args)]
pub struct ConvertArgs {
    #[command(flatten)]
    pub color_arg: ColorArg,

    /// Copy the OKLCH value to the system clipboard
    #[arg(short, long)]
    pub copy: bool,
}

#[derive(Args)]
pub struct ScaleArgs {
    #[command(flatten)]
    pub color_arg: ColorArg,

    /// Comma-separated list of scale steps (default: Tailwind scale)
    #[arg(short, long, value_name = "L", value_delimiter = ',')]
    pub steps: Option<Vec<u16>>,

    /// Name for the color scale
    #[arg(short, long, default_value = "primary")]
    pub name: String,
}

#[derive(Args)]
pub struct ExportArgs {
    #[command(flatten)]
    pub color_arg: ColorArg,

    /// Steps for the scale
    #[arg(short, long, value_name = "L", value_delimiter = ',')]
    pub steps: Option<Vec<u16>>,

    /// Name for the color scale
    #[arg(short, long, default_value = "primary")]
    pub name: String,

    /// Export as colors.json
    #[arg(long)]
    pub json: bool,

    /// Export as Tailwind CSS config
    #[arg(long)]
    pub tailwind: bool,

    /// Export as CSS custom properties
    #[arg(long)]
    pub css: bool,

    /// Output directory
    #[arg(short, long, default_value = ".")]
    pub out: String,
}

pub fn is_hex_value(word: &str) -> bool {
    let inner = word.trim_matches('"');
    let inner = inner.strip_prefix('#').unwrap_or(inner);
    matches!(inner.len(), 3 | 6 | 8) && inner.chars().all(|c| c.is_ascii_hexdigit())
}