//! The destination-side lockfile: types, IO, and pure drift detection.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::hash::sha256_hex;
use crate::{Error, Result};

/// The lockfile's file name at the destination root.
pub const FILE_NAME: &str = "ai-assets.lock.json";

/// The lockfile recording what was installed, where, and with which content.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Lockfile {
    /// The schema version.
    pub version: u32,
    /// Where the assets came from.
    pub registry: RegistryInfo,
    /// The installed assets, sorted by id.
    pub installed: Vec<InstalledAsset>,
}

/// Provenance of the installed assets.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegistryInfo {
    /// The registry source (a local path for now; a git URL once transport lands).
    pub path: String,
}

/// One installed asset and every file it manages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstalledAsset {
    /// The asset id.
    pub id: String,
    /// The asset kind.
    pub kind: String,
    /// The bundle it arrived via, if any.
    pub bundle: Option<String>,
    /// The targets it was rendered for.
    pub targets: Vec<String>,
    /// The composition picks used.
    pub picks: BTreeMap<String, Vec<String>>,
    /// Every managed file with its rendered hash.
    pub files: Vec<ManagedFile>,
}

/// A managed file and its rendered content hash.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ManagedFile {
    /// The path relative to the destination root.
    pub path: PathBuf,
    /// The SHA-256 of the rendered contents at install time.
    pub sha256: String,
}

impl Lockfile {
    /// An empty lockfile at the current schema version.
    pub fn empty(registry_path: &str) -> Self {
        Self {
            version: 1,
            registry: RegistryInfo {
                path: registry_path.to_owned(),
            },
            installed: Vec::new(),
        }
    }

    /// Reads the lockfile from the destination root; `None` when none exists yet.
    pub fn read(dest: &Path) -> Result<Option<Self>> {
        let path = dest.join(FILE_NAME);
        match fs::read_to_string(&path) {
            Ok(text) => {
                serde_json::from_str(&text)
                    .map(Some)
                    .map_err(|error| Error::LockfileInvalid {
                        path,
                        reason: error.to_string(),
                    })
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(Error::Read { path, source }),
        }
    }

    /// Writes the lockfile to the destination root.
    pub fn write(&self, dest: &Path) -> Result<()> {
        let path = dest.join(FILE_NAME);
        let text = serde_json::to_string_pretty(self).map_err(|error| Error::Internal {
            reason: format!("cannot serialise the lockfile: {error}"),
        })?;
        fs::write(&path, text).map_err(|source| Error::Write { path, source })
    }

    /// Replaces or inserts the entry for the given asset, keeping entries sorted.
    pub fn upsert(&mut self, asset: InstalledAsset) {
        self.installed.retain(|existing| existing.id != asset.id);
        self.installed.push(asset);
        self.installed.sort_by(|a, b| a.id.cmp(&b.id));
    }
}

/// The drift state of one managed file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Drift {
    /// The contents match the lockfile.
    Clean,
    /// The contents differ from the lockfile.
    Modified,
    /// The file is gone.
    Missing,
}

/// Computes the drift state of some file contents against the expected hash.
pub fn drift_of(contents: Option<&str>, expected_sha256: &str) -> Drift {
    match contents {
        None => Drift::Missing,
        Some(text) if sha256_hex(text) == expected_sha256 => Drift::Clean,
        Some(_) => Drift::Modified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drift_detection_covers_all_states() {
        let hash = sha256_hex("managed contents");
        assert_eq!(drift_of(None, &hash), Drift::Missing);
        assert_eq!(drift_of(Some("managed contents"), &hash), Drift::Clean);
        assert_eq!(drift_of(Some("edited contents"), &hash), Drift::Modified);
    }

    #[test]
    fn upsert_replaces_and_sorts() {
        let mut lockfile = Lockfile::empty(".");
        let entry = |id: &str| InstalledAsset {
            id: id.to_owned(),
            kind: String::from("skill"),
            bundle: None,
            targets: Vec::new(),
            picks: BTreeMap::new(),
            files: Vec::new(),
        };
        lockfile.upsert(entry("zeta"));
        lockfile.upsert(entry("alpha"));
        lockfile.upsert(entry("zeta"));
        let ids: Vec<&str> = lockfile.installed.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, vec!["alpha", "zeta"]);
    }
}
