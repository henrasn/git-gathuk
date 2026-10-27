use anyhow::Result;
use git_gathuk::{app::AppState, events, git, ui};
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, panic, time::Duration};
fn main() -> Result<()> {
    // Handle CLI arguments like --help, --version, or specific file target
    if let Some(arg) = std::env::args().nth(1) {
        if arg == "-v" || arg == "--version" {
            println!("git-gathuk v0.1.0");
            return Ok(());
        }
        if arg == "-h" || arg == "--help" {
            println!("git-gathuk (gathuk) - Keyboard-driven Git companion & 3-pane merge conflict resolver");
            println!("\nUsage:");
            println!("  git-gathuk / gathuk / git gathuk   Open conflict resolver in current git repository");
            println!("  git-gathuk <file>                  Open directly on a specific conflicted file");
            println!("\nKeybindings:");
            println!("  n / p                     Select next / previous conflict hunk");
            println!("  j / k                     Scroll line-by-line (or navigate files if sidebar is open)");
            println!("  h / l or ← / →            Pan horizontally (left / right) across all panes");
            println!("  Ctrl+l                    Pick Local (HEAD)");
            println!("  Ctrl+r                    Pick Remote (Incoming)");
            println!("  Ctrl+b                    Pick Both (concatenated)");
            println!("  u                         Undo / Revert current hunk");
            println!("  Ctrl+f                    Toggle repository conflict files sidebar");
            println!("  Enter                     (In sidebar) open file and focus into 3-pane diff");
            println!("  :w or Ctrl+s              Save resolved file and stage with git add");
            println!("  q or Ctrl+c               Quit");
            return Ok(());
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set panic hook to ensure terminal is restored on unexpected crash
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Initialize State from Git
    let repo_root = git::get_repo_root().unwrap_or_else(|_| ".".to_string());
    let current_branch = git::get_active_branch().unwrap_or_else(|_| "HEAD".to_string());
    let mut conflicted_files = git::get_conflicted_files().unwrap_or_default();

    // If a specific file was passed as argument (e.g., from `git mergetool`), prioritize it
    if let Some(target_file) = std::env::args().nth(1) {
        if !target_file.starts_with('-') {
            if let Some(pos) = conflicted_files.iter().position(|f| f == &target_file || f.ends_with(&target_file)) {
                let f = conflicted_files.remove(pos);
                conflicted_files.insert(0, f);
            } else if std::path::Path::new(&target_file).exists() {
                conflicted_files.insert(0, target_file);
            }
        }
    }

    let mut app = AppState::new(repo_root, current_branch, conflicted_files);

    if app.conflicted_files.is_empty() {
        app.set_toast("No active git merge conflicts detected".to_string(), false);
    }

    // Main App Loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:#}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut AppState) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press
                    && matches!(events::handle_key_event(app, key), events::AppAction::Quit)
                {
                    break;
                }
            }
        }
    }

    Ok(())
}
