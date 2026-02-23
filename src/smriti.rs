use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;

use crate::history::{read_history_entries, HistoryEntry};

#[derive(Debug, Clone)]
pub struct SmritiPrior {
    pub freq: f64,
    pub recency: f64,
    pub cwd_hits: HashMap<String, usize>,
}

impl SmritiPrior {
    pub fn score_for_cwd(&self, cwd: &str) -> f64 {
        let cwd_boost = if cwd.is_empty() {
            0.0
        } else {
            let hits = self.cwd_hits.get(cwd).copied().unwrap_or(0) as f64;
            (hits / (hits + 3.0)).clamp(0.0, 1.0)
        };
        (0.30 * self.freq) + (0.60 * self.recency) + (0.10 * cwd_boost)
    }
}

#[derive(Debug)]
pub struct SmritiStats {
    pub priors: HashMap<String, SmritiPrior>,      // "git status"
    pub base_priors: HashMap<String, SmritiPrior>, // "git"
}

pub fn build_smriti_stats() -> Result<SmritiStats> {
    let entries = read_history_entries().unwrap_or_default();
    if entries.is_empty() {
        return Ok(SmritiStats { priors: HashMap::new(), base_priors: HashMap::new() });
    }
    build_stats_from_entries(&entries)
}

fn build_stats_from_entries(entries: &[HistoryEntry]) -> Result<SmritiStats> {
    let now = Utc::now();
    let tau = 60.0 * 60.0 * 24.0 * 7.0; // 7 days

    let mut freq_full: HashMap<String, usize> = HashMap::new();
    let mut freq_base: HashMap<String, usize> = HashMap::new();
    let mut rec_full: HashMap<String, f64> = HashMap::new();
    let mut rec_base: HashMap<String, f64> = HashMap::new();
    let mut cwd_full: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let mut cwd_base: HashMap<String, HashMap<String, usize>> = HashMap::new();

    for e in entries {
        let toks: Vec<&str> = e.cmd.split_whitespace().collect();
        if toks.is_empty() { continue; }

        let base = toks[0].to_string();
        let canonical = if toks.len() >= 2 {
            format!("{} {}", toks[0], toks[1])
        } else {
            base.clone()
        };

        *freq_full.entry(canonical.clone()).or_insert(0) += 1;
        *freq_base.entry(base.clone()).or_insert(0) += 1;

        let age = (now - e.ts).num_seconds().max(0) as f64;
        let r = (-age / tau).exp().clamp(0.0, 1.0);

        rec_full.entry(canonical.clone()).and_modify(|v| *v = v.max(r)).or_insert(r);
        rec_base.entry(base.clone()).and_modify(|v| *v = v.max(r)).or_insert(r);

        cwd_full.entry(canonical.clone()).or_insert_with(HashMap::new)
            .entry(e.cwd.clone()).and_modify(|v| *v += 1).or_insert(1);

        cwd_base.entry(base.clone()).or_insert_with(HashMap::new)
            .entry(e.cwd.clone()).and_modify(|v| *v += 1).or_insert(1);
    }

    let max_full = freq_full.values().copied().max().unwrap_or(1) as f64;
    let max_base = freq_base.values().copied().max().unwrap_or(1) as f64;

    let mut priors = HashMap::new();
    for (k, f) in freq_full {
        let f = (f as f64).ln_1p() / max_full.ln_1p();
        let r = rec_full.get(&k).copied().unwrap_or(0.0);
        priors.insert(k.clone(), SmritiPrior {
            freq: f.clamp(0.0, 1.0),
            recency: r.clamp(0.0, 1.0),
            cwd_hits: cwd_full.remove(&k).unwrap_or_default(),
        });
    }

    let mut base_priors = HashMap::new();
    for (k, f) in freq_base {
        let f = (f as f64).ln_1p() / max_base.ln_1p();
        let r = rec_base.get(&k).copied().unwrap_or(0.0);
        base_priors.insert(k.clone(), SmritiPrior {
            freq: f.clamp(0.0, 1.0),
            recency: r.clamp(0.0, 1.0),
            cwd_hits: cwd_base.remove(&k).unwrap_or_default(),
        });
    }

    Ok(SmritiStats { priors, base_priors })
}