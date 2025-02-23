use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::TempDir;

const PROP_EXTRACT: &[u8] = include_bytes!("extract/PropExtract.class");

pub fn extract_java_properties(java_executable: impl AsRef<Path>, props: impl IntoIterator<Item=impl Into<String>>) -> Option<HashMap<String, String>> {
    let exe_path = java_executable.as_ref();
    if !exe_path.exists() {
        return None;
    }

    let temp_dir = TempDir::new().ok()?;
    let temp_dir_path = temp_dir.path();
    fs::write(temp_dir_path.join("PropExtract.class"), PROP_EXTRACT).ok()?;

    let process = Command::new(exe_path.as_os_str())
        .current_dir(temp_dir_path)
        .args(["-Dfile.encoding=UTF8", "-cp", ".", "PropExtract"])
        .args(props.into_iter().map(|e| e.into()))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;

    let mut properties: HashMap<String, String> = HashMap::new();
    process.stdout.into_iter()
        .flat_map(|e| BufReader::new(e).lines())
        .flatten()
        .filter(|e| !e.is_empty())
        .map(|e| e.splitn(2, "=").map(|s| s.into()).collect::<Vec<String>>())
        .filter(|e| e.len() == 2)
        .for_each(|mut e| { properties.insert(e.remove(0), e.remove(0)); });

    Some(properties)
}
