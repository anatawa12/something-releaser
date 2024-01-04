// this implementation is based on cargo set-version which is part of cargo-edit crate
// The MIT License (MIT)
// Copyright (c) 2015 Without Boats, Pascal Hertleif
// https://github.com/killercup/cargo-edit/blob/f9eed30846c182252855ffee74667d17514de0ae/LICENSE

use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use cargo_edit::{LocalManifest, upgrade_requirement};
use cargo_metadata::semver;
use serde::Deserialize;
use log::{debug};
use crate::version_changer::VersionChanger;

#[derive(Debug, Deserialize)]
pub struct Cargo {
    manifest_path: Option<PathBuf>,
    package: Option<String>,
}

impl Display for Cargo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "cargo(at {:?}, package {:?})", self.manifest_path, self.package)
    }
}

impl VersionChanger for Cargo {
    fn parse(info: Option<&str>, path: Option<&str>) -> Self {
        Self {
            manifest_path: path.map(Into::into),
            package: info.map(|s| s.to_owned()),
        }
    }

    async fn load_version(&self) -> String {
        let ws_metadata = cargo_metadata(self.manifest_path.as_deref(), true);
        let root_manifest_path = ws_metadata.workspace_root.as_std_path().join("Cargo.toml");
        let ws_manifest = LocalManifest::try_new(&root_manifest_path)
            .expect("loading manifest of workspace root");

        if ws_manifest.data.get("workspace").is_none() {
            if self.package.is_some() {
                panic!("to upgrade version name of specific package, don't specify package name");
            }

            ws_metadata.packages[0].version.to_string()
        } else if let Some(name) = self.package.as_deref() {
            // name specified: get version of specified package in workspace
            let package = ws_metadata.packages.iter().find(|p| name == &p.name)
                .expect("no package with specified name found");
            package.version.to_string()
        } else {
            // name not specified: get version of workspace
            ws_manifest.get_workspace_version()
                .expect("no workspace-wide version specified. to get version of package in a workspace, specify package name")
                .to_string()
        }
    }

    async fn set_version(&self, version: &str) {
        let version = semver::Version::parse(version).expect("version name is not semver; cargo doesn't support non-semver");

        let ws_metadata = cargo_metadata(self.manifest_path.as_deref(), true);
        let root_manifest_path = ws_metadata.workspace_root.as_std_path().join("Cargo.toml");
        let ws_manifest = LocalManifest::try_new(&root_manifest_path)
            .expect("loading manifest of workspace root");

        if ws_manifest.data.get("workspace").is_none() {
            // it's not workspace (single package) project
            if self.package.is_some() {
                panic!("to upgrade version name of specific package, don't specify package name");
            }
            if ws_metadata.packages.len() != 1 {
                panic!("Logic failure: non workspace project has more than one package");
            }

            let the_package = &ws_metadata.packages[0];

            let mut manifest = LocalManifest::try_new(Path::new(&the_package.manifest_path))
                .expect("loading manifest");

            debug!("upgrading {} from {} to {}", the_package.name, the_package.version, version);
            manifest.set_package_version(&version);
            manifest.write().expect("writing manifest");

            // with single package, there's no need to update dependents

            cargo_metadata(self.manifest_path.as_deref(), false);
        } else if let Some(name) = self.package.as_deref() {
            // name specified: set version of specified package in workspace
            let the_package = ws_metadata.packages.iter().find(|p| name == &p.name)
                .expect("no package with specified name found");

            let mut manifest = LocalManifest::try_new(Path::new(&the_package.manifest_path))
                .expect("loading manifest");
            if manifest.version_is_inherited() {
                panic!("to upgrade version name in workspace, don't specify package name");
            };

            debug!("upgrading {} from {} to {}", the_package.name, the_package.version, version);
            manifest.set_package_version(&version);
            manifest.write().expect("writing manifest");

            let crate_root =
                dunce::canonicalize(the_package.manifest_path.parent().expect("at least a parent"))
                    .expect("canonicalize path");

            update_dependents(
                &crate_root,
                &version,
                &root_manifest_path,
                &ws_metadata.packages,
            );

            cargo_metadata(self.manifest_path.as_deref(), false);
        } else {
            // name not specified: set version of workspace

            let mut ws_manifest = LocalManifest::try_new(&root_manifest_path)
                .expect("loading manifest of workspace root");

            if !ws_manifest.get_workspace_version().is_some() {
                panic!("no workspace-wide version specified. to seet version of package in a workspace, specify package name");
            }

            ws_manifest.set_workspace_version(&version);
            ws_manifest.write().expect("writing manifest");

            for package in &ws_metadata.packages {
                let manifest = LocalManifest::try_new(Path::new(&package.manifest_path))
                    .expect("loading manifest");

                if manifest.version_is_inherited() {
                    let crate_root =
                        dunce::canonicalize(package.manifest_path.parent().expect("at least a parent"))
                            .expect("canonicalize path");
                    update_dependents(
                        &crate_root,
                        &version,
                        &root_manifest_path,
                        &ws_metadata.packages,
                    )
                }
            }

            cargo_metadata(self.manifest_path.as_deref(), false);
        }
    }
}

