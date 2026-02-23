use std::{env, path::Path};

use crate::{candidate::Candidate, smriti::SmritiStats};

pub fn damerau(a: &str, b: &str) -> usize {
    strsim::damerau_levenshtein(a, b)
}

pub fn sim_norm(a: &str, b: &str) -> f64 {
    if a.is_empty() || b.is_empty() { return 0.0; }
    let max_l = (a.len().max(b.len()) as f64).max(1.0);
    let dist = damerau(a, b) as f64;
    (1.0 - dist / max_l).clamp(0.0, 1.0)
}

pub fn token_similarity(input: &[&str], cand: &[&str]) -> f64 {
    if input.is_empty() || cand.is_empty() { return 0.0; }
    let mut sum = 0.0;
    for t in input.iter().take(3) {
        let best = cand.iter().map(|c| sim_norm(t, c)).fold(0.0, f64::max);
        sum += best;
    }
    let n = input.len().min(3) as f64;
    let base = (sum / n).clamp(0.0, 1.0);

    let diff = (input.len() as i64 - cand.len() as i64).abs() as f64;
    let penalty = (0.06 * diff).min(0.18);
    (base - penalty).clamp(0.0, 1.0)
}

fn has_parent_marker(start: &Path, marker: &str) -> bool {
    let mut cur = Some(start);
    for _ in 0..6 {
        if let Some(p) = cur {
            if p.join(marker).exists() { return true; }
            cur = p.parent();
        } else { break; }
    }
    false
}

fn context_boost(canonical: &str) -> f64 {
    let cwd = match env::current_dir().ok() {
        Some(p) => p,
        None => return 0.0,
    };
    let is_git = has_parent_marker(&cwd, ".git");
    if is_git && (canonical == "git" || canonical.starts_with("git ")) {
        return 1.0;
    }
    0.0
}

pub fn score_candidate(
    input_raw: &str,
    cmd0: &str,
    input_tokens: &[&str],
    cand: &Candidate,
    smriti_stats: Option<&SmritiStats>,
    cwd: &str,
) -> f64 {
    let s_typo = cand.keys.iter()
        .map(|k| sim_norm(input_raw, k).max(sim_norm(cmd0, k)))
        .fold(0.0, f64::max);

    let cand_tokens: Vec<&str> = cand.canonical.split_whitespace().collect();
    let s_tok = token_similarity(input_tokens, &cand_tokens);

    let mut s_smriti = 0.0;
    if let Some(stats) = smriti_stats {
        if let Some(pr) = stats.priors.get(&cand.canonical) {
            s_smriti = pr.score_for_cwd(cwd);
        } else if let Some(base) = cand_tokens.get(0) {
            if let Some(pr) = stats.base_priors.get(*base) {
                s_smriti = 0.35 * pr.score_for_cwd(cwd);
            }
        }
    }

    let s_ctx = context_boost(&cand.canonical);

    let (w_typo, w_tok, w_smriti, w_ctx) = if smriti_stats.is_some() {
        (0.45, 0.25, 0.20, 0.10)
    } else {
        (0.60, 0.30, 0.00, 0.10)
    };

    (w_typo * s_typo) + (w_tok * s_tok) + (w_smriti * s_smriti) + (w_ctx * s_ctx)
}