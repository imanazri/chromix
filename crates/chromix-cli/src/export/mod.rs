use chromix_core::Ramp;
use std::fs;
use std::io;
use std::path::PathBuf;

pub mod json;
pub mod tailwind;
pub mod css;

pub fn export(
    json: bool,
    tailwind: bool,
    css: bool,
    name: &str,
    ramp: &Ramp,
    out_dir: &str,
) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(out_dir)?;

    let mut paths = Vec::new();

    if json {
        let path = PathBuf::from(out_dir).join("colors.json");
        json::write(&path, name, ramp)?;
        paths.push(path);
    }

    if tailwind {
        let path = PathBuf::from(out_dir).join("tailwind.colors.js");
        tailwind::write(&path, name, ramp)?;
        paths.push(path);
    }

    if css {
        let path = PathBuf::from(out_dir).join("colors.css");
        css::write(&path, name, ramp)?;
        paths.push(path);
    }

    Ok(paths)
}