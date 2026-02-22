use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use chrono::{DateTime, Utc};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Parser, Debug)]
#[command(name = "sutra", about = "Sutra: guided command suggestions (Smriti mode remembers).")]
struct Cli {
    /// Enable Smriti mode (history-based ranking)
    #[arg(long, default_value_t = false)]
    smriti: bool,

    /// Number of suggestions to show
    #[arg(long, default_value_t = 3)]
    topk: usize,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Suggest for a possibly-wrong input. Optionally run by selecting 1/2/3.
    Suggest {
        /// The raw user input string (e.g. "gti stats")
        #[arg(long)]
        input: String,

        /// If set, after suggesting, ask user to choose 1/2/3 and execute
        #[arg(long, default_value_t = false)]
        interactive: bool,
    },

    /// Run a command via Sutra. If it fails, suggest alternatives and optionally run.
    Run {
        /// The command line to run (use -- to pass through)
        #[arg(required = true, trailing_var_arg = true)]
        args: Vec<String>,

        /// On failure, prompt for 1/2/3 and execute selection
        #[arg(long, default_value_t = true)]
        interactive: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEntry {
    ts: DateTime<Utc>,
    cmd: String,
    cwd: String,
}

#[derive(Debug, Clone)]
struct Candidate {
    canonical: String,      // what we run (e.g. "git status")
    keys: Vec<String>,      // match keys (e.g. ["git status","git","status"])
    source: SourceKind,
}

#[derive(Debug, Clone, Copy)]
enum SourceKind {
    Path,
    Smriti,
    Defaults,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Suggest { input, interactive } => {
            let suggestions = suggest(&input, cli.smriti, cli.topk)?;
            print_suggestions(&input, &suggestions);
            if interactive {
                interactive_pick_and_run(&input, &suggestions, cli.smriti)?;
            }
        }
        Cmd::Run { args, interactive } => {
            let input = args.join(" ");
            let status = run_shell(&input)?;
            if status.success() {
                append_history(&input)?;
                return Ok(());
            }

            // If failure, suggest
            let suggestions = suggest(&input, cli.smriti, cli.topk)?;
            if suggestions.is_empty() {
                eprintln!("No suggestions.");
                return Ok(());
            }

            print_suggestions(&input, &suggestions);
            if interactive {
                interactive_pick_and_run(&input, &suggestions, cli.smriti)?;
            }
        }
    }

    Ok(())
}

fn print_suggestions(input: &str, suggestions: &[ScoredCandidate]) {
    eprintln!("\nSutra: I couldn't run: \"{}\"", input);
    if suggestions.is_empty() {
        eprintln!("No close matches found.");
        return;
    }
    eprintln!("Did you mean:");
    for (i, s) in suggestions.iter().enumerate() {
        let reason = match s.cand.source {
            SourceKind::Smriti => " (Smriti)",
            SourceKind::Path => " (PATH)",
            SourceKind::Defaults => " (defaults)",
        };
        eprintln!("  {}) {}{}  [score {:.3}]", i + 1, s.cand.canonical, reason, s.score);
    }
    eprintln!("  0) cancel");
}

fn interactive_pick_and_run(original: &str, suggestions: &[ScoredCandidate], smriti: bool) -> Result<()> {
    if suggestions.is_empty() {
        return Ok(());
    }
    eprint!("\nPick 1-{} (0 to cancel): ", suggestions.len());
    io::stderr().flush().ok();

    let mut line = String::new();
    io::stdin().read_line(&mut line).ok();
    let choice = line.trim().parse::<usize>().unwrap_or(0);
    if choice == 0 || choice > suggestions.len() {
        eprintln!("Cancelled.");
        return Ok(());
    }

    let selected = &suggestions[choice - 1].cand.canonical;
    eprintln!("Running: {}", selected);

    let status = run_shell(selected)?;
    if status.success() {
        append_history(selected)?;
    } else {
        eprintln!("Command failed with status: {:?}", status.code());
        // Optionally: if smriti and original differs, you could still log the original attempt.
        let _ = smriti; let _ = original;
    }
    Ok(())
}

fn run_shell(cmdline: &str) -> Result<std::process::ExitStatus> {
    // Use sh -lc for portability and to handle quoted args / pipes.
    Command::new("sh")
        .arg("-lc")
        .arg(cmdline)
        .status()
        .with_context(|| format!("failed to execute via sh: {}", cmdline))
}

#[derive(Debug, Clone)]
struct ScoredCandidate {
    cand: Candidate,
    score: f64,
}

/// Main suggestion function
fn suggest(input: &str, smriti: bool, topk: usize) -> Result<Vec<ScoredCandidate>> {
    let input_trim = input.trim();
    if input_trim.is_empty() {
        return Ok(vec![]);
    }

    let cwd = env::current_dir().ok();
    let cwd_str = cwd
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string();

    let tokens: Vec<&str> = input_trim.split_whitespace().collect();
    let cmd0 = tokens.get(0).copied().unwrap_or("");

    // 1) Build candidate pool
    let mut pool: Vec<Candidate> = Vec::new();
    pool.extend(load_path_candidates()?);
    pool.extend(load_smriti_candidates()?);
    // Hook point for curated defaults (later):
    // pool.extend(load_defaults_candidates()?);

    // 2) Build Smriti stats if enabled
    let smriti_stats = if smriti {
        Some(build_smriti_stats()?)
    } else {
        None
    };

    // 3) Score all candidates
    let mut scored: Vec<ScoredCandidate> = pool
        .into_iter()
        .map(|cand| {
            let score = score_candidate(input_trim, cmd0, &tokens, &cand, smriti_stats.as_ref(), &cwd_str);
            ScoredCandidate { cand, score }
        })
        .filter(|s| s.score >= 0.35) // threshold: tune
        .collect();

    // 4) Sort
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // 5) Diversify: avoid showing same canonical repeatedly (and avoid tiny variants)
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for s in scored {
        let key = s.cand.canonical.clone();
        if seen.insert(key) {
            out.push(s);
        }
        if out.len() >= topk {
            break;
        }
    }

    Ok(out)
}

fn score_candidate(
    input_raw: &str,
    cmd0: &str,
    input_tokens: &[&str],
    cand: &Candidate,
    smriti_stats: Option<&SmritiStats>,
    cwd: &str,
) -> f64 {
    // A) Typo similarity: best match across candidate keys
    let s_typo = cand
        .keys
        .iter()
        .map(|k| sim_norm(input_raw, k).max(sim_norm(cmd0, k)))
        .fold(0.0_f64, f64::max);

    // B) Token similarity: compare input tokens vs candidate tokens
    let cand_tokens: Vec<&str> = cand.canonical.split_whitespace().collect();
    let s_tok = token_similarity(input_tokens, &cand_tokens);

    // C) Smriti = frequency + recency (+ cwd match)
    let mut s_smriti = 0.0;
    if let Some(stats) = smriti_stats {
        if let Some(pr) = stats.priors.get(&cand.canonical) {
            s_smriti = pr.score_for_cwd(cwd);
        } else {
            // Also try matching by first token (so "git" helps "git status")
            let base = cand_tokens.get(0).copied().unwrap_or("");
            if let Some(pr) = stats.base_priors.get(base) {
                s_smriti = 0.4 * pr.score_for_cwd(cwd);
            }
        }
    }

    // D) Context: tiny repo heuristic (boost git in git repo, etc.)
    let s_ctx = context_boost(&cand.canonical);

    // Weights: Smriti mode on/off
    let (w_typo, w_tok, w_smriti, w_ctx) = if smriti_stats.is_some() {
        (0.45, 0.25, 0.20, 0.10)
    } else {
        (0.60, 0.30, 0.00, 0.10)
    };

    (w_typo * s_typo) + (w_tok * s_tok) + (w_smriti * s_smriti) + (w_ctx * s_ctx)
}

/// Normalized similarity using Damerau-Levenshtein distance.
/// Returns 1.0 for exact match, ~0.0 for far mismatch.
fn sim_norm(a: &str, b: &str) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let a_l = a.len().max(1);
    let b_l = b.len().max(1);
    let max_l = a_l.max(b_l) as f64;
    let dist = strsim::damerau_levenshtein(a, b) as f64;
    let sim = 1.0 - (dist / max_l);
    sim.clamp(0.0, 1.0)
}

