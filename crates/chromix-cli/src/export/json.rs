use chromix_core::Ramp;
use serde_json::{Map, Value};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn write(path: &Path, name: &str, ramp: &Ramp) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let mut entries: Map<String, Value> = Map::new();
    for entry in &ramp.entries {
        entries.insert(entry.step.to_string(), Value::String(entry.hex.clone()));
    }

    let mut result: Map<String, Value> = Map::new();
    result.insert(name.to_string(), Value::Object(entries));

    let json = serde_json::to_string_pretty(&result)?;
    writer.write_all(json.as_bytes())?;
    writer.write_all(b"\n")?;

    Ok(())
}
