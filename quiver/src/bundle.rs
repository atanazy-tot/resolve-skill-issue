//! Bundle manifests and pure bundle resolution.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;

use crate::asset::{AssetId, Picks};
use crate::{Error, Result};

/// A parsed bundle manifest.
#[derive(Clone, Debug, Deserialize)]
pub struct Bundle {
    /// The bundle name.
    pub name: String,
    /// What the bundle is for.
    pub description: String,
    /// Other bundles included wholesale.
    #[serde(default)]
    pub includes: Vec<String>,
    /// Top-level skill ids to install.
    #[serde(default)]
    pub assets: Vec<String>,
    /// Pre-declared composition picks.
    #[serde(default)]
    pub picks: BTreeMap<String, PickValue>,
}

impl Bundle {
    /// Parses a bundle from manifest text.
    pub fn parse(source: &str, path: &Path) -> Result<Self> {
        toml::from_str(source).map_err(|error| Error::BundleManifestInvalid {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })
    }

    /// The manifest picks, normalised and validated: slot name to chosen ids.
    pub fn picks_normalised(&self) -> Result<Picks> {
        self.picks
            .iter()
            .map(|(slot, value)| {
                let ids = match value {
                    PickValue::One(id) => vec![AssetId::new(id)?],
                    PickValue::Many(ids) => {
                        ids.iter().map(AssetId::new).collect::<Result<Vec<_>>>()?
                    }
                };
                Ok((slot.clone(), ids))
            })
            .collect()
    }
}

/// A manifest pick: one id (one-of) or a list (any).
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum PickValue {
    /// A single pick.
    One(String),
    /// Several picks.
    Many(Vec<String>),
}

/// Expands a bundle into the full ordered list of skill ids, detecting include cycles.
pub fn resolve(bundles: &BTreeMap<String, Bundle>, name: &str) -> Result<Vec<AssetId>> {
    let mut visiting = Vec::new();
    let mut visited = BTreeSet::new();
    let mut output = Vec::new();
    visit(bundles, name, &mut visiting, &mut visited, &mut output)?;
    let mut seen = BTreeSet::new();
    output.retain(|id| seen.insert(id.clone()));
    Ok(output)
}

/// The depth-first worker behind [`resolve`], threading explicit accumulators.
fn visit(
    bundles: &BTreeMap<String, Bundle>,
    name: &str,
    visiting: &mut Vec<String>,
    visited: &mut BTreeSet<String>,
    output: &mut Vec<AssetId>,
) -> Result<()> {
    if visited.contains(name) {
        return Ok(());
    }
    if visiting.iter().any(|entry| entry == name) {
        let cycle = visiting
            .iter()
            .cloned()
            .chain(std::iter::once(name.to_owned()))
            .collect::<Vec<_>>()
            .join(" -> ");
        return Err(Error::BundleCycle { cycle });
    }
    let bundle = bundles.get(name).ok_or_else(|| Error::UnknownBundle {
        name: name.to_owned(),
    })?;
    visiting.push(name.to_owned());
    for include in &bundle.includes {
        visit(bundles, include, visiting, visited, output)?;
    }
    for asset in &bundle.assets {
        output.push(AssetId::new(asset)?);
    }
    visiting.pop();
    visited.insert(name.to_owned());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle(name: &str, includes: &[&str], assets: &[&str]) -> (String, Bundle) {
        let manifest = Bundle {
            name: name.to_owned(),
            description: String::from("test"),
            includes: includes.iter().map(|i| i.to_string()).collect(),
            assets: assets.iter().map(|a| a.to_string()).collect(),
            picks: BTreeMap::new(),
        };
        (name.to_owned(), manifest)
    }

    #[test]
    fn resolves_nested_includes_in_order() {
        let bundles = BTreeMap::from([
            bundle("base", &[], &["git-conventions"]),
            bundle("python", &["base"], &["programmer"]),
        ]);
        let ids = resolve(&bundles, "python");
        match ids {
            Ok(ids) => assert_eq!(
                ids.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                vec!["git-conventions", "programmer"]
            ),
            Err(error) => panic!("expected resolution, got {error}"),
        }
    }

    #[test]
    fn detects_include_cycles() {
        let bundles = BTreeMap::from([bundle("a", &["b"], &[]), bundle("b", &["a"], &[])]);
        assert!(matches!(
            resolve(&bundles, "a"),
            Err(Error::BundleCycle { .. })
        ));
    }

    #[test]
    fn rejects_unknown_includes() {
        let bundles = BTreeMap::from([bundle("a", &["ghost"], &[])]);
        assert!(matches!(
            resolve(&bundles, "a"),
            Err(Error::UnknownBundle { .. })
        ));
    }

    #[test]
    fn dedupes_assets_keeping_first_occurrence() {
        let bundles = BTreeMap::from([
            bundle("base", &[], &["one"]),
            bundle("top", &["base"], &["one", "two"]),
        ]);
        match resolve(&bundles, "top") {
            Ok(ids) => assert_eq!(
                ids.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                vec!["one", "two"]
            ),
            Err(error) => panic!("expected resolution, got {error}"),
        }
    }

    #[test]
    fn parses_manifest_with_picks() {
        let manifest = "name = \"python\"\ndescription = \"d\"\nassets = [\"programmer\"]\n\n[picks]\nlang-guide = \"lang-python\"\nparadigm = [\"paradigm-fp\"]\n";
        let parsed = Bundle::parse(manifest, Path::new("bundles/python.toml"));
        match parsed {
            Ok(parsed) => {
                let picks = match parsed.picks_normalised() {
                    Ok(picks) => picks,
                    Err(error) => panic!("expected valid picks, got {error}"),
                };
                assert_eq!(picks["lang-guide"][0].as_str(), "lang-python");
                assert_eq!(picks["paradigm"][0].as_str(), "paradigm-fp");
            }
            Err(error) => panic!("expected a parsed bundle, got {error}"),
        }
    }
}
