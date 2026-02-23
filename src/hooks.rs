use anyhow::{Context, Result};
use dirs::home_dir;
use std::{fs, path::PathBuf};

const START: &str = "# >>> sutra hooks >>>";
const END: &str = "# <<< sutra hooks <<<";

pub fn hook_block_bash() -> String {
    format!(r#"{START}
command_not_found_handle() {{
  local input="$*"
  sutra --smriti suggest --input "$input" --interactive
  return 127
}}
{END}
"#)
}

pub fn hook_block_zsh() -> String {
    format!(r#"{START}
command_not_found_handler() {{
  local input="$*"
  sutra --smriti suggest --input "$input" --interactive
  return 127
}}
{END}
"#)
}

pub fn print_hooks() {
    println!("\n# ---- Sutra hooks (bash) ----\n{}", hook_block_bash());
    println!("\n# ---- Sutra hooks (zsh) ----\n{}", hook_block_zsh());
}

pub fn install_hooks(install_bash: bool, install_zsh: bool) -> Result<()> {
    // If neither specified, auto-pick based on $SHELL (best effort) but allow both
    let (bash, zsh) = if !install_bash && !install_zsh {
        let shell = std::env::var("SHELL").unwrap_or_default();
        if shell.contains("zsh") { (false, true) }
        else if shell.contains("bash") { (true, false) }
        else { (true, true) } // unknown: install both
    } else {
        (install_bash, install_zsh)
    };

    if bash {
        if let Err(e) = install_into_rc(".bashrc", &hook_block_bash()) {
            eprintln!("Sutra: couldn't install into ~/.bashrc ({e}). You can run `sutra hooks print` and paste manually.");
        } else {
            eprintln!("Sutra: installed hooks into ~/.bashrc");
        }
    }

    if zsh {
        if let Err(e) = install_into_rc(".zshrc", &hook_block_zsh()) {
            eprintln!("Sutra: couldn't install into ~/.zshrc ({e}). You can run `sutra hooks print` and paste manually.");
        } else {
            eprintln!("Sutra: installed hooks into ~/.zshrc");
        }
    }

    Ok(())
}

fn rc_path(name: &str) -> Result<PathBuf> {
    let mut p = home_dir().context("no home dir")?;
    p.push(name);
    Ok(p)
}

fn install_into_rc(rc_name: &str, block: &str) -> Result<()> {
    let p = rc_path(rc_name)?;
    let existing = fs::read_to_string(&p).unwrap_or_default();

    if existing.contains(START) && existing.contains(END) {
        // already installed; keep idempotent
        return Ok(());
    }

    // backup if file exists and non-empty
    if p.exists() && !existing.is_empty() {
        let bak = p.with_extension(format!("{}.bak", rc_name.trim_start_matches('.')));
        // If backup fails, continue but warn
        if let Err(e) = fs::write(&bak, &existing) {
            eprintln!("Sutra: warning: couldn't write backup {:?} ({e})", bak);
        }
    }

    let mut new_content = existing;
    if !new_content.ends_with('\n') && !new_content.is_empty() {
        new_content.push('\n');
    }
    new_content.push_str("\n");
    new_content.push_str(block);

    fs::write(&p, new_content).with_context(|| format!("failed writing {:?}", p))?;
    Ok(())
}