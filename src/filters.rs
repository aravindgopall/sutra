/// High-precision filtering to avoid suggesting shell builtins and meta-commands.

pub fn is_banned_base(cmd0: &str) -> bool {
    matches!(
        cmd0,
        // Shell builtins (bash/zsh)
        "alias" | "unalias" | "export" | "set" | "unset" | "typeset"
            | "declare" | "local" | "function" | "source" | "." | "eval"
            | "exec" | "trap" | "return" | "exit"
            // Directory navigation (shells typically handle these)
            | "cd" | "pushd" | "popd"
            // Process control
            | "ulimit" | "umask" | "fg" | "bg" | "jobs"
            // History and line editing
            | "history" | "bindkey"
            // Other meta
            | "builtin" | "command" | "type" | "hash"
    )
}

pub fn candidate_is_allowed(canonical: &str) -> bool {
    let base = canonical.split_whitespace().next().unwrap_or("");
    if base.is_empty() {
        return false;
    }
    !is_banned_base(base)
}

/// Determine subcommand depth for known commands that support multi-level subcommands.
/// Returns the max token count to preserve for replacement (1–3).
///
/// Examples:
/// - `git remote add origin ...` → 3 (git, remote, add)
/// - `kubectl get pods -n kube-system` → 3 (kubectl, get, pods)
/// - `ls -la` → 2 (command + 1 subcommand)
pub fn get_subcommand_depth(cmd0: &str) -> usize {
    match cmd0 {
        // Multi-level subcommand systems: preserve up to 3 tokens
        "git" | "kubectl" | "docker" | "aws" | "gcloud" => 3,
        // Default to 2 tokens (command + 1 subcommand)
        _ => 2,
    }
}

/// Detect if a command is destructive and should require confirmation.
pub fn is_destructive(canonical: &str) -> bool {
    let base = canonical.split_whitespace().next().unwrap_or("");

    // Simple cases: straightforward destructive commands
    if matches!(
        base,
        "rm" | "rmdir" | "dd" | "mkfs" | "wipefs" | "shred" | "srm" | "kill" | "killall"
    ) {
        return true;
    }

    // Conditional: git operations that are risky
    if base == "git" {
        return canonical.contains("reset --hard")
            || canonical.contains("rebase -i")
            || canonical.contains("push --force")
            || canonical.contains("force-push");
    }

    // Conditional: package manager removals
    if matches!(base, "apt" | "yum" | "dnf" | "pacman") {
        return canonical.contains("remove") || canonical.contains("uninstall");
    }

    // Conditional: sudo with risky commands
    if base == "sudo" {
        return canonical.contains("rm") || canonical.contains("dd") || canonical.contains("mkfs");
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_builtins() {
        assert!(!candidate_is_allowed("alias foo='bar'"));
        assert!(!candidate_is_allowed("export PATH=$PATH:/usr/local/bin"));
        assert!(!candidate_is_allowed("cd /tmp"));
        assert!(!candidate_is_allowed("history | grep git"));
    }

    #[test]
    fn allows_real_commands() {
        assert!(candidate_is_allowed("git status"));
        assert!(candidate_is_allowed("kubectl get pods"));
        assert!(candidate_is_allowed("ls -la"));
        assert!(candidate_is_allowed("python script.py"));
    }

    #[test]
    fn detects_destructive() {
        assert!(is_destructive("rm -rf /tmp/test"));
        assert!(is_destructive("git reset --hard HEAD"));
        assert!(is_destructive("sudo rm -rf /"));
        assert!(!is_destructive("git status"));
        assert!(!is_destructive("ls -la"));
    }

    #[test]
    fn subcommand_depths() {
        assert_eq!(get_subcommand_depth("git"), 3);
        assert_eq!(get_subcommand_depth("kubectl"), 3);
        assert_eq!(get_subcommand_depth("ls"), 2);
        assert_eq!(get_subcommand_depth("grep"), 2);
    }
}
