use chromix_core::Oklch;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

const MARGIN: &str = "  ";
const BAR_WIDTH: usize = 55;
const BAR_HEIGHT: usize = 2;
const FRAMES: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Theme {
    Dark,
    Light,
}

fn get_theme() -> Theme {
    match terminal_light::luma() {
        Ok(luma) if luma > 0.5 => Theme::Light,
        _ => Theme::Dark,
    }
}

fn wordmark_colors(theme: Theme) -> [(u8, u8, u8); 5] {
    match theme {
        Theme::Dark => [
            (235, 235, 235),
            (200, 200, 200),
            (165, 165, 165),
            (130, 130, 130),
            (95, 95, 95),
        ],
        Theme::Light => [
            (60, 60, 60),
            (95, 95, 95),
            (130, 130, 130),
            (165, 165, 165),
            (200, 200, 200),
        ],
    }
}

fn body_colors(theme: Theme) -> (String, String, String, String, String) {
    match theme {
        Theme::Dark => (
            "\x1b[38;2;170;170;170m".to_string(), // subtle
            "\x1b[38;2;120;120;120m".to_string(), // dim
            "\x1b[38;2;80;80;80m".to_string(),    // rule
            "\x1b[38;2;235;235;235m".to_string(), // syn_cmd
            "\x1b[38;2;120;120;120m".to_string(), // syn_digit
        ),
        Theme::Light => (
            "\x1b[38;2;80;80;80m".to_string(),    // subtle
            "\x1b[38;2;120;120;120m".to_string(), // dim
            "\x1b[38;2;175;175;175m".to_string(), // rule
            "\x1b[38;2;30;30;30m".to_string(),    // syn_cmd
            "\x1b[38;2;120;120;120m".to_string(), // syn_digit
        ),
    }
}

const SYN_SUB: &str = "\x1b[38;2;198;160;246m"; // soft purple
const SYN_STR: &str = "\x1b[38;2;126;196;145m"; // soft green
const SYN_FLAG: &str = "\x1b[38;2;138;170;240m"; // soft blue
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

fn is_hex_value(word: &str) -> bool {
    let inner = word.trim_matches('"');
    let inner = inner.strip_prefix('#').unwrap_or(inner);
    matches!(inner.len(), 3 | 6 | 8) && inner.chars().all(|c| c.is_ascii_hexdigit())
}

fn draw_gradient_bar(hue_offset: f64) {
    for _ in 0..BAR_HEIGHT {
        print!("{}", MARGIN);
        for x in 0..BAR_WIDTH {
            let h = ((x as f64 / (BAR_WIDTH - 1) as f64) * 360.0 + hue_offset) % 360.0;
            let oklch = Oklch::new(0.70, 0.16, h);
            let srgb = oklch.to_srgb();
            print!("\x1b[48;2;{};{};{}m ", srgb.r, srgb.g, srgb.b);
        }
        println!("\x1b[0m");
    }
}

pub fn show_splash() {
    let theme = get_theme();
    let (subtle, dim, rule, syn_cmd, _syn_digit) = body_colors(theme);
    let is_tty = atty::is(atty::Stream::Stdout);

    println!();

    let colors = wordmark_colors(theme);
    let wordmark_rows = [
        " ██████╗██╗  ██╗██████╗  ██████╗ ███╗   ███╗██╗██╗  ██╗",
        "██╔════╝██║  ██║██╔══██╗██╔═══██╗████╗ ████║██║╚██╗██╔╝",
        "██║     ███████║██████╔╝██║   ██║██╔████╔██║██║ ╚███╔╝ ",
        "██║     ██╔══██║██╔══██╗██║   ██║██║╚██╔╝██║██║ ██╔██╗ ",
        "╚██████╗██║  ██║██║  ██║╚██████╔╝██║ ╚═╝ ██║██║██╔╝ ██╗",
    ];

    for (i, row) in wordmark_rows.iter().enumerate() {
        let rgb = colors[i % colors.len()];
        println!(
            "{}{}\x1b[38;2;{};{};{}m{}\x1b[0m",
            MARGIN, BOLD, rgb.0, rgb.1, rgb.2, row
        );
    }

    println!();

    if is_tty {
        for frame in 0..=FRAMES {
            let t = frame as f64 / FRAMES as f64;
            let eased = 1.0 - (1.0 - t).powi(3);
            let hue_offset = (1.0 - eased) * 360.0;

            draw_gradient_bar(hue_offset);
            io::stdout().flush().ok();

            thread::sleep(Duration::from_millis(28));

            if frame < FRAMES {
                print!("\x1b[{}A", BAR_HEIGHT);
            }
        }
    } else {
        draw_gradient_bar(0.0);
    }

    println!();

    println!(
        "{}{}OKLCH color tool right in your terminal.{}",
        MARGIN, subtle, RESET
    );
    println!(
        "{}{}Generate OKLCH color scale and convert hex to OKLCH.{}",
        MARGIN, dim, RESET
    );
    println!();
    println!(
        "{}{}v{}  by Iman Nazri{}",
        MARGIN,
        dim,
        env!("CARGO_PKG_VERSION"),
        RESET
    );
    println!();

    print!("{}{}", MARGIN, rule);
    for _ in 0..48 {
        print!("─");
    }
    println!("\x1b[0m");

    println!("{}{}{}HOW TO USE{}", MARGIN, subtle, BOLD, RESET);
    println!();

    // Examples
    let examples = [
        "chromix convert 3b82f6 # convert a color to OKLCH",
        "chromix scale 3b82f6  # print an 11-step OKLCH ramp",
        "chromix export 3b82f6 --json --css  # write the selected token files",
        "chromix gradient 3b82f6  # blend a color into an OKLCH band",
    ];

    for ex in &examples {
        let (code, comment) = match ex.find('#') {
            Some(pos) => (&ex[..pos], Some(&ex[pos..])),
            None => (*ex, None),
        };

        let mut line = String::new();
        for word in code.split_whitespace() {
            if !line.is_empty() {
                line.push(' ');
            }
            if word == "chromix" {
                line.push_str(&syn_cmd);
                line.push_str(BOLD);
                line.push_str(word);
                line.push_str(RESET);
            } else if matches!(word, "convert" | "scale" | "export" | "gradient") {
                line.push_str(SYN_SUB);
                line.push_str(word);
                line.push_str(RESET);
            } else if word.starts_with("--") {
                line.push_str(SYN_FLAG);
                line.push_str(word);
                line.push_str(RESET);
            } else if is_hex_value(word) {
                line.push_str(SYN_STR);
                line.push_str(word);
                line.push_str(RESET);
            } else {
                line.push_str(&dim);
                line.push_str(word);
                line.push_str(RESET);
            }
        }

        if let Some(c) = comment {
            line.push_str("  ");
            line.push_str(&dim);
            line.push_str(c);
            line.push_str(RESET);
        }

        println!("{} {}", MARGIN, line);
    }

    println!();

    // Footer
    println!(
        "{}{}Run `chromix --help` for all options.{}",
        MARGIN, dim, RESET
    );
    println!();
}
