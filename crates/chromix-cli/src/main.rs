use chromix_core::{generate_ramp, generate_ramp_with_steps, Oklch};
use chromix_cli::{cli, render, splash};
use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = cli::Cli::parse();

    match cli.command {
        None => {
            splash::show_splash();
            ExitCode::SUCCESS
        }
        Some(commands) => match commands {
            cli::Commands::Convert(args) => handle_convert(&args),
            cli::Commands::Scale(args) => handle_scale(&args),
            cli::Commands::Export(args) => handle_export(&args),
        },
    }
}

fn handle_convert(args: &cli::ConvertArgs) -> ExitCode {
    match Oklch::from_hex(&args.color_arg.color) {
        Ok(oklch) => {
            let copied = if args.copy {
                if copy_to_clipboard(&oklch.to_css()) {
                    println!();
                    println!("  copied to clipboard");
                    true
                } else {
                    eprintln!("  warning: failed to copy to clipboard");
                    false
                }
            } else {
                false
            };
            render::render_color(oklch, copied);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: invalid color '{}': {}", args.color_arg.color, e);
            ExitCode::FAILURE
        }
    }
}

fn handle_scale(args: &cli::ScaleArgs) -> ExitCode {
    match Oklch::from_hex(&args.color_arg.color) {
        Ok(base) => {
            let ramp = if let Some(steps) = &args.steps {
                if steps.is_empty() {
                    generate_ramp(base)
                } else {
                    generate_ramp_with_steps(base, steps)
                }
            } else {
                generate_ramp(base)
            };
            render::render_input_header(base);
            render::render_ramp(&args.name, &ramp);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: invalid color '{}': {}", args.color_arg.color, e);
            ExitCode::FAILURE
        }
    }
}

fn handle_export(args: &cli::ExportArgs) -> ExitCode {
    let has_format = args.json || args.tailwind || args.css;
    if !has_format {
        eprintln!("error: pick at least one format to export: --json, --tailwind, and/or --css");
        return ExitCode::FAILURE;
    }

    match Oklch::from_hex(&args.color_arg.color) {
        Ok(base) => {
            let ramp = if let Some(steps) = &args.steps {
                if steps.is_empty() {
                    generate_ramp(base)
                } else {
                    generate_ramp_with_steps(base, steps)
                }
            } else {
                generate_ramp(base)
            };

            render::render_input_header(base);
            render::render_ramp(&args.name, &ramp);

            match chromix_cli::export::export(
                args.json,
                args.tailwind,
                args.css,
                &args.name,
                &ramp,
                &args.out,
            ) {
                Ok(paths) => {
                    for path in &paths {
                        println!("wrote {}", path.display());
                    }
                    println!();
                    println!("✓ exported {} file(s) to {}", paths.len(), args.out);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: failed to write tokens: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        Err(e) => {
            eprintln!("error: invalid color '{}': {}", args.color_arg.color, e);
            ExitCode::FAILURE
        }
    }
}

fn copy_to_clipboard(content: &str) -> bool {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => clipboard.set_text(content.to_string()).is_ok(),
        Err(_) => false,
    }
}