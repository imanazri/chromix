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
    // Gray sweep, brightest at the top fading to darker at the bottom.
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
            "\x1b[38;2;30;30;30m".to_string(),     // syn_cmd
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

/// Draw the gradient bar in place: BAR_HEIGHT rows, each BAR_WIDTH cells wide,
/// indented by MARGIN, with a hue rotation applied across the bar.
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
    let (subtle, dim, rule, _syn_cmd, _syn_digit) = body_colors(theme);
    let is_tty = atty::is(atty::Stream::Stdout);

    // Margin
    println!();

    // Wordmark - 5 rows for CHROMIX
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
        println!("{}{}\x1b[38;2;{};{};{}m{}\x1b[0m", MARGIN, BOLD, rgb.0, rgb.1, rgb.2, row);
    }

    // Blank
    println!();

    // Gradient bar (fixed BAR_WIDTH x BAR_HEIGHT, indented by MARGIN)
    if is_tty {
        // Animation loop
        for frame in 0..=FRAMES {
            let t = frame as f64 / FRAMES as f64;
            let eased = 1.0 - (1.0 - t).powi(3);
            let hue_offset = (1.0 - eased) * 360.0;

            draw_gradient_bar(hue_offset);
            io::stdout().flush().ok();

            thread::sleep(Duration::from_millis(28));

            // Move cursor back up to redraw the bar in place (except after the last frame)
            if frame < FRAMES {
                print!("\x1b[{}A", BAR_HEIGHT);
            }
        }
    } else {
        // Static draw
        draw_gradient_bar(0.0);
    }

    // Blank
    println!();

    // Description lines
    println!("{}OKLCH color tool right in your terminal.{}", MARGIN, dim);
    println!("{}Generate OKLCH color scale and convert hex to OKLCH.{}", MARGIN, dim);

    // Rule
    print!("{}{}", MARGIN, rule);
    for _ in 0..48 {
        print!("─");
    }
    println!("\x1b[0m");

    // HOW TO USE
    println!("{}{}{} ", MARGIN, subtle, BOLD);
    println!();

    // Examples
    let examples = [
        "chromix convert 3b82f6 # convert a color to OKLCH",
        "chromix scale 3b82f6  # print an 11-step OKLCH ramp",
        "chromix export 3b82f6 --json --css  # write the selected token files",
    ];

    for ex in &examples {
        let mut line = String::new();
        let mut current_word = String::new();

        for ch in ex.chars() {
            if ch == ' ' {
                if !current_word.is_empty() {
                    if current_word == "chromix" || matches!(current_word.as_str(), "convert" | "scale" | "export") {
                        line.push_str(SYN_SUB);
                        line.push_str(&current_word);
                        line.push_str(RESET);
                    } else if current_word.starts_with("--") {
                        line.push_str(SYN_FLAG);
                        line.push_str(&current_word);
                        line.push_str(RESET);
                    } else if is_hex_value(&current_word) {
                        line.push_str(SYN_STR);
                        line.push_str(&current_word);
                        line.push_str(RESET);
                    } else {
                        line.push_str(&current_word);
                    }
                    line.push(' ');
                    current_word.clear();
                } else {
                    line.push(ch);
                }
            } else if ch == '#' {
                // Start of comment - output remaining as-is
                line.push(ch);
                // Continue reading the rest of the comment
                break;
            } else {
                current_word.push(ch);
            }
        }

        // Handle any remaining word (for non-comment parts)
        if !current_word.is_empty() {
            if current_word == "chromix" || matches!(current_word.as_str(), "convert" | "scale" | "export") {
                line.push_str(SYN_SUB);
                line.push_str(&current_word);
                line.push_str(RESET);
            } else if current_word.starts_with("--") {
                line.push_str(SYN_FLAG);
                line.push_str(&current_word);
                line.push_str(RESET);
            } else if is_hex_value(&current_word) {
                line.push_str(SYN_STR);
                line.push_str(&current_word);
                line.push_str(RESET);
            } else {
                line.push_str(&current_word);
            }
        }

        println!("{} {}", MARGIN, line);
    }

    println!();

    // Footer
    println!("{}Run `chromix --help` for all options.{}", MARGIN, dim);
    println!();
}