fn update_dependents(
    crate_root: &Path,
    next: &semver::Version,
    root_manifest_path: &Path,
    workspace_members: &[cargo_metadata::Package],
) {
    // This is redundant with iterating over `workspace_members`
    // - As `get_dependency_tables_mut` returns workspace dependencies
    // - If there is a root package
    //
    // But split this out for
    // - Virtual manifests
    // - Nicer message to the user
    {
        update_dependent(crate_root, next, root_manifest_path, "workspace");
    }

    for member in workspace_members.iter() {
        update_dependent(
            crate_root,
            next,
            member.manifest_path.as_std_path(),
            &member.name,
        );
    }
}

fn is_relevant(d: &dyn toml_edit::TableLike, dep_crate_root: &Path, crate_root: &Path) -> bool {
    if !d.contains_key("version") {
        return false;
    }
    match d
        .get("path")
        .and_then(|i| i.as_str())
        .and_then(|relpath| dunce::canonicalize(dep_crate_root.join(relpath)).ok())
    {
        Some(dep_path) => dep_path == crate_root,
        None => false,
    }
}

fn update_dependent(
    crate_root: &Path,
    next: &semver::Version,
    manifest_path: &Path,
    name: &str,
) {
    let mut dep_manifest = LocalManifest::try_new(manifest_path).expect("loading Cargo.toml");
    let mut changed = false;
    let dep_crate_root = dep_manifest
        .path
        .parent()
        .expect("at least a parent")
        .to_owned();

    for dep in dep_manifest
        .get_dependency_tables_mut()
        .flat_map(|t| t.iter_mut().filter_map(|(_, d)| d.as_table_like_mut()))
        .filter(|d| is_relevant(*d, &dep_crate_root, crate_root))
    {
        let old_req = dep
            .get("version")
            .expect("filter ensures this")
            .as_str()
            .unwrap_or("*");
        if let Some(new_req) = upgrade_requirement(old_req, next).expect("updating requirement"){
            debug!("Updating {name}'s dependency from {old_req} to {new_req}");
            dep.insert("version", toml_edit::value(new_req));
            changed = true;
        }
    }
    if changed {
        dep_manifest.write().expect("writing Cargo.toml")
    }
}

fn cargo_metadata(manifest_path: Option<&Path>, no_deps: bool) -> cargo_metadata::Metadata {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    if let Some(manifest_path) = manifest_path {
        cmd.manifest_path(manifest_path);
    }
    cmd.features(cargo_metadata::CargoOpt::AllFeatures);
    if no_deps {
        cmd.no_deps();
    }
    cmd.exec().expect("cargo metadata failed")
}
