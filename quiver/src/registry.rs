//! Loading the registry from disk into memory.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::{Asset, AssetId, AssetKind};
use crate::bundle::Bundle;
use crate::{Error, Result};

/// The in-memory view of a registry checkout.
pub struct Registry {
    /// Skills by id.
    pub skills: BTreeMap<AssetId, Asset>,
    /// Shared-references by id.
    pub references: BTreeMap<AssetId, Asset>,
    /// Bundles by name.
    pub bundles: BTreeMap<String, Bundle>,
    /// The registry root on disk.
    pub root: PathBuf,
}

impl Registry {
    /// Loads the registry rooted at the given path.
    pub fn load(root: &Path) -> Result<Self> {
        let skills = load_skills(root)?;
        let references = load_references(root)?;
        let bundles = load_bundles(root)?;
        Ok(Self {
            skills,
            references,
            bundles,
            root: root.to_path_buf(),
        })
    }

    /// Looks up a shared-reference by id.
    pub fn reference(&self, id: &AssetId) -> Result<&Asset> {
        self.references.get(id).ok_or_else(|| Error::UnknownAsset {
            id: id.as_str().to_owned(),
        })
    }
}

/// Reads a file to string, attributing IO errors to the path.
fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })
}

/// Lists a directory's entry paths, attributing IO errors to the directory.
fn list_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dir).map_err(|source| Error::Read {
        path: dir.to_path_buf(),
        source,
    })?;
    entries
        .map(|entry| {
            entry.map(|e| e.path()).map_err(|source| Error::Read {
                path: dir.to_path_buf(),
                source,
            })
        })
        .collect()
}

fn load_skills(root: &Path) -> Result<BTreeMap<AssetId, Asset>> {
    let skills_dir = root.join("assets/skills");
    list_dir(&skills_dir)?
        .iter()
        .filter(|path| path.is_dir())
        .map(|dir| {
            let path = dir.join("SKILL.md");
            let source = read_text(&path)?;
            let asset = Asset::parse(&source, &path)?;
            ensure_kind(&asset, AssetKind::Skill)?;
            Ok((asset.id.clone(), asset))
        })
        .collect()
}

fn load_references(root: &Path) -> Result<BTreeMap<AssetId, Asset>> {
    let references_dir = root.join("assets/shared-references");
    list_dir(&references_dir)?
        .iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .map(|path| {
            let source = read_text(path)?;
            let asset = Asset::parse(&source, path)?;
            ensure_kind(&asset, AssetKind::SharedReference)?;
            Ok((asset.id.clone(), asset))
        })
        .collect()
}

/// Ensures the declared kind matches the location the asset was loaded from.
fn ensure_kind(asset: &Asset, expected: AssetKind) -> Result<()> {
    if asset.kind == expected {
        Ok(())
    } else {
        Err(Error::KindMismatch {
            path: asset.source_path.clone(),
            expected: expected.name().to_owned(),
            found: asset.kind.name().to_owned(),
        })
    }
}

fn load_bundles(root: &Path) -> Result<BTreeMap<String, Bundle>> {
    let bundles_dir = root.join("bundles");
    list_dir(&bundles_dir)?
        .iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .map(|path| {
            let source = read_text(path)?;
            let bundle = Bundle::parse(&source, path)?;
            Ok((bundle.name.clone(), bundle))
        })
        .collect()
}
