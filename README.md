<p align="center">
  <img src="assets/banner.svg" alt="Chromix — OKLCH color tool, right in your terminal" width="720">
</p>

# Chromix

OKLCH color tool right in your terminal.
Generate OKLCH color scale and convert hex to OKLCH. 

Note: This is a fun tool I've built to test out Poolside's Laguna model. 

v0.1.0 by Iman Nazri

## Install

Linux & MacOS
```
curl -fsSL https://raw.githubusercontent.com/imanazri/chromix/main/install.sh | sh
```

### Build from source

Requires [Rust](https://rustup.rs).

```
git clone https://github.com/imanazri/chromix.git
cd chromix
cargo install --path crates/chromix-cli
```

## Usage

### Convert a color to OKLCH

```
chromix convert 3b82f6
chromix convert 3b82f6 --copy
```

### Generate a color scale

```
chromix scale 3b82f6
chromix scale 3b82f6 --steps 100,300,500,700,900
chromix scale 3b82f6 --name brand
```

### Export design tokens

At least one format flag is required.

```
chromix export 3b82f6 --json
chromix export 3b82f6 --tailwind
chromix export 3b82f6 --css
chromix export 3b82f6 --json --css --name brand --out ./tokens
```

This writes `colors.json`, `tailwind.colors.js`, and/or `colors.css` to the output directory.

## Hex input

The leading `#` is optional (and best left off, since most shells treat `#` as a comment). Shorthand 3-digit hex is supported. Alpha channels in 4 or 8-digit hex are accepted but ignored.

```
chromix convert 3b82f6
chromix convert f00
```

## License

MIT
