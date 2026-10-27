use crate::app::AppState;
use crate::parser::Resolution;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum AppAction {
    None,
    Quit,
}

pub fn handle_key_event(app: &mut AppState, key: KeyEvent) -> AppAction {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    if ctrl {
        match key.code {
            KeyCode::Char('c') => return AppAction::Quit,
            KeyCode::Char('f') => {
                app.toggle_sidebar();
            }
            KeyCode::Char('l') => {
                app.resolve_active(Resolution::Local);
            }
            KeyCode::Char('r') => {
                app.resolve_active(Resolution::Remote);
            }
            KeyCode::Char('b') => {
                app.resolve_active(Resolution::Both);
            }
            KeyCode::Char('s') => {
                if let Err(e) = app.save_and_stage() {
                    app.set_toast(format!("Save error: {}", e), false);
                }
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Char('q') => return AppAction::Quit,
            // n / p select next / previous conflict hunk
            KeyCode::Char('n') => {
                app.next_conflict();
            }
            KeyCode::Char('p') => {
                app.prev_conflict();
            }
            // j / k navigate down / up inside file pane or sidebar
            KeyCode::Char('j') | KeyCode::Down => {
                if app.show_sidebar {
                    app.select_next_file();
                } else {
                    app.scroll_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.show_sidebar {
                    app.select_prev_file();
                } else {
                    app.scroll_up();
                }
            }
            // h / l or Left / Right navigate horizontally across long code lines
            KeyCode::Char('h') | KeyCode::Left => {
                if app.show_sidebar {
                    app.show_sidebar = false;
                } else {
                    app.scroll_left(6);
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if app.show_sidebar {
                    app.show_sidebar = false;
                    app.set_toast("[KEY] 'l' -> Focused 3-Pane Workspace".to_string(), false);
                } else {
                    app.scroll_right(6);
                }
            }
            // Enter in sidebar: confirm file and return focus to 3-pane
            KeyCode::Enter => {
                if app.show_sidebar {
                    app.show_sidebar = false;
                    app.set_toast("[KEY] 'Enter' -> Focused 3-Pane Workspace".to_string(), false);
                }
            }
            KeyCode::Char('u') => {
                app.resolve_active(Resolution::Unresolved);
            }
            KeyCode::Char('w') | KeyCode::Char(':') => {
                if let Err(e) = app.save_and_stage() {
                    app.set_toast(format!("Save error: {}", e), false);
                }
            }
            _ => {}
        }
    }

    AppAction::None
}
