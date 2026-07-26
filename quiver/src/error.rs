//! All errors which can occur while running quiver.

use std::path::PathBuf;

/// The crate's error type, preserving concrete sources for robust handling.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot find frontmatter in {path}")]
    FrontmatterMissing { path: PathBuf },
    #[error("cannot parse frontmatter in {path}: {reason}")]
    FrontmatterInvalid { path: PathBuf, reason: String },
    #[error("cannot parse bundle manifest {path}: {reason}")]
    BundleManifestInvalid { path: PathBuf, reason: String },
    #[error("cannot parse lockfile {path}: {reason}")]
    LockfileInvalid { path: PathBuf, reason: String },
    #[error("invalid asset id '{id}': {reason}")]
    InvalidAssetId { id: String, reason: String },
    #[error("asset name '{name}' does not match the path convention '{expected}' in {path}")]
    NameMismatch {
        path: PathBuf,
        name: String,
        expected: String,
    },
    #[error("asset in {path} is declared as '{found}' but its location requires '{expected}'")]
    KindMismatch {
        path: PathBuf,
        expected: String,
        found: String,
    },
    #[error("unknown asset '{id}'")]
    UnknownAsset { id: String },
    #[error("unknown bundle '{name}'")]
    UnknownBundle { name: String },
    #[error("'{name}' matches both a bundle and a skill; rename one of them")]
    AmbiguousName { name: String },
    #[error("'{name}' matches neither a bundle nor a skill in the registry")]
    UnknownAssetOrBundle { name: String },
    #[error("bundle include cycle detected: {cycle}")]
    BundleCycle { cycle: String },
    #[error(
        "cannot resolve slot '{slot}' of '{asset}': pass --pick {slot}=<id> (interactive resolution is not yet implemented)"
    )]
    MissingPick { asset: String, slot: String },
    #[error("invalid pick '{pick}': expected the form <slot>=<asset-id>")]
    InvalidPick { pick: String },
    #[error("pick '{id}' for slot '{slot}' does not carry the tag '{tag}'")]
    PickTagMismatch {
        slot: String,
        id: String,
        tag: String,
    },
    #[error("unknown target '{name}': expected 'claude' or 'opencode'")]
    InvalidTarget { name: String },
    #[error("internal error: {reason}")]
    Internal { reason: String },
}
