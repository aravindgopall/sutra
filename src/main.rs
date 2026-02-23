use anyhow::{Context, Result};
use std::io::{self, Write};
use std::process::Command;
use clap::Parser;

mod cli;
mod history;
mod defaults;
mod candidate;
mod smriti;
mod bktree;
mod scoring;
mod suggest;
mod hooks;
mod shell_history_import;
mod filters;
mod tui;

use cli::{Cli, Cmd, HookAction, SmritiAction};
use history::append_history;
use filters::{get_subcommand_depth, is_destructive};

fn run_shell(cmdline: &str) -> Result<std::process::ExitStatus> {
    Command::new("sh")
        .arg("-lc")
        .arg(cmdline)
        .status()
        .with_context(|| format!("failed to execute via sh: {}", cmdline))
}

/// Apply a suggested candidate to the user's original input, preserving user args.
/// 
/// This replaces the command/subcommand tokens from the suggestion while keeping
/// any trailing args the user provided. The number of tokens replaced varies based
/// on the command structure (e.g., 3 for git, kubectl; 2 for others).
///
/// Example: `input="gti chekout main"` + `canonical="git checkout"`
///          → returns `"git checkout main"`
/// 
/// Example: `input="kubectl gett pods"` + `canonical="kubectl get pods"`
///          → returns `"kubectl get pods"` (replaces 3 tokens)
fn apply_candidate_to_input(input: &str, canonical: &str) -> String {
    let in_toks: Vec<&str> = input.split_whitespace().collect();
    let cand_toks: Vec<&str> = canonical.split_whitespace().collect();

    if in_toks.is_empty() || cand_toks.is_empty() {
        return canonical.to_string();
    }

    let cmd0 = in_toks[0];
    let depth = get_subcommand_depth(cmd0);
    // Replace first N tokens (command/subcommand), where N = min(depth, len(candidate), len(input))
    let n = depth.min(cand_toks.len()).min(in_toks.len());
    let mut out: Vec<String> = Vec::new();

    // Add the candidate's tokens up to depth
    for i in 0..n {
        out.push(cand_toks[i].to_string());
    }

    // Keep remaining user args unchanged
    for i in n..in_toks.len() {
        out.push(in_toks[i].to_string());
    }

    out.join(" ")
}

fn print_suggestions(input: &str, suggestions: &[candidate::ScoredCandidate]) {
    eprintln!("\nSutra: I couldn't run: \"{}\"", input);
    if suggestions.is_empty() {
        eprintln!("No close matches found.");
        return;
    }
    eprintln!("Did you mean:");
    for (i, s) in suggestions.iter().enumerate() {
        let reason = match s.cand.source {
            candidate::SourceKind::Smriti => "Smriti",
            candidate::SourceKind::Path => "PATH",
            candidate::SourceKind::Defaults => "defaults",
            candidate::SourceKind::Plugin => "plugin",
        };
        eprintln!("  {}) {}  [{} | {:.3}]", i + 1, s.cand.canonical, reason, s.score);
    }
    eprintln!("  0) cancel");
}

fn interactive_pick_and_run(
    input: &str,
    suggestions: &[candidate::ScoredCandidate],
    use_tui: bool,
) -> Result<()> {
    if suggestions.is_empty() {
        return Ok(());
    }

    // Prefer TUI if enabled. If it fails, fall back to stdin prompt.
    if use_tui {
        match tui::pick_candidate_tui(input, suggestions) {
            Ok(Some(idx)) => return run_selected(input, &suggestions[idx]),
            Ok(None) => {
                eprintln!("Cancelled.");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Sutra: TUI failed ({e}). Falling back to numeric prompt…");
                // Only print suggestions on TUI fallback
                print_suggestions(input, suggestions);
            }
        }
    } else {
        // Non-TUI path: print suggestions before numeric prompt
        print_suggestions(input, suggestions);
    }

    // Fallback: numeric prompt
    eprint!("\nPick 1-{} (0 to cancel): ", suggestions.len());
    io::stderr().flush().ok();
    let mut line = String::new();
    io::stdin().read_line(&mut line).ok();
    let choice = line.trim().parse::<usize>().unwrap_or(0);

    if choice == 0 || choice > suggestions.len() {
        eprintln!("Cancelled.");
        return Ok(());
    }
    run_selected(input, &suggestions[choice - 1])
}

