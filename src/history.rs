use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub ts: DateTime<Utc>,
    pub cmd: String,
    pub cwd: String,
}

pub fn history_path() -> Result<PathBuf> {
    let mut p = home_dir().context("no home dir found")?;
    p.push(".sutra");
    fs::create_dir_all(&p).ok();
    p.push("history.jsonl");
    Ok(p)
}

pub fn append_history(cmd: &str) -> Result<()> {
    let _hp = history_path()?;
    let cwd = env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let entry = HistoryEntry {
        ts: Utc::now(),
        cmd: cmd.to_string(),
        cwd,
    };

    append_entry(&entry)
}

pub fn append_entry(entry: &HistoryEntry) -> Result<()> {
    let hp = history_path()?;
    let mut f = OpenOptions::new().create(true).append(true).open(hp)?;
    writeln!(f, "{}", serde_json::to_string(entry)?)?;
    Ok(())
}

pub fn read_history_entries() -> Result<Vec<HistoryEntry>> {
    let hp = history_path()?;
    if !hp.exists() {
        return Ok(vec![]);
    }
    let f = fs::File::open(hp)?;
    let br = BufReader::new(f);
    let mut out = Vec::new();
    for line in br.lines().flatten() {
        if let Ok(e) = serde_json::from_str::<HistoryEntry>(&line) {
            out.push(e);
        }
    }
    Ok(out)
}