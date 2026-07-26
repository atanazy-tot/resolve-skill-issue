//! Pure rendering of canonical assets into target-native files.

use std::path::PathBuf;

use crate::asset::{Asset, AssetId, Picks};
use crate::hash::sha256_hex;
use crate::registry::Registry;
use crate::{Error, Result};

/// A tool target assets are rendered for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Target {
    /// Claude Code (`.claude/skills/`).
    Claude,
    /// opencode (`.opencode/skills/`).
    Opencode,
}

impl Target {
    /// Parses a target from its CLI name.
    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "claude" => Ok(Self::Claude),
            "opencode" => Ok(Self::Opencode),
            other => Err(Error::InvalidTarget {
                name: other.to_owned(),
            }),
        }
    }

    /// The directory skills are rendered into, relative to the destination root.
    pub fn skills_dir(&self) -> PathBuf {
        match *self {
            Self::Claude => PathBuf::from(".claude/skills"),
            Self::Opencode => PathBuf::from(".opencode/skills"),
        }
    }

    /// The stable name used in lockfiles and reports.
    pub fn name(&self) -> &'static str {
        match *self {
            Self::Claude => "claude",
            Self::Opencode => "opencode",
        }
    }
}

/// A single file ready to be written to a destination.
#[derive(Clone, Debug)]
pub struct RenderedFile {
    /// The path relative to the destination root.
    pub path: PathBuf,
    /// The full file contents.
    pub contents: String,
}

/// The marker inserted into every managed file for drift detection and provenance.
pub fn marker(id: &AssetId, source: &str) -> String {
    format!(
        "<!-- quiver:managed {}@sha256:{} -->",
        id.as_str(),
        sha256_hex(source)
    )
}

/// Parses CLI `--pick slot=id` flags into normalised picks.
pub fn parse_picks(flags: &[String]) -> Result<Picks> {
    let picks = flags.iter().map(|flag| {
        let (slot, id) = flag
            .split_once('=')
            .ok_or_else(|| Error::InvalidPick { pick: flag.clone() })?;
        Ok((slot.to_owned(), AssetId::new(id)?))
    });
    let mut grouped = Picks::new();
    for pick in picks {
        let (slot, id) = pick?;
        grouped.entry(slot).or_default().push(id);
    }
    Ok(grouped)
}

/// Renders a skill and all its composition products for the given target.
pub fn render_skill(
    registry: &Registry,
    skill: &Asset,
    target: Target,
    picks: &Picks,
) -> Result<Vec<RenderedFile>> {
    let skill_dir = target.skills_dir().join(skill.id.as_str());
    let files =
        {
            let mut files = Vec::new();
            files.push(RenderedFile {
                path: skill_dir.join("SKILL.md"),
                contents: render_skill_markdown(skill),
            });
            for required in &skill.requires {
                let reference = registry.reference(required)?;
                files.push(RenderedFile {
                    path: skill_dir
                        .join("references")
                        .join(format!("{}.md", required.as_str())),
                    contents: render_reference_markdown(required, reference),
                });
            }
            for (slot, spec) in &skill.requires_one_of {
                let picked = picks.get(slot).and_then(|ids| ids.first()).ok_or_else(|| {
                    Error::MissingPick {
                        asset: skill.id.as_str().to_owned(),
                        slot: slot.clone(),
                    }
                })?;
                let reference = tagged_reference(registry, slot, picked, &spec.tag)?;
                files.push(RenderedFile {
                    path: skill_dir.join("references").join(format!("{slot}.md")),
                    contents: render_reference_markdown(picked, reference),
                });
            }
            for (slot, spec) in &skill.requires_any {
                let ids = picks.get(slot).cloned().unwrap_or_default();
                for id in &ids {
                    let reference = tagged_reference(registry, slot, id, &spec.tag)?;
                    files.push(RenderedFile {
                        path: skill_dir
                            .join("references")
                            .join(slot)
                            .join(format!("{}.md", id.as_str())),
                        contents: render_reference_markdown(id, reference),
                    });
                }
            }
            files
        };
    Ok(files)
}

/// A shared-reference carrying the slot's tag, or a pick-mismatch error.
fn tagged_reference<'registry>(
    registry: &'registry Registry,
    slot: &str,
    id: &AssetId,
    tag: &str,
) -> Result<&'registry Asset> {
    let reference = registry.reference(id)?;
    if reference.tags.iter().any(|candidate| candidate == tag) {
        Ok(reference)
    } else {
        Err(Error::PickTagMismatch {
            slot: slot.to_owned(),
            id: id.as_str().to_owned(),
            tag: tag.to_owned(),
        })
    }
}

/// The target-native `SKILL.md`: native frontmatter, marker, then the body.
fn render_skill_markdown(skill: &Asset) -> String {
    format!(
        "---\nname: {}\ndescription: {}\n---\n{}\n{}",
        skill.id.as_str(),
        yaml_double_quoted(&skill.description),
        marker(&skill.id, &skill.source),
        skill.body.trim_start_matches('\n')
    )
}

/// A rendered shared-reference: marker, then the body (frontmatter stripped).
fn render_reference_markdown(id: &AssetId, reference: &Asset) -> String {
    format!(
        "{}\n{}",
        marker(id, &reference.source),
        reference.body.trim_start_matches('\n')
    )
}

