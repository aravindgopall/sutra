# Sutra

**Sutra** is a common command suggestor for your terminal.

When you type a wrong command (e.g. `gti stats`), Sutra looks at:
- a **default catalog** of known commands/subcommands
- executables available in your **PATH**
- your **command history** (optional under smriti flag)

…and shows clickable-style numbered suggestions (1/2/3). You pick one and Sutra runs it.

**Smriti Mode** makes Sutra smarter by using your history with **frequency + recency** boosting.

---

## Features

- **Typo correction** using edit-distance (Damerau–Levenshtein)
- **Token-aware suggestions** (command + subcommand)
- **Smriti Mode** (history priors: frequency + recency + optional cwd affinity)
- **Defaults catalog** (`~/.sutra/defaults.json`) for curated command families/subcommands
- **PATH discovery** for real environment-aware suggestions
- **Plugin subcommands**: `git-foo` → suggests `git foo`, `kubectl-foo` → suggests `kubectl foo`
- **Fast retrieval** using a **BK-tree** (edit-distance radius search)
- **Shell hooks** (bash/zsh) to trigger automatically on `command not found`

---

## Install

### Install from upstream
```bash
cargo install isutra
```

### Build from source
```bash
git clone https://github.com/aravindgopall/sutra
cd sutra
cargo build --release
sudo cp target/release/sutra /usr/local/bin/sutra
````

Verify:

```bash
sutra --help
```

---

## Quick Start

### 1) Suggest for a typo (interactive)

```bash
sutra --smriti suggest --input "gti stats" --interactive
```

Example output:

```
Sutra: I couldn't run: "gti stats"
Did you mean:
  1) git status  [defaults | 0.88]
  2) git stash   [defaults | 0.71]
  3) git         [PATH | 0.62]
  0) cancel
Pick 1-3 (0 to cancel):
```

### 2) Run through Sutra

```bash
sutra run -- gti stats
```

If the command succeeds, Sutra appends it to Smriti history.

---

## Smriti Mode (history learning)

Smriti Mode boosts suggestions based on:

* **frequency** (what you run often)
* **recency** (what you ran recently)
* **cwd affinity** (optional, lightweight boost if used in the same directory)

Enable it using:

```bash
sutra --smriti suggest --input "..." --interactive
```

History file:

* `~/.sutra/history.jsonl` (JSON lines)

---

## Defaults Catalog

Create:

* `~/.sutra/defaults.json`

Example:

```json
{
  "families": [
    {
      "base": "git",
      "aliases": ["g"],
      "subcommands": ["status", "checkout", "commit", "push", "pull", "branch", "log", "diff", "stash"],
      "patterns": ["git checkout <branch>", "git commit -m <msg>"]
    },
    {
      "base": "kubectl",
      "aliases": ["k"],
      "subcommands": ["get", "describe", "apply", "delete", "logs", "exec", "config", "context"],
      "patterns": ["kubectl get <resource>", "kubectl logs <pod>"]
    }
  ]
}
```

Notes:

* `base` becomes a candidate (`git`)
* each `base + subcommand` becomes a candidate (`git status`)
* `aliases` become matching keys (`g status`)
* `patterns` are used as soft match keys (help ranking), but the executed suggestion is still a real command candidate

---

## Shell Hooks (bash/zsh)

To auto-run Sutra on unknown commands:

### Print hooks

```bash
sutra hooks
```

### Bash

Add to `~/.bashrc`:

```bash
command_not_found_handle() {
  local input="$*"
  sutra --smriti suggest --input "$input" --interactive
  return 127
}
```

### Zsh

Add to `~/.zshrc`:

```zsh
command_not_found_handler() {
  local input="$*"
  sutra --smriti suggest --input "$input" --interactive
  return 127
}
```

Reload your shell:

```bash
source ~/.bashrc   # or source ~/.zshrc
```

Now try a typo:

```bash
gti stats
```

---

## How it works (high level)

1. **Candidate generation**

   * PATH executables (`git`, `docker`, ...)
   * Defaults catalog families + subcommands
   * Smriti-mined commands/subcommands from history
   * Plugin expansion (`git-foo` → `git foo`, `kubectl-foo` → `kubectl foo`)

2. **Fast shortlist retrieval**

   * Build a **BK-tree** over candidate keys
   * Query with edit-distance radius (default `2`)

3. **Scoring**

   * Typo similarity (normalized edit distance)
   * Token-aware similarity
   * Smriti prior (freq + recency + cwd boost) if enabled
   * Context boost (light heuristics, e.g. git repo → boost `git`)

4. **Diversification**

   * Avoid near-duplicate suggestions

5. **Selection**

   * User picks `1/2/3` to execute

---

## CLI Reference

### Suggest

```bash
sutra [--smriti] [--topk N] [--radius R] suggest --input "<cmd>" [--interactive]
```

### Run

```bash
sutra [--smriti] [--topk N] [--radius R] run -- <cmd...> [--interactive]
```

### Hooks

```bash
sutra hooks
```

Flags:

* `--smriti` : enable history priors
* `--topk`   : number of suggestions (default: 3)
* `--radius` : BK-tree edit-distance radius (default: 2)

---

## Roadmap

* SLM model training to do this better.
* PATH scan caching for faster startup
* Argument-shape fixing (keep args, only correct command/subcommand tokens)
* Better diversification per command family
* Optional TUI selector (arrows/enter)