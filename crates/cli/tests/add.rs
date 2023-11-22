use assert_cmd::prelude::*;
use command_extra::CommandExtra;
use pacquet_package_manifest::{DependencyGroup, PackageManifest};
use pacquet_testing_utils::{
    bin::{AddMockedRegistry, CommandTempCwd},
    fs::{get_all_folders, get_filenames_in_folder},
};
use pretty_assertions::assert_eq;
use std::{env, ffi::OsStr, fs, path::PathBuf};
use tempfile::TempDir;

fn exec_pacquet_in_temp_cwd<Args>(args: Args) -> (TempDir, PathBuf, AddMockedRegistry)
where
    Args: IntoIterator,
    Args::Item: AsRef<OsStr>,
{
    let CommandTempCwd { pacquet, root, workspace, npmrc_info, .. } =
        CommandTempCwd::init().add_mocked_registry();
    pacquet.with_args(args).assert().success();
    (root, workspace, npmrc_info)
}

#[test]
fn should_install_all_dependencies() {
    let (root, workspace, anchor) =
        exec_pacquet_in_temp_cwd(["add", "@pnpm.e2e/hello-world-js-bin-parent"]);

    eprintln!("Directory list");
    insta::assert_debug_snapshot!(get_all_folders(&workspace));

    let manifest_path = workspace.join("package.json");

    eprintln!("Ensure the manifest file ({manifest_path:?}) exists");
    assert!(manifest_path.exists());

    let virtual_store_dir = workspace.join("node_modules").join(".pnpm");

    eprintln!("Ensure virtual store dir ({virtual_store_dir:?}) exists");
    assert!(virtual_store_dir.exists());

    eprintln!("Ensure that @pnpm.e2e/hello-world-js-bin has no other dependencies than itself");
    let path = virtual_store_dir.join("@pnpm.e2e+hello-world-js-bin@1.0.0/node_modules");
    assert_eq!(get_filenames_in_folder(&path), ["@pnpm.e2e"]);
    assert_eq!(get_filenames_in_folder(&path.join("@pnpm.e2e")), ["hello-world-js-bin"]);

    eprintln!("Ensure that @pnpm.e2e/hello-world-js-bin-parent has correct dependencies");
    let path = virtual_store_dir.join("@pnpm.e2e+hello-world-js-bin-parent@1.0.0/node_modules");
    assert_eq!(get_filenames_in_folder(&path), ["@pnpm.e2e"]);
    assert_eq!(
        get_filenames_in_folder(&path.join("@pnpm.e2e")),
        ["hello-world-js-bin", "hello-world-js-bin-parent"],
    );

    drop((root, anchor)); // cleanup
}

#[test]
#[cfg(unix)]
pub fn should_symlink_correctly() {
    use pipe_trait::Pipe;

    let (root, workspace, anchor) =
        exec_pacquet_in_temp_cwd(["add", "@pnpm.e2e/hello-world-js-bin-parent"]);

    eprintln!("Directory list");
    insta::assert_debug_snapshot!(get_all_folders(&workspace));

    let manifest_path = workspace.join("package.json");

    eprintln!("Ensure the manifest file ({manifest_path:?}) exists");
    assert!(manifest_path.exists());

    let virtual_store_dir = workspace.join("node_modules").join(".pnpm");

    eprintln!("Ensure virtual store dir ({virtual_store_dir:?}) exists");
    assert!(virtual_store_dir.exists());

    eprintln!("Make sure the symlinks are correct");
    assert_eq!(
        virtual_store_dir
            .join("@pnpm.e2e+hello-world-js-bin-parent@1.0.0")
            .join("node_modules")
            .join("@pnpm.e2e")
            .join("hello-world-js-bin")
            .pipe(fs::read_link)
            .expect("read link"),
        virtual_store_dir
            .join("@pnpm.e2e+hello-world-js-bin@1.0.0")
            .join("node_modules")
            .join("@pnpm.e2e")
            .join("hello-world-js-bin")
            .pipe(fs::canonicalize)
            .expect("canonicalize link target"),
    );

    drop((root, anchor)); // cleanup
}

#[test]
fn should_add_to_package_json() {
    let (root, dir, anchor) = exec_pacquet_in_temp_cwd(["add", "@pnpm.e2e/hello-world-js-bin"]);
    let file = PackageManifest::from_path(dir.join("package.json")).unwrap();
    eprintln!("Ensure @pnpm.e2e/hello-world-js-bin is added to package.json#dependencies");
    assert!(file
        .dependencies([DependencyGroup::Prod])
        .any(|(k, _)| k == "@pnpm.e2e/hello-world-js-bin"));
    drop((root, anchor)); // cleanup
}

#[test]
fn should_add_dev_dependency() {
    let (root, dir, anchor) =
        exec_pacquet_in_temp_cwd(["add", "@pnpm.e2e/hello-world-js-bin", "--save-dev"]);
    let file = PackageManifest::from_path(dir.join("package.json")).unwrap();
    eprintln!("Ensure @pnpm.e2e/hello-world-js-bin is added to package.json#devDependencies");
    assert!(file
        .dependencies([DependencyGroup::Dev])
        .any(|(k, _)| k == "@pnpm.e2e/hello-world-js-bin"));
    drop((root, anchor)); // cleanup
}

#[test]
fn should_add_peer_dependency() {
    let (root, dir, anchor) =
        exec_pacquet_in_temp_cwd(["add", "@pnpm.e2e/hello-world-js-bin", "--save-peer"]);
    let file = PackageManifest::from_path(dir.join("package.json")).unwrap();
    eprintln!("Ensure @pnpm.e2e/hello-world-js-bin is added to package.json#devDependencies");
    assert!(file
        .dependencies([DependencyGroup::Dev])
        .any(|(k, _)| k == "@pnpm.e2e/hello-world-js-bin"));
    eprintln!("Ensure @pnpm.e2e/hello-world-js-bin is added to package.json#peerDependencies");
    assert!(file
        .dependencies([DependencyGroup::Peer])
        .any(|(k, _)| k == "@pnpm.e2e/hello-world-js-bin"));
    drop((root, anchor)); // cleanup
}
