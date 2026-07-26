//! quiver — distribute AI assets from a central registry into repos.

mod asset;
mod bundle;
mod cli;
mod error;
mod hash;
mod install;
mod lockfile;
mod registry;
mod render;
mod result;

use std::path::Path;
use std::process::ExitCode;

use clap::Parser;

use self::cli::{Cli, Command, InstallArgs, ListArgs, StatusArgs};
use self::error::Error;
use self::lockfile::{Drift, Lockfile, drift_of};
use self::registry::Registry;
use self::render::{Target, parse_picks};
use self::result::Result;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// The imperative shell: parses, dispatches, prints, and sets the exit code.
fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    match cli.command {
        Command::List(args) => list(&args),
        Command::Install(args) => install(&args),
        Command::Status(args) => status(&args),
    }
}

/// Lists the registry's skills, shared-references, and bundles.
fn list(args: &ListArgs) -> Result<ExitCode> {
    let registry = Registry::load(Path::new(&args.registry))?;
    println!("skills:");
    for skill in registry.skills.values() {
        println!("  {:<16} {}", skill.id.as_str(), skill.description);
    }
    println!("shared-references:");
    for reference in registry.references.values() {
        println!("  {:<16} {}", reference.id.as_str(), reference.description);
    }
    println!("bundles:");
    for bundle in registry.bundles.values() {
        println!(
            "  {:<16} {} ({} assets)",
            bundle.name,
            bundle.description,
            bundle.assets.len()
        );
    }
    Ok(ExitCode::SUCCESS)
}

/// Installs a bundle or skill into the destination repo.
fn install(args: &InstallArgs) -> Result<ExitCode> {
    let registry = Registry::load(Path::new(&args.registry))?;
    let targets = parse_targets(&args.targets)?;
    let picks = parse_picks(&args.picks)?;
    let reports = install::install(
        &registry,
        &args.name,
        &targets,
        &picks,
        Path::new(&args.dest),
    )?;
    for report in &reports {
        println!("installed {} ({} files)", report.id, report.files.len());
        for path in &report.files {
            println!("  {}", path.display());
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// Reports drift per managed file; fails the exit code when any drift is found.
fn status(args: &StatusArgs) -> Result<ExitCode> {
    let dest = Path::new(&args.dest);
    let lockfile = match Lockfile::read(dest)? {
        Some(lockfile) => lockfile,
        None => {
            println!("nothing installed (no lockfile)");
            return Ok(ExitCode::SUCCESS);
        }
    };
    let rows: Vec<(String, String, Drift)> = lockfile
        .installed
        .iter()
        .flat_map(|asset| {
            asset.files.iter().map(|file| {
                let contents = std::fs::read_to_string(dest.join(&file.path)).ok();
                (
                    asset.id.clone(),
                    file.path.to_string_lossy().into_owned(),
                    drift_of(contents.as_deref(), &file.sha256),
                )
            })
        })
        .collect();
    for (id, path, drift) in &rows {
        let label = match drift {
            Drift::Clean => "ok",
            Drift::Modified => "modified",
            Drift::Missing => "missing",
        };
        println!("{label:<9} {id} {path}");
    }
    let drifted = rows.iter().any(|(_, _, drift)| *drift != Drift::Clean);
    if drifted {
        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

/// Parses the comma-separated `--targets` flag.
fn parse_targets(text: &str) -> Result<Vec<Target>> {
    text.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(Target::parse)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comma_separated_targets() {
        let targets = parse_targets("claude, opencode");
        match targets {
            Ok(targets) => assert_eq!(targets, vec![Target::Claude, Target::Opencode]),
            Err(error) => panic!("expected targets, got {error}"),
        }
        assert!(matches!(
            parse_targets("emacs"),
            Err(Error::InvalidTarget { .. })
        ));
    }
}