fn run_selected(input: &str, s: &candidate::ScoredCandidate) -> Result<()> {
    let exec_line = apply_candidate_to_input(input, &s.cand.canonical);
    
    // Show hint if available
    if let Some(reason) = &s.reason {
        eprintln!("  [{}]", reason.display());
    }
    
    eprintln!("Running: {}", exec_line);

    // Check if the command is destructive and require confirmation
    if is_destructive(&exec_line) {
        eprint!("⚠️  This command may be destructive. Continue? (y/n): ");
        io::stderr().flush().ok();
        let mut line = String::new();
        io::stdin().read_line(&mut line).ok();
        if !line.trim().eq_ignore_ascii_case("y") {
            eprintln!("Cancelled.");
            return Ok(());
        }
    }

    let status = run_shell(&exec_line)?;
    if status.success() {
        append_history(&exec_line)?;
    } else {
        eprintln!("Command failed with status: {:?}", status.code());
    }
    Ok(())
}

fn cmd_exists(cmd0: &str) -> bool {
    if cmd0.trim().is_empty() {
        return false;
    }
    which::which(cmd0).is_ok()
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Handle --no-tui flag: explicitly disables TUI
    let use_tui = cli.tui && !cli.no_tui;

    match cli.cmd {
        Cmd::Hooks { action } => match action {
            HookAction::Print => hooks::print_hooks(),
            HookAction::Install { bash, zsh } => hooks::install_hooks(bash, zsh)?,
        },

        Cmd::Smriti { action } => match action {
            SmritiAction::Import { bash, zsh, limit, dry_run } => {
                let import_bash = bash || (!bash && !zsh); // default both if none specified
                let import_zsh = zsh || (!bash && !zsh);

                let rep = shell_history_import::import_shell_history(import_bash, import_zsh, limit, dry_run)?;
                eprintln!(
                    "Sutra Smriti import: imported={}, skipped={}, errors={}, dry_run={}",
                    rep.imported, rep.skipped, rep.errors, dry_run
                );
            }
        },

        Cmd::Suggest { input, interactive } => {
            let suggestions = suggest::suggest(&input, cli.smriti, cli.topk, cli.radius)?;
            if interactive && use_tui {
                // TUI is the primary UI; no noisy pre-print
                interactive_pick_and_run(&input, &suggestions, true)?;
            } else {
                // Non-TUI path or non-interactive
                print_suggestions(&input, &suggestions);
                if interactive {
                    interactive_pick_and_run(&input, &suggestions, false)?;
                }
            }
        }

        Cmd::Run { args, interactive } => {
            let input = args.join(" ");

            // if the base command doesn't exist, don't execute the failing command.
            // Go straight to suggestions so we avoid "sh: ... command not found".
            let cmd0 = args.get(0).map(|s| s.as_str()).unwrap_or("");
            if !cmd_exists(cmd0) {
                let suggestions = suggest::suggest(&input, cli.smriti, cli.topk, cli.radius)?;
                if suggestions.is_empty() {
                    eprintln!("No suggestions.");
                    return Ok(());
                }

                if interactive && use_tui {
                    // TUI is the primary UI; no noisy pre-print
                    interactive_pick_and_run(&input, &suggestions, true)?;
                } else if interactive {
                    // Non-TUI interactive: show suggestions + numeric prompt
                    interactive_pick_and_run(&input, &suggestions, false)?;
                } else {
                    // Non-interactive: just show suggestions
                    print_suggestions(&input, &suggestions);
                }
                return Ok(());
            }

            let status = run_shell(&input)?;
            if status.success() {
                append_history(&input)?;
                return Ok(());
            }

            let suggestions = suggest::suggest(&input, cli.smriti, cli.topk, cli.radius)?;
            if suggestions.is_empty() {
                eprintln!("No suggestions.");
                return Ok(());
            }

            if interactive && use_tui {
                // TUI is the primary UI; no noisy pre-print
                interactive_pick_and_run(&input, &suggestions, true)?;
            } else if interactive {
                // Non-TUI interactive: show suggestions + numeric prompt
                interactive_pick_and_run(&input, &suggestions, false)?;
            } else {
                // Non-interactive: just show suggestions
                print_suggestions(&input, &suggestions);
            }
        }
    }

    Ok(())
}