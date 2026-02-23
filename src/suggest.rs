use anyhow::Result;
use std::collections::{HashSet};
use std::{env, fs, path::Path};

use crate::{
    bktree::BkTree,
    candidate::{Candidate, ScoredCandidate, SourceKind, SuggestionReason, dedupe_candidates},
    defaults::load_defaults_catalog,
    filters::candidate_is_allowed,
    history::read_history_entries,
    smriti::build_smriti_stats,
    scoring::{damerau, score_candidate},
};

fn load_path_executables() -> Result<HashSet<String>> {
    let path = env::var("PATH").unwrap_or_default();
    let mut seen = HashSet::new();

    for dir in path.split(':') {
        let p = Path::new(dir);
        if !p.is_dir() { continue; }
        if let Ok(rd) = fs::read_dir(p) {
            for ent in rd.flatten() {
                let fp = ent.path();
                if let Some(name) = fp.file_name().and_then(|s| s.to_str()) {
                    seen.insert(name.to_string());
                }
            }
        }
    }
    Ok(seen)
}

fn candidates_from_path_base(exes: &HashSet<String>) -> Vec<Candidate> {
    exes.iter().map(|name| Candidate {
        canonical: name.clone(),
        keys: vec![name.clone()],
        source: SourceKind::Path,
    }).collect()
}

fn plugin_subcommands_from_path(exes: &HashSet<String>, prefixes: &[&str]) -> Vec<Candidate> {
    let mut out = Vec::new();
    for &base in prefixes {
        let pref = format!("{}-", base);
        for e in exes.iter() {
            if let Some(rest) = e.strip_prefix(&pref) {
                let canonical = format!("{} {}", base, rest);
                let keys = vec![canonical.clone(), rest.to_string(), base.to_string(), e.clone()];
                out.push(Candidate { canonical, keys, source: SourceKind::Plugin });
            }
        }
    }
    out
}

fn candidates_from_defaults() -> Vec<Candidate> {
    let defaults = load_defaults_catalog();
    let mut out = Vec::new();
    for fam in defaults.families {
        // base
        let mut base_keys = vec![fam.base.clone()];
        base_keys.extend(fam.aliases.clone());
        out.push(Candidate { canonical: fam.base.clone(), keys: base_keys, source: SourceKind::Defaults });

        // base + subcmd
        for sc in fam.subcommands {
            let canonical = format!("{} {}", fam.base, sc);
            let mut keys = vec![canonical.clone(), sc.clone(), fam.base.clone()];
            keys.extend(fam.aliases.iter().map(|a| format!("{} {}", a, sc)));
            out.push(Candidate { canonical, keys, source: SourceKind::Defaults });
        }

        // patterns (soft keys)
        for pat in fam.patterns {
            out.push(Candidate { canonical: fam.base.clone(), keys: vec![pat], source: SourceKind::Defaults });
        }
    }
    out
}

fn candidates_from_smriti() -> Vec<Candidate> {
    let history = read_history_entries().unwrap_or_default();
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for e in history {
        let toks: Vec<&str> = e.cmd.split_whitespace().collect();
        if toks.is_empty() { continue; }

        let base = toks[0].to_string();
        if seen.insert(base.clone()) {
            out.push(Candidate { canonical: base.clone(), keys: vec![base.clone()], source: SourceKind::Smriti });
        }

        if toks.len() >= 2 {
            let two = format!("{} {}", toks[0], toks[1]);
            if seen.insert(two.clone()) {
                out.push(Candidate {
                    canonical: two.clone(),
                    keys: vec![two.clone(), toks[1].to_string(), toks[0].to_string()],
                    source: SourceKind::Smriti,
                });
            }
        }
    }
    out
}

pub fn suggest(input: &str, smriti: bool, topk: usize, radius: usize) -> Result<Vec<ScoredCandidate>> {
    let input_trim = input.trim();
    if input_trim.is_empty() { return Ok(vec![]); }

    let cwd_str = env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let tokens: Vec<&str> = input_trim.split_whitespace().collect();
    let cmd0 = tokens.get(0).copied().unwrap_or("");

    let exes = load_path_executables()?;

    let mut candidates = Vec::new();
    candidates.extend(candidates_from_defaults());
    candidates.extend(candidates_from_path_base(&exes));
    candidates.extend(plugin_subcommands_from_path(&exes, &["git", "kubectl"]));
    candidates.extend(candidates_from_smriti());

    let candidates = dedupe_candidates(candidates);

    // Apply high-precision filtering to remove shell builtins and meta-commands
    let candidates: Vec<Candidate> = candidates
        .into_iter()
        .filter(|c| candidate_is_allowed(&c.canonical))
        .collect();

    let mut tree = BkTree::new();
    for (i, c) in candidates.iter().enumerate() {
        for k in &c.keys {
            tree.insert(k.to_string(), i, &damerau);
        }
    }

    let mut hits: Vec<usize> = Vec::new();
    tree.search(cmd0, radius, &mut hits, &damerau);
    if tokens.len() >= 2 {
        let two = format!("{} {}", tokens[0], tokens[1]);
        tree.search(&two, radius, &mut hits, &damerau);
    }
    tree.search(input_trim, radius, &mut hits, &damerau);

    // uniq ids
    let mut uniq = HashSet::new();
    hits.retain(|id| uniq.insert(*id));

    // adaptive widen
    if hits.len() < 10 {
        let r2 = (radius + 1).min(3);
        let mut more = Vec::new();
        tree.search(cmd0, r2, &mut more, &damerau);
        for id in more { uniq.insert(id); }
        hits = uniq.into_iter().collect();
    }

    let smriti_stats = if smriti { Some(build_smriti_stats()?) } else { None };

    let mut scored: Vec<ScoredCandidate> = hits.into_iter()
        .map(|id| {
            let cand = candidates[id].clone();
            let score = score_candidate(input_trim, cmd0, &tokens, &cand, smriti_stats.as_ref(), &cwd_str);
            
            // Generate a reason for the suggestion based on source and context
            let reason = match cand.source {
                SourceKind::Defaults => Some(SuggestionReason::DefaultsKnown),
                SourceKind::Smriti => {
                    // In the future, could look up actual frequency/last-used from smriti_stats
                    Some(SuggestionReason::SmritiFrequent { count: 0, last_used_secs: 0 })
                }
                SourceKind::Plugin => Some(SuggestionReason::RepoContext { context: "git/kubectl plugin".to_string() }),
                SourceKind::Path => Some(SuggestionReason::ClosestMatch),
            };
            
            ScoredCandidate { cand, score, reason }
        })
        .filter(|s| s.score >= 0.35)
        .collect();

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // diversify by base a bit
    let mut out = Vec::new();
    let mut seen_bases = HashSet::new();
    let mut seen_canon = HashSet::new();

    for s in scored {
        if !seen_canon.insert(s.cand.canonical.clone()) { continue; }
        let base = s.cand.canonical.split_whitespace().next().unwrap_or("").to_string();
        if out.len() < 2 && seen_bases.contains(&base) { continue; }
        seen_bases.insert(base);
        out.push(s);
        if out.len() >= topk { break; }
    }

    Ok(out)
}