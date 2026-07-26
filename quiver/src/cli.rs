//! Command-line interface definitions.

use clap::{Parser, Subcommand};

/// quiver — distribute AI assets from a central registry into repos.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// The available commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// List assets and bundles in the registry.
    List(ListArgs),
    /// Install a bundle or skill into a destination repo.
    Install(InstallArgs),
    /// Show drift between the lockfile and installed files.
    Status(StatusArgs),
}

/// Arguments for `quiver list`.
#[derive(Debug, Parser)]
pub struct ListArgs {
    /// The registry path.
    #[arg(long, default_value = ".")]
    pub registry: String,
}

/// Arguments for `quiver install`.
#[derive(Debug, Parser)]
pub struct InstallArgs {
    /// The bundle or skill name.
    pub name: String,
    /// The targets to render for (comma-separated: claude,opencode).
    #[arg(long, default_value = "claude,opencode")]
    pub targets: String,
    /// The destination repo root.
    #[arg(long, default_value = ".")]
    pub dest: String,
    /// The registry path.
    #[arg(long, default_value = ".")]
    pub registry: String,
    /// A composition pick in the form slot=asset-id (repeatable).
    #[arg(long = "pick")]
    pub picks: Vec<String>,
}

/// Arguments for `quiver status`.
#[derive(Debug, Parser)]
pub struct StatusArgs {
    /// The destination repo root.
    #[arg(long, default_value = ".")]
    pub dest: String,
}
