use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::package_manager::PackageManagerError;
use async_trait::async_trait;
use pacquet_npmrc::PackageImportMethod;
use rayon::prelude::*;

#[async_trait]
pub trait ImportMethodImpl {
    async fn import(
        &self,
        cas_files: &HashMap<String, PathBuf>,
        save_path: PathBuf,
        symlink_to: PathBuf,
    ) -> Result<(), PackageManagerError>;
}

#[async_trait]
impl ImportMethodImpl for PackageImportMethod {
    async fn import(
        &self,
        cas_files: &HashMap<String, PathBuf>,
        save_path: PathBuf,
        symlink_to: PathBuf,
    ) -> Result<(), PackageManagerError> {
        match self {
            PackageImportMethod::Auto => {
                cas_files
                    .into_par_iter()
                    .try_for_each(|(cleaned_entry, store_path)| {
                        auto_import(store_path, &save_path.join(cleaned_entry))
                    })
                    .expect("expected no write errors");

                if !symlink_to.is_symlink() {
                    fs::create_dir_all(symlink_to.parent().unwrap())?;
                    crate::fs::symlink_dir(save_path, symlink_to)?;
                }
            }
            _ => panic!("Not implemented yet"),
        }

        Ok(())
    }
}

fn auto_import<P: AsRef<Path>>(
    original_path: P,
    symlink_path: P,
) -> Result<(), PackageManagerError> {
    if !symlink_path.as_ref().exists() {
        // Create parent folder
        if let Some(parent_folder) = &symlink_path.as_ref().parent() {
            if !parent_folder.exists() {
                fs::create_dir_all(parent_folder)?;
            }
        }

        reflink_copy::reflink_or_copy(original_path, &symlink_path)?;
    }

    Ok(())
}