use sha2::{Digest, Sha256, Sha512};
use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn sha256_file(path: impl AsRef<Path>) -> io::Result<String> {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(&path)?;
    io::copy(&mut file, &mut hasher)?;

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn hash_directory(path: impl AsRef<Path>) -> io::Result<String> {
    let mut hasher = Sha512::new();
    process_file(&mut hasher, path.as_ref())?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn process_file(hasher: &mut Sha512, path: &Path) -> io::Result<()> {
    if path.is_dir() {
        add_dir_to_hasher(hasher, &path)?;
    } else {
        hasher.update(fs::metadata(&path)?.len().to_ne_bytes());
        let mut file = fs::File::open(&path)?;
        io::copy(&mut file, hasher)?;
    }

    Ok(())
}

fn add_dir_to_hasher(hasher: &mut Sha512, path: &Path) -> io::Result<()> {
    let mut entries: Vec<PathBuf> = path.read_dir().into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();

    entries.sort();

    for path in entries {
        process_file(hasher, &path)?;
    }

    Ok(())
}
