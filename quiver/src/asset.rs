//! Asset types and their parsing from canonical Markdown.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{Error, Result};

/// Normalised composition picks: slot name to chosen shared-reference ids.
pub type Picks = BTreeMap<String, Vec<AssetId>>;

/// A validated asset identifier, e.g. `audit-skills`.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AssetId(String);

impl AssetId {
    /// Validates and constructs an asset id.
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let id = id.into();
        if is_valid_id(&id) {
            Ok(Self(id))
        } else {
            Err(Error::InvalidAssetId {
                id,
                reason: String::from(
                    "expected 1-64 chars of lowercase alphanumerics separated by single hyphens",
                ),
            })
        }
    }

    /// The id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The kind of an asset, declared in frontmatter.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum AssetKind {
    /// A discoverable skill installed into tool directories.
    Skill,
    /// A library document installed only via composition.
    SharedReference,
}

impl AssetKind {
    /// The kebab-case name used in frontmatter.
    pub fn name(&self) -> &'static str {
        match *self {
            Self::Skill => "skill",
            Self::SharedReference => "shared-reference",
        }
    }
}

/// A composition slot specification from frontmatter.
#[derive(Clone, Debug, Deserialize)]
pub struct SlotSpec {
    /// The tag group candidates are drawn from.
    pub tag: String,
}

/// A parsed canonical asset: frontmatter metadata plus the raw body.
#[derive(Clone, Debug)]
pub struct Asset {
    /// The validated identifier (equals the directory or file-stem name).
    pub id: AssetId,
    /// The declared kind.
    pub kind: AssetKind,
    /// The discovery description.
    pub description: String,
    /// Search and discovery tags.
    pub tags: Vec<String>,
    /// Fixed shared-reference dependencies.
    pub requires: Vec<AssetId>,
    /// Exactly-one composition slots.
    pub requires_one_of: BTreeMap<String, SlotSpec>,
    /// Zero-or-more composition slots.
    pub requires_any: BTreeMap<String, SlotSpec>,
    /// The Markdown body (everything after the frontmatter).
    pub body: String,
    /// The canonical source text, hashed into markers.
    pub source: String,
    /// Where the canonical file lives.
    pub source_path: PathBuf,
}

impl Asset {
    /// Parses an asset from the canonical file text at the given path.
    pub fn parse(source: &str, source_path: &Path) -> Result<Self> {
        let (frontmatter, body) = split_frontmatter(source, source_path)?;
        let parsed: Frontmatter =
            serde_yaml::from_str(&frontmatter).map_err(|error| Error::FrontmatterInvalid {
                path: source_path.to_path_buf(),
                reason: error.to_string(),
            })?;
        let id = AssetId::new(parsed.name.clone())?;
        let expected = conventional_id(source_path);
        if id.as_str() != expected {
            return Err(Error::NameMismatch {
                path: source_path.to_path_buf(),
                name: parsed.name,
                expected,
            });
        }
        let description = parsed.description.trim().to_owned();
        let requires = parsed
            .requires
            .iter()
            .map(AssetId::new)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            id,
            kind: parsed.kind,
            description,
            tags: parsed.tags,
            requires,
            requires_one_of: parsed.requires_one_of,
            requires_any: parsed.requires_any,
            body: body.to_owned(),
            source: source.to_owned(),
            source_path: source_path.to_path_buf(),
        })
    }
}

/// Splits canonical text into the frontmatter block and the body after it.
fn split_frontmatter<'source>(source: &'source str, path: &Path) -> Result<(String, &'source str)> {
    let after_open = source
        .strip_prefix("---\n")
        .ok_or_else(|| Error::FrontmatterMissing {
            path: path.to_path_buf(),
        })?;
    match after_open.find("\n---\n") {
        Some(end) => {
            let frontmatter = &after_open[..end];
            let body = &after_open[end + "\n---\n".len()..];
            Ok((frontmatter.to_owned(), body))
        }
        None => Err(Error::FrontmatterMissing {
            path: path.to_path_buf(),
        }),
    }
}

/// The id a path implies: the directory name for `SKILL.md`, else the file stem.
fn conventional_id(path: &Path) -> String {
    if path.file_name().is_some_and(|name| name == "SKILL.md") {
        path.parent()
            .and_then(|parent| parent.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        path.file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

/// Whether the text is a valid id: `^[a-z0-9]+(-[a-z0-9]+)*$`, at most 64 chars.
fn is_valid_id(id: &str) -> bool {
    let valid_part = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    };
    !id.is_empty() && id.len() <= 64 && id.split('-').all(valid_part)
}

// serde types.
#[derive(Deserialize)]
struct Frontmatter {
    kind: AssetKind,
    name: String,
    description: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default, rename = "requires-one-of")]
    requires_one_of: BTreeMap<String, SlotSpec>,
    #[serde(default, rename = "requires-any")]
    requires_any: BTreeMap<String, SlotSpec>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SKILL: &str = "---\nkind: skill\nname: demo-skill\ndescription: >\n  Does demo things. USE WHEN demoing.\ntags: [meta]\nrequires: [descriptions]\nrequires-one-of:\n  lang-guide: { tag: lang-guide }\n---\n# Demo\n\nBody text.\n";

    #[test]
    fn parses_valid_skill() {
        let asset = Asset::parse(SKILL, Path::new("assets/skills/demo-skill/SKILL.md"));
        let asset = match asset {
            Ok(asset) => asset,
            Err(error) => panic!("expected a parsed asset, got {error}"),
        };
        assert_eq!(asset.id.as_str(), "demo-skill");
        assert_eq!(asset.kind, AssetKind::Skill);
        assert_eq!(asset.description, "Does demo things. USE WHEN demoing.");
        assert_eq!(asset.requires.len(), 1);
        assert!(asset.requires_one_of.contains_key("lang-guide"));
        assert_eq!(asset.body, "# Demo\n\nBody text.\n");
    }

    #[test]
    fn rejects_missing_frontmatter() {
        let result = Asset::parse("# No frontmatter\n", Path::new("assets/skills/x/SKILL.md"));
        assert!(matches!(result, Err(Error::FrontmatterMissing { .. })));
    }

    #[test]
    fn rejects_name_mismatch() {
        let result = Asset::parse(SKILL, Path::new("assets/skills/other-name/SKILL.md"));
        assert!(matches!(result, Err(Error::NameMismatch { .. })));
    }

    #[test]
    fn validates_ids() {
        assert!(AssetId::new("audit-skills").is_ok());
        assert!(AssetId::new("a").is_ok());
        assert!(AssetId::new("-leading").is_err());
        assert!(AssetId::new("trailing-").is_err());
        assert!(AssetId::new("double--hyphen").is_err());
        assert!(AssetId::new("Upper").is_err());
        assert!(AssetId::new("").is_err());
    }

    #[test]
    fn parses_shared_reference_by_stem() {
        let source = "---\nkind: shared-reference\nname: descriptions\ndescription: A playbook.\ntags: [skill-authoring]\n---\n# Playbook\n";
        let asset = Asset::parse(
            source,
            Path::new("assets/shared-references/descriptions.md"),
        );
        let asset = match asset {
            Ok(asset) => asset,
            Err(error) => panic!("expected a parsed asset, got {error}"),
        };
        assert_eq!(asset.id.as_str(), "descriptions");
        assert_eq!(asset.kind, AssetKind::SharedReference);
    }
}
