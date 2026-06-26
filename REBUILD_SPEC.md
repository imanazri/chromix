# chromix — Rebuild Specsheet

A complete, implementation-ready specification for rebuilding **chromix**, a Rust
CLI that generates perceptually-uniform **OKLCH** color systems (tonal ramps +
design tokens) from a single base color. This document is written to be handed to
another AI/engineer to reproduce the tool from scratch. It describes *what* the
program does, the *exact math and formats*, and the *nuances* that are easy to get
wrong.

Target reproduction language: **Rust** (recommended — the math is float-heavy and
the original is Rust). The algorithms are language-agnostic, so a port to
TypeScript/Go/Python is viable; a "Porting notes" section covers that.

---

## 1. What the tool is

`chromix` is a terminal CLI. Given a hex color it:

1. Converts it to OKLCH (perceptually-uniform model).
2. Generates an 11-step tonal ramp (Tailwind-style `50…950`) that holds hue
   roughly constant and steps evenly in *perceived* lightness, with every step
   forced into the sRGB gamut.
3. Renders the ramp in the terminal as truecolor swatches.
4. Optionally exports the ramp as design tokens (JSON / Tailwind / CSS).

It also has a no-argument **splash screen**: a grayscale ASCII wordmark, an
animated OKLCH spectrum bar, and a syntax-highlighted "how to use" block.

The product thesis: HSL/sRGB ramps step unevenly in apparent brightness and drift
in hue; doing the same in OKLCH yields smoother, more professional ramps. That
perceptual quality is the entire value proposition — **the conversion math and the
lightness/chroma tuning are the product**, not the CLI plumbing.

---

## 2. Architecture

Two crates in a Cargo workspace. The split matters: the engine is pure and I/O-free
so it can later back a GUI or WASM front end. Preserve this boundary.

```
workspace/
├─ Cargo.toml                 # [workspace], resolver = "2"
├─ crates/
│  ├─ chromix-core/           # pure color engine — NO terminal/fs/clap deps
│  │  └─ src/
│  │     ├─ lib.rs            # re-exports + integration tests
│  │     ├─ color.rs          # Oklch, Srgb types; hex parse/format; CSS output
│  │     ├─ convert.rs        # OKLCH <-> sRGB math (Ottosson Oklab)
│  │     ├─ gamut.rs          # in_gamut + clamp_to_gamut (chroma reduction)
│  │     ├─ ramp.rs           # tonal ramp generation + tuned lightness table
│  │     └─ wcag.rs           # WCAG 2.x contrast analysis (built but not wired to UI)
│  └─ chromix-cli/            # binary `chromix`
│     └─ src/
│        ├─ main.rs           # entry, command dispatch, clipboard
│        ├─ cli.rs            # clap derive arg definitions
│        ├─ render.rs         # terminal swatch/ramp rendering (owo-colors)
│        ├─ splash.rs         # no-arg intro screen + gradient animation
│        └─ export/
│           ├─ mod.rs         # format selection + file writing
│           ├─ json.rs        # colors.json
│           ├─ tailwind.rs    # tailwind.colors.js
│           └─ css.rs         # colors.css
```

### Workspace `Cargo.toml`
- `resolver = "2"`, `edition = "2021"`, `version = "0.1.0"`, `license = "MIT"`.
- Workspace dependencies: `serde` (derive), `serde_json`, `clap` (derive),
  `owo-colors = "4"`, `arboard = "3"`.

### `chromix-core/Cargo.toml`
- Only dependency: `serde` with `derive`. **No** clap, no owo-colors, no fs. This
  purity is a hard requirement.

### `chromix-cli/Cargo.toml`
- `[[bin]] name = "chromix"`, `path = "src/main.rs"`.
- Deps: `chromix-core` (path), `clap`, `owo-colors`, `serde_json`,
  `terminal-light = "1.8"` (terminal background detection), `arboard` (clipboard).

> Naming nuance: the binary is `chromix`; the crates are `chromix-core` /
> `chromix-cli`. Some doc comments and generated-file headers still say `oklch`
> (an earlier name). When rebuilding, standardize on `chromix` everywhere
> (including the generated-token header comments — see §7).

