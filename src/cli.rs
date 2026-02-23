use clap::{Parser, Subcommand, ArgAction};

#[derive(Parser, Debug)]
#[command(name = "sutra", about = "Sutra: guided command suggestions (Smriti mode remembers).")]
pub struct Cli {
    #[arg(long, default_value_t = false)]
    pub smriti: bool,
    #[arg(long, default_value_t = 3)]
    pub topk: usize,
    #[arg(long, default_value_t = 2)]
    pub radius: usize,

    /// Enable TUI mode (default: true unless --no-tui is passed)
    #[arg(long, action = ArgAction::SetTrue, default_value_t = true, overrides_with = "no_tui")]
    pub tui: bool,

    /// Disable TUI mode, use numeric prompt instead
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_tui: bool,

    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    Suggest {
        #[arg(long)]
        input: String,
        #[arg(long, default_value_t = false)]
        interactive: bool,
    },
    Run {
        #[arg(required = true, trailing_var_arg = true)]
        args: Vec<String>,
        #[arg(long, default_value_t = true)]
        interactive: bool,
    },

    Hooks {
        #[command(subcommand)]
        action: HookAction,
    },

    Smriti {
        #[command(subcommand)]
        action: SmritiAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum HookAction {
    /// Print hook snippets (no file writes)
    Print,
    /// Install hooks into shell rc files (~/.bashrc, ~/.zshrc) safely
    Install {
        /// Install only for bash
        #[arg(long, default_value_t = false)]
        bash: bool,
        /// Install only for zsh
        #[arg(long, default_value_t = false)]
        zsh: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum SmritiAction {
    /// Import from bash/zsh history into Sutra history (bootstrap)
    Import {
        /// Import bash history (~/.bash_history)
        #[arg(long, default_value_t = false)]
        bash: bool,
        /// Import zsh history (~/.zsh_history)
        #[arg(long, default_value_t = false)]
        zsh: bool,
        /// Max number of lines to import from each file (default: 20000)
        #[arg(long, default_value_t = 20_000)]
        limit: usize,
        /// If set, don't write; just report what would be imported
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Generate defaults catalog for common commands
    GenerateDefaults {
        /// Output file path (default: ~/.sutra/defaults.json)
        #[arg(long)]
        output: Option<std::path::PathBuf>,
        /// Overwrite existing defaults file
        #[arg(long, default_value_t = false)]
        overwrite: bool,
    },
}
