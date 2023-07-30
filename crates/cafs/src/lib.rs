use std::{
    fs,
    path::{Path, PathBuf},
};

use ssri::{Algorithm, IntegrityOpts};
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
#[error(transparent)]
pub enum CafsError {
    #[error("io error")]
    Io(#[from] std::io::Error),
}

enum FileType {
    Exec,
    NonExec,
    Index,
}

fn content_path_from_hex(file_type: FileType, hex: &str) -> PathBuf {
    let mut p = PathBuf::new();
    p.push(&hex[0..2]);

    let extension = match file_type {
        FileType::Exec => "-exec",
        FileType::NonExec => "",
        FileType::Index => "-index.json",
    };

    p.join(format!("{}{}", &hex[2..], extension))
}

pub fn write_sync(store_dir: &Path, buffer: &Vec<u8>) -> Result<String, CafsError> {
    let hex_integrity =
        IntegrityOpts::new().algorithm(Algorithm::Sha512).chain(buffer).result().to_hex().1;

    let file_path = store_dir.join(content_path_from_hex(FileType::NonExec, &hex_integrity));

    if !file_path.exists() {
        let parent_dir = file_path.parent().unwrap();
        fs::create_dir_all(parent_dir)?;
        fs::write(&file_path, buffer)?;
    }

    Ok(file_path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_content_path_from_hex() {
        assert_eq!(
            content_path_from_hex(FileType::NonExec, "1234567890abcdef1234567890abcdef12345678"),
            PathBuf::from("12/34567890abcdef1234567890abcdef12345678")
        );
        assert_eq!(
            content_path_from_hex(FileType::Exec, "1234567890abcdef1234567890abcdef12345678"),
            PathBuf::from("12/34567890abcdef1234567890abcdef12345678-exec")
        );
        assert_eq!(
            content_path_from_hex(FileType::Index, "1234567890abcdef1234567890abcdef12345678"),
            PathBuf::from("12/34567890abcdef1234567890abcdef12345678-index.json")
        );
    }
}
