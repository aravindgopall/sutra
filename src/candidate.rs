use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy)]
pub enum SourceKind {
    Path,
    Smriti,
    Defaults,
    Plugin,
}

#[derive(Debug, Clone)]
pub enum SuggestionReason {
    ClosestMatch,
    SmritiFrequent { count: usize, last_used_secs: u64 },
    RepoContext { context: String },
    DefaultsKnown,
}

impl SuggestionReason {
    pub fn display(&self) -> String {
        match self {
            SuggestionReason::ClosestMatch => "closest match".to_string(),
            SuggestionReason::SmritiFrequent { count, last_used_secs } => {
                let time_str = format_time_ago(*last_used_secs);
                format!("Smriti: used {} times, {} ago", count, time_str)
            }
            SuggestionReason::RepoContext { context } => {
                format!("repo context: {}", context)
            }
            SuggestionReason::DefaultsKnown => {
                "in defaults catalog (frequently used)".to_string()
            }
        }
    }
}

fn format_time_ago(secs: u64) -> String {
    match secs {
        s if s < 60 => format!("{}s ago", s),
        s if s < 3600 => format!("{}m ago", s / 60),
        s if s < 86400 => format!("{}h ago", s / 3600),
        s => format!("{}d ago", s / 86400),
    }
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub canonical: String,
    pub keys: Vec<String>,
    pub source: SourceKind,
}

#[derive(Debug, Clone)]
pub struct ScoredCandidate {
    pub cand: Candidate,
    pub score: f64,
    pub reason: Option<SuggestionReason>,
}

pub fn prefer_source(a: SourceKind, b: SourceKind) -> SourceKind {
    use SourceKind::*;
    let rank = |s| match s {
        Defaults => 4,
        Smriti => 3,
        Plugin => 2,
        Path => 1,
    };
    if rank(b) > rank(a) { b } else { a }
}

pub fn dedupe_candidates(candidates: Vec<Candidate>) -> Vec<Candidate> {
    let mut by_canon: HashMap<String, Candidate> = HashMap::new();

    for mut c in candidates {
        match by_canon.get_mut(&c.canonical) {
            Some(existing) => {
                let mut set: HashSet<String> = existing.keys.drain(..).collect();
                set.extend(c.keys.drain(..));
                existing.keys = set.into_iter().collect();
                existing.source = prefer_source(existing.source, c.source);
            }
            None => {
                by_canon.insert(c.canonical.clone(), c);
            }
        }
    }

    by_canon.into_values().collect()
}