/// A YAML double-quoted scalar, robust against any single-line content.
fn yaml_double_quoted(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::asset::{AssetKind, SlotSpec};

    fn reference(id: &str, tags: &[&str]) -> Asset {
        Asset {
            id: AssetId::new(id).unwrap_or_else(|e| panic!("valid id: {e}")),
            kind: AssetKind::SharedReference,
            description: String::from("A reference."),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            requires: Vec::new(),
            requires_one_of: BTreeMap::new(),
            requires_any: BTreeMap::new(),
            body: String::from("# Guide\n\nContent.\n"),
            source: format!("source-of-{id}"),
            source_path: PathBuf::from(format!("assets/shared-references/{id}.md")),
        }
    }

    fn skill() -> Asset {
        Asset {
            id: AssetId::new("demo-skill").unwrap_or_else(|e| panic!("valid id: {e}")),
            kind: AssetKind::Skill,
            description: String::from("Does: demo things. USE WHEN demoing."),
            tags: vec![String::from("meta")],
            requires: vec![
                AssetId::new("descriptions").unwrap_or_else(|e| panic!("valid id: {e}")),
            ],
            requires_one_of: BTreeMap::from([(
                String::from("lang-guide"),
                SlotSpec {
                    tag: String::from("lang-guide"),
                },
            )]),
            requires_any: BTreeMap::new(),
            body: String::from("# Demo\n\nBody.\n"),
            source: String::from("canonical-source"),
            source_path: PathBuf::from("assets/skills/demo-skill/SKILL.md"),
        }
    }

    fn registry() -> Registry {
        Registry {
            skills: BTreeMap::new(),
            references: BTreeMap::from([
                (
                    AssetId::new("descriptions").unwrap_or_else(|e| panic!("valid id: {e}")),
                    reference("descriptions", &["skill-authoring"]),
                ),
                (
                    AssetId::new("lang-rust").unwrap_or_else(|e| panic!("valid id: {e}")),
                    reference("lang-rust", &["lang-guide"]),
                ),
            ]),
            bundles: BTreeMap::new(),
            root: PathBuf::from("."),
        }
    }

    #[test]
    fn target_parsing_is_exact() {
        assert_eq!(Target::parse("claude").ok(), Some(Target::Claude));
        assert_eq!(Target::parse("opencode").ok(), Some(Target::Opencode));
        assert!(matches!(
            Target::parse("cursor"),
            Err(Error::InvalidTarget { .. })
        ));
        assert_eq!(Target::Claude.skills_dir(), PathBuf::from(".claude/skills"));
        assert_eq!(
            Target::Opencode.skills_dir(),
            PathBuf::from(".opencode/skills")
        );
    }

    #[test]
    fn renders_native_frontmatter_without_quiver_fields() {
        let rendered = render_skill_markdown(&skill());
        assert!(rendered.starts_with("---\nname: demo-skill\ndescription: \"Does: demo things. USE WHEN demoing.\"\n---\n<!-- quiver:managed demo-skill@sha256:"));
        assert!(rendered.ends_with("# Demo\n\nBody.\n"));
        assert!(!rendered.contains("requires"));
        assert!(!rendered.contains("kind:"));
    }

    #[test]
    fn renders_requires_flat_into_references() {
        let picks = Picks::from([(
            String::from("lang-guide"),
            vec![AssetId::new("lang-rust").unwrap_or_else(|e| panic!("valid id: {e}"))],
        )]);
        let files = match render_skill(&registry(), &skill(), Target::Opencode, &picks) {
            Ok(files) => files,
            Err(error) => panic!("expected rendered files, got {error}"),
        };
        let paths: Vec<String> = files
            .iter()
            .map(|f| f.path.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            paths,
            vec![
                ".opencode/skills/demo-skill/SKILL.md",
                ".opencode/skills/demo-skill/references/descriptions.md",
                ".opencode/skills/demo-skill/references/lang-guide.md",
            ]
        );
    }

    #[test]
    fn missing_one_of_pick_is_an_error() {
        let result = render_skill(&registry(), &skill(), Target::Claude, &Picks::new());
        assert!(matches!(result, Err(Error::MissingPick { .. })));
    }

    #[test]
    fn pick_without_the_slot_tag_is_an_error() {
        let picks = Picks::from([(
            String::from("lang-guide"),
            vec![AssetId::new("descriptions").unwrap_or_else(|e| panic!("valid id: {e}"))],
        )]);
        let result = render_skill(&registry(), &skill(), Target::Claude, &picks);
        assert!(matches!(result, Err(Error::PickTagMismatch { .. })));
    }

    #[test]
    fn parses_pick_flags() {
        let picks = parse_picks(&[
            String::from("lang-guide=lang-rust"),
            String::from("paradigm=paradigm-fp"),
        ]);
        match picks {
            Ok(picks) => assert_eq!(picks["lang-guide"][0].as_str(), "lang-rust"),
            Err(error) => panic!("expected picks, got {error}"),
        }
        assert!(matches!(
            parse_picks(&[String::from("no-equals")]),
            Err(Error::InvalidPick { .. })
        ));
    }
}
