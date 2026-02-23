use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use dirs::home_dir;
use std::{collections::HashSet, fs, path::PathBuf};

use crate::history::{append_entry, read_history_entries, HistoryEntry};
use crate::filters::is_banned_base;

fn bash_history_path() -> Option<PathBuf> {
    let mut p = home_dir()?;
    p.push(".bash_history");
    Some(p)
}

fn zsh_history_path() -> Option<PathBuf> {
    let mut p = home_dir()?;
    p.push(".zsh_history");
    Some(p)
}

pub struct ImportReport {
    pub imported: usize,
    pub skipped: usize,
    pub errors: usize,
}

pub fn import_shell_history(import_bash: bool, import_zsh: bool, limit: usize, dry_run: bool) -> Result<ImportReport> {
    let mut report = ImportReport { imported: 0, skipped: 0, errors: 0 };

    // Build dedupe set from existing Sutra history
    let existing = read_history_entries().unwrap_or_default();
    let mut seen_cmds: HashSet<String> = existing.into_iter().map(|e| e.cmd).collect();

    if import_zsh {
        match import_zsh_history(limit, dry_run, &mut seen_cmds) {
            Ok((imp, skip)) => { report.imported += imp; report.skipped += skip; }
            Err(_) => { report.errors += 1; }
        }
    }

    if import_bash {
        match import_bash_history(limit, dry_run, &mut seen_cmds) {
            Ok((imp, skip)) => { report.imported += imp; report.skipped += skip; }
            Err(_) => { report.errors += 1; }
        }
    }

    Ok(report)
}

fn import_zsh_history(limit: usize, dry_run: bool, seen: &mut HashSet<String>) -> Result<(usize, usize)> {
    let p = zsh_history_path().context("no home dir")?;
    if !p.exists() {
        return Ok((0, 0)); // graceful
    }

    let text = fs::read_to_string(&p).with_context(|| format!("failed reading {:?}", p))?;
    let lines: Vec<&str> = text.lines().rev().take(limit).collect(); // newest-ish first in zsh file? not guaranteed but ok
    let mut imported = 0usize;
    let mut skipped = 0usize;

    for line in lines.into_iter().rev() { // preserve older->newer among selected
        // zsh extended history commonly: ": 1700000000:0;git status"
        let cmd_opt = parse_zsh_extended(line);
        let Some((ts, cmd)) = cmd_opt else { continue; };

        let cmd = cmd.trim().to_string();
        if cmd.is_empty() { continue; }

        // Filter out shell builtins and meta-commands
        let base = cmd.split_whitespace().next().unwrap_or("");
        if is_banned_base(base) {
            skipped += 1;
            continue;
        }

        if !seen.insert(cmd.clone()) {
            skipped += 1;
            continue;
        }

        if !dry_run {
            let entry = HistoryEntry { ts, cmd, cwd: String::new() };
            append_entry(&entry)?;
        }
        imported += 1;
    }

    Ok((imported, skipped))
}

fn parse_zsh_extended(line: &str) -> Option<(DateTime<Utc>, String)> {
    let s = line.trim();
    if !s.starts_with(": ") { return None; }
    // ": <epoch>:<duration>;<cmd>"
    // find first space then first ':' etc
    let rest = s.strip_prefix(": ")?;
    let mut parts = rest.splitn(2, ';');
    let meta = parts.next()?;
    let cmd = parts.next()?.to_string();

    let mut meta_parts = meta.split(':');
    let epoch_str = meta_parts.next()?.trim();
    // meta_parts.next() is duration; ignore
    let epoch: i64 = epoch_str.parse().ok()?;
    let ts = Utc.timestamp_opt(epoch, 0).single()?;
    Some((ts, cmd))
}

fn import_bash_history(limit: usize, dry_run: bool, seen: &mut HashSet<String>) -> Result<(usize, usize)> {
    let p = bash_history_path().context("no home dir")?;
    if !p.exists() {
        return Ok((0, 0)); // graceful
    }

    let text = fs::read_to_string(&p).with_context(|| format!("failed reading {:?}", p))?;
    let all: Vec<&str> = text.lines().collect();
    let tail = all.into_iter().rev().take(limit).collect::<Vec<_>>();
    let tail = tail.into_iter().rev().collect::<Vec<_>>();

    // Approximate timestamps: spread over last 14 days (tuneable)
    let now = Utc::now();
    let window_secs = 60 * 60 * 24 * 14;
    let n = tail.len().max(1) as i64;

    let mut imported = 0usize;
    let mut skipped = 0usize;

    for (i, line) in tail.into_iter().enumerate() {
        let cmd = line.trim().to_string();
        if cmd.is_empty() { continue; }

        // Filter out shell builtins and meta-commands
        let base = cmd.split_whitespace().next().unwrap_or("");
        if is_banned_base(base) {
            skipped += 1;
            continue;
        }

        if !seen.insert(cmd.clone()) {
            skipped += 1;
            continue;
        }

        // Older lines get older timestamps
        let frac = i as f64 / (n as f64);
        let age = (window_secs as f64 * (1.0 - frac)) as i64;
        let ts = now - chrono::Duration::seconds(age.max(0));

        if !dry_run {
            let entry = HistoryEntry { ts, cmd, cwd: String::new() };
            append_entry(&entry)?;
        }
        imported += 1;
    }

    Ok((imported, skipped))
}