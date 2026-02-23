use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

use crate::candidate::{ScoredCandidate, SourceKind};

fn src_tag(s: SourceKind) -> &'static str {
    match s {
        SourceKind::Smriti => "Smriti",
        SourceKind::Path => "PATH",
        SourceKind::Defaults => "defaults",
        SourceKind::Plugin => "plugin",
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Returns:
/// - Ok(Some(index)) on selection
/// - Ok(None) on cancel
pub fn pick_candidate_tui(
    input: &str,
    items: &[ScoredCandidate],
) -> Result<Option<usize>> {
    if items.is_empty() {
        return Ok(None);
    }

    let mut terminal = setup_terminal()?;
    let mut state = ListState::default();
    state.select(Some(0));

    let res = run_loop(&mut terminal, input, items, &mut state);

    // Always restore terminal
    let _ = restore_terminal(terminal);
    res
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    input: &str,
    items: &[ScoredCandidate],
    state: &mut ListState,
) -> Result<Option<usize>> {
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(2),
                ])
                .split(size);

            let header = Paragraph::new(Line::from(vec![
                Span::styled("Sutra", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("  —  "),
                Span::raw("I couldn't run: "),
                Span::styled(format!("\"{}\"", input), Style::default().add_modifier(Modifier::BOLD)),
            ]))
            .block(Block::default().borders(Borders::ALL).title("Suggestion"));

            f.render_widget(header, chunks[0]);

            let list_items: Vec<ListItem> = items
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    // 1) cmd [src | score]
                    // show numeric shortcut (1..)
                    let num = i + 1;
                    let line = Line::from(vec![
                        Span::styled(format!("{num}) "), Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(&s.cand.canonical),
                        Span::raw("  "),
                        Span::styled(
                            format!("[{} | {:.3}]", src_tag(s.cand.source), s.score),
                            Style::default().add_modifier(Modifier::DIM),
                        ),
                    ]);
                    ListItem::new(line)
                })
                .collect();

            let list = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title("Pick one (↑/↓, Enter, 1-9, q/Esc)"))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol("▶ ");

            f.render_stateful_widget(list, chunks[1], state);

            let footer = Paragraph::new(Line::from(vec![
                Span::raw("Enter: run   "),
                Span::raw("1-9: quick pick   "),
                Span::raw("q/Esc: cancel   "),
                Span::raw("Ctrl+C: exit"),
            ]));
            f.render_widget(footer, chunks[2]);
        })?;

        // input handling
        if event::poll(Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => {
                    // Ctrl+C
                    if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
                        return Ok(None);
                    }

                    match k.code {
                        KeyCode::Esc => return Ok(None),
                        KeyCode::Char('q') => return Ok(None),

                        KeyCode::Up => {
                            let i = state.selected().unwrap_or(0);
                            let next = if i == 0 { items.len() - 1 } else { i - 1 };
                            state.select(Some(next));
                        }
                        KeyCode::Down => {
                            let i = state.selected().unwrap_or(0);
                            let next = if i + 1 >= items.len() { 0 } else { i + 1 };
                            state.select(Some(next));
                        }
                        KeyCode::Enter => {
                            return Ok(state.selected());
                        }

                        // number shortcuts 1..9
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            let n = (c as u8 - b'0') as usize;
                            if n >= 1 && n <= items.len().min(9) {
                                return Ok(Some(n - 1));
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}