---

## 3. Core types (`color.rs`)

### `Oklch`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Oklch { pub l: f64, pub c: f64, pub h: f64 }
```
- `l`: lightness `0.0` (black) … `1.0` (white).
- `c`: chroma `0.0` (gray) … ~`0.37` (most saturated representable).
- `h`: hue angle in degrees `0.0`…`360.0`.
- `const fn new(l, c, h)`.

### `Srgb`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Srgb { pub r: u8, pub g: u8, pub b: u8 }
```
- `const fn new(r, g, b)`.

### `ParseHexError`
```rust
pub enum ParseHexError { BadLength(usize), BadDigit(char) }
```
- Implements `Display` + `std::error::Error`.
- `BadLength(n)` message: `"expected 3, 6, or 8 hex digits, got {n}"`.
- `BadDigit(ch)` message: `"invalid hex digit '{ch}'"`.

### Hex parsing — `Srgb::from_hex(s: &str) -> Result<Srgb, ParseHexError>`
Rules (get these exactly right — they're load-bearing for shell ergonomics):
1. Strip a single leading `#` if present (optional).
2. Validate: if any char is not an ASCII hex digit, return `BadDigit(firstBad)`.
   **This check happens before length** — a bad digit wins over a bad length.
3. Dispatch on length (after `#` strip):
   - `3 | 4` → **shorthand**: each of the first 3 digits is doubled
     (`f0a` → `ff00aa`). The 4th digit (shorthand alpha) is ignored.
   - `6 | 8` → take bytes `[0..2]`, `[2..4]`, `[4..6]` as r/g/b. Bytes `[6..8]`
     (alpha) are ignored.
   - anything else → `BadLength(len)`.
4. **Alpha is always discarded — v1 is opaque-only.**

### Hex formatting — `Srgb::to_hex(self) -> String`
- Lowercase, always 6 digits with `#` prefix: `format!("#{:02x}{:02x}{:02x}", r, g, b)`.

### `Oklch::from_hex(s) -> Result<Oklch, ParseHexError>`
- = `Srgb::from_hex(s)?.to_oklch()`.

### `Oklch::to_css(self) -> String`
- Format: `oklch(L% C H)` where L is `l*100` to **1 decimal**, C to **3 decimals**,
  H to **1 decimal**. Example: `oklch(62.3% 0.188 259.8)`.
- Exact format string: `"oklch({:.1}% {:.3} {:.1})"` with `(l*100.0, c, h)`.

### Convenience methods
- `Srgb::to_oklch()` → calls `convert::srgb_to_oklch`.
- `Oklch::to_srgb()` → calls `convert::oklch_to_srgb` (may clip if out of gamut;
  clamp first for guaranteed-valid output).

---

## 4. Conversion math (`convert.rs`) — the heart

Hand-rolled from Björn Ottosson's Oklab spec (https://bottosson.github.io/posts/oklab/).
Pipeline: `sRGB (gamma) ↔ linear sRGB ↔ LMS ↔ Oklab ↔ OKLCH`. **Use these exact
constants** — they are not optional; published OKLCH values depend on them.

### sRGB transfer functions (per channel, value in [0,1])
```
srgb_to_linear(c) = c <= 0.04045 ? c/12.92 : ((c+0.055)/1.055)^2.4
linear_to_srgb(c) = c <= 0.0031308 ? c*12.92 : 1.055*c^(1/2.4) - 0.055
```

### linear sRGB → Oklab
```
l = 0.4122214708*r + 0.5363325363*g + 0.0514459929*b
m = 0.2119034982*r + 0.6806995451*g + 0.1073969566*b
s = 0.0883024619*r + 0.2817188376*g + 0.6299787005*b

l_ = cbrt(l); m_ = cbrt(m); s_ = cbrt(s)

L = 0.2104542553*l_ + 0.7936177850*m_ - 0.0040720468*s_
a = 1.9779984951*l_ - 2.4285922050*m_ + 0.4505937099*s_
b = 0.0259040371*l_ + 0.7827717662*m_ - 0.8086757660*s_
```

### Oklab → linear sRGB
```
l_ = L + 0.3963377774*a + 0.2158037573*b
m_ = L - 0.1055613458*a - 0.0638541728*b
s_ = L - 0.0894841775*a - 1.2914855480*b

l = l_^3; m = m_^3; s = s_^3

r =  4.0767416621*l - 3.3077115913*m + 0.2309699292*s
g = -1.2684380046*l + 2.6097574011*m - 0.3413193965*s
b = -0.0041960863*l - 0.7034186147*m + 1.7076147010*s
```

### sRGB(8-bit) → OKLCH — `srgb_to_oklch(Srgb) -> Oklch`
1. Normalize each channel `/255.0`, apply `srgb_to_linear`.
2. linear → Oklab `(L, a, b)`.
3. `chroma = sqrt(a² + b²)`.
4. `hue = atan2(b, a)` in **degrees**; if negative, add `360.0`.
5. Return `Oklch { l: L, c: chroma, h: hue }`.

### OKLCH → sRGB(8-bit) — `oklch_to_srgb(Oklch) -> Srgb`
1. `oklch_to_linear_srgb` (below) → `(r,g,b)` linear, possibly out of [0,1].
2. Per channel: `clamp(v, 0, 1)` → `linear_to_srgb` → `*255` → `round` →
   `clamp(0,255)` → `as u8`.
   - **Nuance:** the clamp-to-[0,1] here is *channel clipping*, which can shift
     hue. Callers wanting fidelity should `clamp_to_gamut` (chroma reduction)
     first. The ramp generator always does.

### OKLCH → linear sRGB (unclamped) — `oklch_to_linear_srgb(Oklch) -> (f64,f64,f64)`
1. `h_rad = h.to_radians()`; `a = c*cos(h_rad)`; `b = c*sin(h_rad)`.
2. `oklab_to_linear_srgb(l, a, b)`.
- Used by gamut testing (needs raw, unclamped linear values).

### Verification tests (replicate)
- **Round-trip:** for `#3b82f6, #ef4444, #10b981, #000000, #ffffff, #808080`,
  hex → OKLCH → sRGB must match the original ±1 per channel.
- **Known value:** `#3b82f6` (Tailwind blue-500) → `l≈0.623` (±0.02),
  `c≈0.188` (±0.02), `h≈259.8` (±2.0).
- **Transfer round-trip:** `linear_to_srgb(srgb_to_linear(v)) ≈ v` within `1e-9`
  for several `v`.
- **Extremes:** white → `L≈1.0`, black → `L≈0.0` (each within `0.01`).

---

## 5. Gamut handling (`gamut.rs`)

Many OKLCH triples have no sRGB representation. Bring them in by **reducing chroma**
while holding L and H fixed — this preserves perceived hue and brightness far better
than channel clipping.

### Constant
- `EPS = 1e-4` — float-error tolerance so colors a hair outside the cube still count
  as in-gamut and don't trigger needless chroma reduction.

### `in_gamut(c: Oklch) -> bool`
- Compute `oklch_to_linear_srgb(c)`; color is in gamut iff every channel is in
  `[-EPS, 1.0 + EPS]`.

### `clamp_to_gamut(c: Oklch) -> Oklch`
- If already `in_gamut`, return unchanged (so applying twice is a no-op /
  idempotent — this is a tested invariant).
- Otherwise **binary search on chroma**: `lo = 0.0` (gray, always in gamut),
  `hi = c.c` (the requested, out-of-gamut chroma). Run **exactly 24 iterations**:
  `mid = (lo+hi)/2`; if `in_gamut(l, mid, h)` then `lo = mid` else `hi = mid`.
  Return `Oklch::new(c.l, lo, c.h)` (the largest known-good chroma).

### Tests (replicate)
- `clamp_to_gamut` idempotent: clamping an over-saturated color twice changes
  chroma by `< 1e-9` and the result is `in_gamut`.
- An in-gamut color is returned exactly unchanged.

---

## 6. Ramp generation (`ramp.rs`) — the product's signature

Given a base OKLCH, produce a Tailwind-style scale. Hue ≈ constant, lightness
distributed perceptually light→dark, chroma tapered at the extremes, every step
gamut-clamped.

### Constants
```rust
pub const DEFAULT_STEPS: &[u16] = &[50,100,200,300,400,500,600,700,800,900,950];
```
Tuned per-step lightness table (the "secret sauce" — spaced wider in midtones,
gentle at the ends, with 500 near a typical mid lightness):
```
50→0.971  100→0.936  200→0.885  300→0.808  400→0.704  500→0.610
600→0.530 700→0.452  800→0.378  900→0.310  950→0.220
```

### Output types
```rust
pub struct RampEntry { pub step: u16, pub oklch: Oklch, pub srgb: Srgb, pub hex: String }  // Serialize
pub struct Ramp { pub entries: Vec<RampEntry> }                                            // Serialize
```
- `oklch` is the **gamut-clamped** color; `srgb`/`hex` are its sRGB resolution.

### `generate_ramp(base) -> Ramp` = `generate_ramp_with_steps(base, DEFAULT_STEPS)`.

### `generate_ramp_with_steps(base, steps) -> Ramp`
For each step at index `i` of `n = steps.len()`:
1. **Lightness** `l = target_lightness(step, i, n)` (below).
2. **Chroma taper** — a downward parabola peaking mid-scale so near-white /
   near-black steps don't get aggressively clamped:
   ```
   t = (n > 1) ? i/(n-1) : 0.5
   taper = 1.0 - (2t - 1)^2 * 0.5     // 1.0 at the middle, 0.5 at both ends
   chroma = base.c * taper
   ```
3. `target = Oklch::new(l, chroma, base.h)`.
4. `oklch = clamp_to_gamut(target)`; `srgb = oklch.to_srgb()`; `hex = srgb.to_hex()`.

### `target_lightness(step, index, count) -> f64`
- If `step` is one of the default steps, use the **tuned table** value.
- Otherwise interpolate linearly across `LIGHTEST = 0.971` … `DARKEST = 0.220` by
  list position: `t = index/(count-1)` (or `0.5` if `count==1`); return
  `LIGHTEST + (DARKEST - LIGHTEST)*t`.
- **Nuance:** the table is keyed by step *number*, not position. A custom step list
  that happens to include `500` will pull `500`'s tuned lightness even if it sits at
  a different position; non-default step numbers interpolate. This is intentional.

### Tested invariants (replicate)
- Ramp has exactly `DEFAULT_STEPS.len()` entries; every entry's `oklch` is `in_gamut`.
- Lightness is **monotonically non-increasing** across the ramp (each step is ≤ the
  previous in `oklch.l`).

---

## 7. Export formats (`export/`)

`Formats { json: bool, tailwind: bool, css: bool }` selects which files to write.

`export(formats, name, ramp, out_dir) -> io::Result<Vec<PathBuf>>`:
1. `fs::create_dir_all(out_dir)` (created if missing).
2. For each selected format, write its file into `out_dir` and collect the path.
3. Return the list of written paths.

### `colors.json` (`json.rs`)
- Shape: `{ "<name>": { "<step>": "#hex", … } }`, **hex** values (not oklch).
- Pretty-printed (`serde_json::to_string_pretty`), trailing newline appended.
- Example (`name = "primary"`, blue base) — note serde's `serde_json::Map` orders
  keys alphabetically, so `50` sorts after `400`:
  ```json
  {
    "primary": {
      "100": "#deebff",
      "200": "#c4dbff",
      "300": "#9cc2ff",
      "400": "#639eff",
      "50": "#f0f6ff",
      "500": "#377ef1",
      ...
      "950": "#001743"
    }
  }
  ```
  (If you want numeric step ordering instead of lexicographic, use an ordered map —
  the original ships with lexicographic order, matching the committed `colors.json`.)

### `tailwind.colors.js` (`tailwind.rs`)
- A `module.exports` snippet with `theme.extend.colors.<name>`, values are **oklch()**
  strings (via `Oklch::to_css`), steps quoted as keys:
  ```js
  /** Generated by chromix. Drop into your tailwind.config.js. */
  module.exports = {
    theme: {
      extend: {
        colors: {
          primary: {
            "50": "oklch(97.1% 0.013 257.4)",
            ...
          },
        },
      },
    },
  };
  ```
- `js_key(name)`: leave unquoted only if it's a valid JS identifier — non-empty,
  every char is `_`/ASCII-letter, or an ASCII digit at position > 0 (no leading
  digit). Otherwise wrap in double quotes.

### `colors.css` (`css.rs`)
- `:root` block of custom properties, values are **oklch()** strings:
  ```css
  /* Generated by chromix. */
  :root {
    --primary-50: oklch(97.1% 0.013 257.4);
    ...
    --primary-950: oklch(...);
  }
  ```
- No identifier sanitization on `name` here (it's interpolated directly into the
  property name); the caller controls `name`.

> Header-comment nuance: the original files say "Generated by oklch." — standardize
> to "Generated by chromix." on rebuild.

---

## 8. CLI surface (`cli.rs` + `main.rs`)

clap derive. Program name `chromix`, with `version` and `about` from Cargo
metadata. `command` is **optional** — absent → splash screen.

```
chromix [SUBCOMMAND]
```

### Shared `ColorArg`
- Positional `color: String` — base hex (`#rgb`, `#rrggbb`, `#rrggbbaa`; `#`
  optional). Flattened into every subcommand.

### `convert <COLOR> [--copy]`
- Parse color → OKLCH. If `--copy`, put `oklch.to_css()` on the system clipboard
  (best-effort; failure prints a warning to stderr and is non-fatal).
- Render: success line + one inline row `swatch · oklch(...) [📋] | #hex`. If copied,
  also print a dim "copied to clipboard" line.

### `scale <COLOR> [--steps L] [--name N]`
- `--steps`: comma-separated `Vec<u16>` (clap `value_delimiter = ','`), optional;
  default = Tailwind scale.
- `--name`: default `"primary"` — used as the ramp heading.
- Renders the input header (swatch + `input` + hex), then the ramp.

### `export <COLOR> [--steps L] [--name N] [--json] [--tailwind] [--css] [--out DIR]`
- `--name` default `"primary"`; `--out` default `"."` (created if missing).
- `--json` / `--tailwind` / `--css` are **independent boolean flags**. At least one
  is required: if none set, print
  `error: pick at least one format to export: --json, --tailwind, and/or --css`
  and exit FAILURE.
- Behavior: render input header + ramp (so you see what was generated), then write
  the selected files; print `wrote <path>` per file and a final
  `\n✓ exported N file(s) to <out>`. On write error: `error: failed to write
  tokens: {e}`, exit FAILURE.

> **README drift to resolve:** the README documents `export … --format json|tailwind|css|all`.
> The actual implemented interface is the three boolean flags above (no `--format`,
> no `all`). When rebuilding, pick ONE and make code + README agree. Recommended:
> keep the boolean flags (simpler, matches shipped binary) and fix the README; or,
> if you prefer the README's design, add a `--format` enum with an `all` value and a
> default. Do not ship both descriptions out of sync.

### `parse_color` helper
- `Oklch::from_hex(color)` mapping the error to: print
  `error: invalid color '{color}': {e}` to stderr, return `ExitCode::FAILURE`.

### `build_ramp` helper
- `Some(steps)` non-empty → `generate_ramp_with_steps`; else `generate_ramp`.

### Exit codes
- Success → `ExitCode::SUCCESS`; bad input / no-format / write failure → `ExitCode::FAILURE`.

---

## 9. Terminal rendering (`render.rs`)

Uses `owo-colors` truecolor. **Design rule: the color swatches are the only
chromatic elements; all labels/headers/values are grayscale** so the generated
colors visually pop.

Grayscale palette (RGB triples):
```
HEADER = (235,235,235)   LABEL = (200,200,200)   DIM = (130,130,130)
OK_GREEN = (126,196,145) // the only non-gray accent — used for ✓ and 📋
```

### `render_color(oklch, copied)`
- Swatch = 6 spaces with truecolor **background** (`"      ".on_truecolor(r,g,b)`).
- Layout (2-space left margin throughout):
  ```
  <blank>
    ✓ converted to OKLCH        (✓ in OK_GREEN)
  <blank>
    <swatch>  oklch(...)[📋]  |  #hex     (oklch bold HEADER; 📋 OK_GREEN if copied; | and hex DIM)
  [if copied:]
  <blank>
    copied to clipboard          (DIM)
  <blank>
  ```

### `render_input_header(oklch)`
- `<blank>` then `  <swatch>  input  #hex` (`input` DIM, hex LABEL).

### `render_ramp(name, ramp)`
- Heading: `  <name (bold HEADER)> · <N> steps (DIM)`.
- Per entry, one row:
  ```
    <swatch> <step right-aligned width 4 (LABEL)>  <oklch left-aligned width 24 (LABEL)>  <#hex (DIM)>
  ```
- Footer tip: `  Tips: use chromix export to export as json` (`chromix export` bold HEADER).

---

## 10. Splash screen (`splash.rs`) — the no-arg experience

Shown when `chromix` runs with no subcommand. Order: blank → wordmark → blank →
animated gradient bar → blank → two description lines → blank → rule → "HOW TO USE"
→ blank → 3 highlighted example rows → blank → help hint → blank. Shared left
`MARGIN = "  "` (2 spaces) on every line.

### Theme detection
- `terminal_light::luma()` queries the terminal background via OSC 11, returns
  `0.0`(black)…`1.0`(white). `luma > 0.5` → **light** palette; on any error (not a
  tty, no OSC 11 support) → **dark** palette (the default). Done once per run.

### Two palettes
- Wordmark is a 5-row light→dark grayscale sweep:
  - DARK rows: `(245),(210),(175),(140),(105)` (each repeated as r=g=b).
  - LIGHT rows (inverted + darkened to stay legible on white):
    `(40),(70),(100),(130),(160)`.
- Body grays per theme:
  - DARK: subtle `(170)`, dim `(120)`, rule `(80)`, syn_cmd `(235)` (near-white).
  - LIGHT: subtle `(80)`, dim `(120)`, rule `(175)`, syn_cmd `(30)` (near-black).
- Constant syntax hues (read fine on both themes, don't vary):
  - `SYN_SUB = (198,160,246)` soft purple (subcommands),
    `SYN_STR = (126,196,145)` soft green (hex values),
    `SYN_FLAG = (138,170,240)` soft blue (`--flags`).

### Wordmark
- 5-row block-font "CHROMIX" built from `█` glyphs (see source for the exact 5
  strings; each row is 52 chars wide, left edge flush at column 0). Printed bold,
  one grayscale level per row from the chosen sweep.

### Gradient bar
- `BAR_WIDTH = 55`, `BAR_HEIGHT = 2`. Each column `x` maps to hue
  `(x/(BAR_WIDTH-1)*360 + hue_offset) % 360`; color =
  `clamp_to_gamut(Oklch::new(0.70, 0.16, hue)).to_srgb()`, drawn as `█` in that
  truecolor. Two stacked identical rows give it height.
- **Animation:** only if stdout `is_terminal()` (else draw once, static, so piped
  output stays clean). `FRAMES = 24`. For `frame in 0..=24`:
  `t = frame/24`; `eased = 1 - (1-t)^3` (cubic ease-out);
  `offset = (1 - eased) * 360` (drift decelerates to rest at 0). Draw the bar,
  flush, `sleep(28ms)`, then emit `"\x1b[{BAR_HEIGHT}A"` to move the cursor up over
  the bar so the next frame overwrites it. Last frame leaves it settled at offset 0.

### Description + how-to
- Line 1 (subtle): `OKLCH color tool right in your terminal.`
- Line 2 (dim): `Generate OKLCH color scale, gradients and a converter.`
- Rule: `"─".repeat(48)` in `rule` gray. Heading: `HOW TO USE` (subtle, bold).
- Examples (use **bare hex, no `#`** so they're copy-paste safe in zsh/bash where a
  leading `#` starts a comment):
  ```
  chromix convert 3b82f6              # convert a color to OKLCH
  chromix scale 3b82f6               # print an 11-step OKLCH ramp
  chromix export 3b82f6 --json --css  # write the selected token files
  ```
  Each example is syntax-highlighted (see below) with a dim trailing `# comment`.
- Footer (dim): ``Run `chromix --help` for all options.``

### `highlight(cmd, pal)` — tiny tokenizer
- Split on spaces (preserving them via `split_inclusive(' ')`). For each word:
  - `chromix` → `syn_cmd` bold.
  - `convert`/`scale`/`export` → `SYN_SUB`.
  - starts with `--` → `SYN_FLAG`.
  - looks like a hex value (`is_hex_value`) → `SYN_STR`.
  - else → `dim`.
- `is_hex_value(word)`: strip surrounding `"`, strip a leading `#`, then true iff
  length ∈ {3,6,8} and all ASCII hex digits (so both `3b82f6` and `"#3b82f6"` match).

---

## 11. WCAG module (`wcag.rs`) — built but intentionally not wired in

The engine ships a complete WCAG 2.x analyzer. It is **not** currently called by the
CLI rendering (it's roadmap scaffolding / library API). Replicate it as a pure
library feature; wiring it into the ramp output is a v2 decision.

### Thresholds
```
AA_NORMAL_TEXT = 4.5   (WCAG 1.4.3 normal text)
AAA_NORMAL_TEXT = 7.0  (1.4.6 enhanced)
AA_LARGE_OR_UI = 3.0   (1.4.3 large text / 1.4.11 non-text UI, borders)
```

### Luminance + contrast
- `relative_luminance(Srgb) -> f64`: per channel `cs = ch/255`; linearize
  `cs <= 0.03928 ? cs/12.92 : ((cs+0.055)/1.055)^2.4`; weight
  `0.2126*r + 0.7152*g + 0.0722*b` (Rec.709). Range 0..1.
  - **Nuance:** the WCAG threshold is `0.03928`, *slightly different* from the
    sRGB transfer's `0.04045` in `convert.rs`. Keep them distinct — they come from
    different specs. Don't "unify" them.
- `contrast_ratio(a, b) -> f64`: `(hi + 0.05)/(lo + 0.05)` where hi/lo are the
  larger/smaller relative luminance. Range 1.0..21.0.

### Types
- `Usage { Text, Border, Background }` — `label()` → `"text"`/`"border"`/`"bg"`;
  serde lowercase.
- `Rating { Aaa, Aa, Fail }` — `label()` → `"AAA"`/`"AA"`/`"—"`; serde lowercase.
- `Wcag { usage, rating, on: Option<Srgb>, vs_white: f64, vs_black: f64 }`.

### `analyze(swatch: Srgb) -> Wcag` (anchored to contrast vs **white**)
- Let `vs_white = contrast_ratio(swatch, WHITE)`, `vs_black = contrast_ratio(swatch, BLACK)`.
- If `vs_white >= 4.5` → `Usage::Text`, `on = None`, rating `Aaa` if `vs_white>=7`
  else `Aa`.
- Else if `vs_white >= 3.0` → `Usage::Border`, rating `Aa`, `on = None`.
- Else (too light to sit on white) → `Usage::Background`; `on = BLACK if vs_black>=vs_white else WHITE`;
  `on_contrast = max(vs_white, vs_black)`; rating `Aaa` if `>=7`, `Aa` if `>=4.5`,
  else `Fail`.

### Tests (replicate)
- white↔black contrast ≈ 21.0; white↔white ≈ 1.0.
- white swatch → Background, on=BLACK, AAA.
- black swatch → Text, AAA, on=None.
- `#2563eb` → Text. `#eff6ff` → Background, on=BLACK. `#949494` → Border, AA.

---

## 12. Cross-cutting nuances (the easy-to-miss list)

1. **Hex `#` optional everywhere** and **bare hex in all examples/docs** — this is a
   deliberate shell-ergonomics choice (zsh/bash treat `#…` as a comment).
2. **Alpha is parsed-but-discarded** — accept `#rgba`/`#rrggbbaa`, ignore alpha.
3. **Bad-digit error beats bad-length error** (validation order).
4. **Chroma reduction, never channel clipping, for gamut** in any user-facing color.
   Channel clipping only happens at the final quantization and is a fallback.
5. **24 binary-search iterations** for gamut clamp; **EPS = 1e-4** tolerance.
6. **Chroma taper parabola** (`0.5` at ends, `1.0` mid) is what keeps `50`/`950`
   in gamut without ugly clamping — don't drop it.
7. **Tuned lightness table is keyed by step number**, with positional interpolation
   only for non-default steps.
8. **JSON keys sort lexicographically** (serde Map) → `50` lands after `400`.
   Match this, or consciously switch to numeric ordering.
9. **Two different sRGB-linearization thresholds** exist on purpose (`0.04045` in
   convert, `0.03928` in WCAG). Keep both.
10. **Grayscale-only chrome**; the swatches and the splash gradient are the sole
    color. `OK_GREEN` is the one accent (✓ / 📋 / hex highlight).
11. **Splash animates only on a tty**; static single draw when piped.
12. **Clipboard failure is non-fatal** (warn + continue).
13. **Engine purity:** `chromix-core` must not gain terminal/fs/clap deps. The
    `Serialize` derives on all public types exist so a future GUI/WASM/JSON-API can
    reuse them.
14. **Generated-file header comments** currently say "oklch" — fix to "chromix".
15. **README ↔ CLI `export` flag mismatch** — resolve (see §8).

---

## 13. Suggested rebuild order

1. `chromix-core`: `color.rs` (types + hex) → `convert.rs` (+ tests) →
   `gamut.rs` (+ tests) → `ramp.rs` (+ tests) → `wcag.rs` (+ tests) → `lib.rs`
   re-exports + integration tests. Run `cargo test -p chromix-core` and confirm the
   known-value and round-trip tests pass before touching the CLI.
2. `chromix-cli`: `cli.rs` (clap) → `main.rs` (dispatch + parse/build helpers +
   clipboard) → `render.rs` → `export/*` → `splash.rs`.
3. Reconcile README with the implemented flags; standardize the `chromix` name in
   all generated headers and doc comments.

---

## 14. Porting notes (if not Rust)

- **f64 throughout** the math; matrix constants and transfer functions are
  language-independent — copy them verbatim.
- `cbrt` and `atan2` must be available (most stdlibs have them). Hue from `atan2`
  is in radians → convert to degrees, normalize to `[0,360)`.
- Replace `clap` with the host language's arg parser; keep the exact flag names,
  defaults, and the "at least one format" rule.
- Replace `owo-colors` with ANSI truecolor escapes: foreground
  `\x1b[38;2;{r};{g};{b}m`, background `\x1b[48;2;{r};{g};{b}m`, reset `\x1b[0m`.
  Swatch = background-colored spaces.
- Replace `terminal-light` with an OSC 11 query (`\x1b]11;?\x07`) and parse the
  reply, or skip detection and default to the dark palette.
- Replace `arboard` with the platform clipboard (or omit `--copy`).
- WASM target: compile `chromix-core` only (it's already I/O-free); expose
  `from_hex`, `generate_ramp`, `to_css`, and the `Serialize` structs as JSON.

---

## 15. Acceptance checklist

- [ ] `chromix convert 3b82f6` prints `oklch(62.3% 0.188 259.8)` (±rounding).
- [ ] `chromix scale 3b82f6` prints 11 swatched rows, lightness monotonic, all in gamut.
- [ ] `chromix export 3b82f6 --json` writes a `colors.json` matching the §7 shape.
- [ ] `chromix export 3b82f6` (no format) errors and exits non-zero.
- [ ] `chromix` (no args) shows the wordmark, an animated spectrum bar (in a tty),
      and the how-to block; piped output is static.
- [ ] All `chromix-core` unit + integration tests pass.
- [ ] `chromix-core` has zero terminal/fs/cli dependencies.
```