/// Token-wise similarity: for each input token, find best match in candidate tokens.
/// Penalize token-count mismatch slightly.
fn token_similarity(input: &[&str], cand: &[&str]) -> f64 {
    if input.is_empty() || cand.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0;
    for t in input.iter().take(3) { // keep cheap
        let best = cand.iter().map(|c| sim_norm(t, c)).fold(0.0, f64::max);
        sum += best;
    }
    let n = input.len().min(3) as f64;
    let base = (sum / n).clamp(0.0, 1.0);

    // small penalty for token count mismatch
    let diff = (input.len() as i64 - cand.len() as i64).abs() as f64;
    let penalty = (0.06 * diff).min(0.18);
    (base - penalty).clamp(0.0, 1.0)
}

fn context_boost(canonical: &str) -> f64 {
    // Very lightweight. You can expand this later.
    let cwd = env::current_dir().ok();
    let cwd = match cwd {
        Some(p) => p,
        None => return 0.0,
    };

    // If inside git repo, boost git commands a bit.
    let is_git = has_parent_marker(&cwd, ".git");
    if is_git && canonical.starts_with("git ") || canonical == "git" {
        return 1.0;
    }
    0.0
}

fn has_parent_marker(start: &Path, marker: &str) -> bool {
    let mut cur = Some(start);
    for _ in 0..6 {
        if let Some(p) = cur {
            if p.join(marker).exists() {
                return true;
            }
            cur = p.parent();
        } else {
            break;
        }
    }
    false
}

