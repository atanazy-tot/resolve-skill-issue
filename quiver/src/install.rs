//! Planning and executing installs: the orchestration between pure core and disk.

use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::{AssetId, Picks};
use crate::bundle;
use crate::hash::sha256_hex;
use crate::lockfile::{InstalledAsset, Lockfile, ManagedFile};
use crate::registry::Registry;
use crate::render::{RenderedFile, Target, render_skill};
use crate::{Error, Result};

/// The outcome of installing one asset.
pub struct InstallReport {
    /// The asset id.
    pub id: String,
    /// The files written, as paths relative to the destination root.
    pub files: Vec<PathBuf>,
}

/// What the positional install name resolved to.
enum Resolution {
    Bundle {
        ids: Vec<AssetId>,
        name: String,
        picks: Picks,
    },
    Skill {
        id: AssetId,
    },
}

/// Resolves a name as a bundle or a skill, rejecting ambiguity and absence.
fn resolve_name(registry: &Registry, name: &str) -> Result<Resolution> {
    let bundle = registry.bundles.get(name);
    let skill_id = AssetId::new(name)
        .ok()
        .filter(|id| registry.skills.contains_key(id));
    match (bundle, skill_id) {
        (Some(_), Some(_)) => Err(Error::AmbiguousName {
            name: name.to_owned(),
        }),
        (Some(bundle), None) => {
            let ids = bundle::resolve(&registry.bundles, name)?;
            let picks = bundle.picks_normalised()?;
            Ok(Resolution::Bundle {
                ids,
                name: name.to_owned(),
                picks,
            })
        }
        (None, Some(id)) => Ok(Resolution::Skill { id }),
        (None, None) => Err(Error::UnknownAssetOrBundle {
            name: name.to_owned(),
        }),
    }
}

/// CLI picks override bundle picks, slot by slot.
fn merge_picks(base: &Picks, overrides: &Picks) -> Picks {
    overrides
        .iter()
        .fold(base.clone(), |mut merged, (slot, ids)| {
            merged.insert(slot.clone(), ids.clone());
            merged
        })
}

/// Installs a bundle or a single skill into the destination repo.
pub fn install(
    registry: &Registry,
    name: &str,
    targets: &[Target],
    cli_picks: &Picks,
    dest: &Path,
) -> Result<Vec<InstallReport>> {
    let resolution = resolve_name(registry, name)?;
    let (skill_ids, bundle_name, bundle_picks) = match resolution {
        Resolution::Bundle { ids, name, picks } => (ids, Some(name), picks),
        Resolution::Skill { id } => (vec![id], None, Picks::new()),
    };
    let picks = merge_picks(&bundle_picks, cli_picks);
    let mut lockfile =
        Lockfile::read(dest)?.unwrap_or_else(|| Lockfile::empty(&registry.root.to_string_lossy()));
    let mut reports = Vec::new();
    for id in &skill_ids {
        let skill = registry.skills.get(id).ok_or_else(|| Error::UnknownAsset {
            id: id.as_str().to_owned(),
        })?;
        let mut files = Vec::new();
        for target in targets {
            files.extend(render_skill(registry, skill, *target, &picks)?);
        }
        for file in &files {
            write_file(dest, file)?;
        }
        let managed = files
            .iter()
            .map(|file| ManagedFile {
                path: file.path.clone(),
                sha256: sha256_hex(&file.contents),
            })
            .collect();
        lockfile.upsert(InstalledAsset {
            id: id.as_str().to_owned(),
            kind: String::from("skill"),
            bundle: bundle_name.clone(),
            targets: targets
                .iter()
                .map(|target| target.name().to_owned())
                .collect(),
            picks: picks
                .iter()
                .map(|(slot, ids)| {
                    (
                        slot.clone(),
                        ids.iter().map(|id| id.as_str().to_owned()).collect(),
                    )
                })
                .collect(),
            files: managed,
        });
        reports.push(InstallReport {
            id: id.as_str().to_owned(),
            files: files.iter().map(|file| file.path.clone()).collect(),
        });
    }
    lockfile.write(dest)?;
    Ok(reports)
}

/// Writes one rendered file, creating parent directories as needed.
fn write_file(dest: &Path, file: &RenderedFile) -> Result<()> {
    let path = dest.join(&file.path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(&path, &file.contents).map_err(|source| Error::Write { path, source })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_picks_override_bundle_picks() {
        let base = Picks::from([(
            String::from("lang-guide"),
            vec![AssetId::new("lang-python").unwrap_or_else(|e| panic!("valid id: {e}"))],
        )]);
        let overrides = Picks::from([(
            String::from("lang-guide"),
            vec![AssetId::new("lang-rust").unwrap_or_else(|e| panic!("valid id: {e}"))],
        )]);
        let merged = merge_picks(&base, &overrides);
        assert_eq!(merged["lang-guide"][0].as_str(), "lang-rust");
    }
}