/// Candidate set from PATH executables (first token commands).
fn load_path_candidates() -> Result<Vec<Candidate>> {
    let path = env::var("PATH").unwrap_or_default();
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for dir in path.split(':') {
        let p = Path::new(dir);
        if !p.is_dir() {
            continue;
        }
        if let Ok(rd) = fs::read_dir(p) {
            for ent in rd.flatten() {
                let fp = ent.path();
                if let Some(name) = fp.file_name().and_then(|s| s.to_str()) {
                    if seen.insert(name.to_string()) {
                        out.push(Candidate {
                            canonical: name.to_string(),
                            keys: vec![name.to_string()],
                            source: SourceKind::Path,
                        });
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Read Smriti history entries and convert to candidates.
/// We include:
/// - base command (first token)
/// - base + subcommand (first 2 tokens) as canonical, when available
fn load_smriti_candidates() -> Result<Vec<Candidate>> {
    let entries = read_history_entries().unwrap_or_default();
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for e in entries {
        let toks: Vec<&str> = e.cmd.split_whitespace().collect();
        if toks.is_empty() {
            continue;
        }
        // base command
        let base = toks[0].to_string();
        if seen.insert(base.clone()) {
            out.push(Candidate {
                canonical: base.clone(),
                keys: vec![base.clone()],
                source: SourceKind::Smriti,
            });
        }
        // base + subcommand (2 tokens)
        if toks.len() >= 2 {
            let two = format!("{} {}", toks[0], toks[1]);
            if seen.insert(two.clone()) {
                out.push(Candidate {
                    canonical: two.clone(),
                    keys: vec![two.clone(), toks[1].to_string()],
                    source: SourceKind::Smriti,
                });
            }
        }
    }
    Ok(out)
}

/// History location: ~/.sutra/history.jsonl
fn history_path() -> Result<PathBuf> {
    let mut p = home_dir().context("no home dir found")?;
    p.push(".sutra");
    fs::create_dir_all(&p).ok();
    p.push("history.jsonl");
    Ok(p)
}

fn append_history(cmd: &str) -> Result<()> {
    let hp = history_path()?;
    let cwd = env::current_dir().ok().and_then(|p| p.to_str().map(|s| s.to_string())).unwrap_or_default();

    let entry = HistoryEntry {
        ts: Utc::now(),
        cmd: cmd.to_string(),
        cwd,
    };

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(hp)?;

    writeln!(f, "{}", serde_json::to_string(&entry)?)?;
    Ok(())
}

fn read_history_entries() -> Result<Vec<HistoryEntry>> {
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

#[derive(Debug, Clone)]
struct SmritiPrior {
    freq: f64,       // normalized 0..1
    recency: f64,    // normalized 0..1
    cwd_hits: HashMap<String, usize>,
}

impl SmritiPrior {
    fn score_for_cwd(&self, cwd: &str) -> f64 {
        let cwd_boost = if cwd.is_empty() {
            0.0
        } else {
            let hits = self.cwd_hits.get(cwd).copied().unwrap_or(0) as f64;
            // saturating boost
            (hits / (hits + 3.0)).clamp(0.0, 1.0)
        };
        // Weight recency more than freq per your suggestion
        (0.35 * self.freq) + (0.55 * self.recency) + (0.10 * cwd_boost)
    }
}

#[derive(Debug)]
struct SmritiStats {
    priors: HashMap<String, SmritiPrior>,      // canonical -> prior
    base_priors: HashMap<String, SmritiPrior>, // base cmd -> prior
}

/// Build frequency + recency priors from history.
/// Recency uses exponential decay: exp(-age_seconds / tau)
fn build_smriti_stats() -> Result<SmritiStats> {
    let entries = read_history_entries().unwrap_or_default();
    if entries.is_empty() {
        return Ok(SmritiStats { priors: HashMap::new(), base_priors: HashMap::new() });
    }

    let now = Utc::now();
    let tau = 60.0 * 60.0 * 24.0 * 7.0; // 7 days half-ish window (tune)

    // We build stats for:
    // - full canonical = first 2 tokens when available else 1 token
    // - base = first token
    let mut freq_full: HashMap<String, usize> = HashMap::new();
    let mut freq_base: HashMap<String, usize> = HashMap::new();
    let mut rec_full: HashMap<String, f64> = HashMap::new();
    let mut rec_base: HashMap<String, f64> = HashMap::new();
    let mut cwd_full: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let mut cwd_base: HashMap<String, HashMap<String, usize>> = HashMap::new();

    for e in entries {
        let toks: Vec<&str> = e.cmd.split_whitespace().collect();
        if toks.is_empty() {
            continue;
        }

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

        // keep max recency (most recent highest)
        rec_full
            .entry(canonical.clone())
            .and_modify(|v| *v = v.max(r))
            .or_insert(r);

        rec_base
            .entry(base.clone())
            .and_modify(|v| *v = v.max(r))
            .or_insert(r);

        cwd_full
            .entry(canonical.clone())
            .or_insert_with(HashMap::new)
            .entry(e.cwd.clone())
            .and_modify(|v| *v += 1)
            .or_insert(1);

        cwd_base
            .entry(base.clone())
            .or_insert_with(HashMap::new)
            .entry(e.cwd.clone())
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    // Normalize frequency with log scaling
    let max_full = freq_full.values().copied().max().unwrap_or(1) as f64;
    let max_base = freq_base.values().copied().max().unwrap_or(1) as f64;

    let mut priors = HashMap::new();
    for (k, f) in freq_full {
        let f = (f as f64).ln_1p() / max_full.ln_1p();
        let r = rec_full.get(&k).copied().unwrap_or(0.0);
        priors.insert(
            k.clone(),
            SmritiPrior {
                freq: f.clamp(0.0, 1.0),
                recency: r.clamp(0.0, 1.0),
                cwd_hits: cwd_full.remove(&k).unwrap_or_default(),
            },
        );
    }

    let mut base_priors = HashMap::new();
    for (k, f) in freq_base {
        let f = (f as f64).ln_1p() / max_base.ln_1p();
        let r = rec_base.get(&k).copied().unwrap_or(0.0);
        base_priors.insert(
            k.clone(),
            SmritiPrior {
                freq: f.clamp(0.0, 1.0),
                recency: r.clamp(0.0, 1.0),
                cwd_hits: cwd_base.remove(&k).unwrap_or_default(),
            },
        );
    }

    Ok(SmritiStats { priors, base_priors })
}
