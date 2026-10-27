use ratatui::{backend::TestBackend, Terminal};
use git_gathuk::app::AppState;
use git_gathuk::parser::{Chunk, Resolution};
use git_gathuk::ui;

fn create_mock_app() -> AppState {
    let chunks = vec![
        Chunk::Unchanged(vec![
            "pub struct UserSession {".into(),
            "    token: String,".into(),
            "    user_id: Uuid,".into(),
        ]),
        Chunk::Conflict {
            local: vec![
                "    pub fn is_valid(&self) -> bool {".into(),
                "        true".into(),
                "    }".into(),
            ],
            remote: vec![
                "    pub async fn is_valid(&self) -> Result<bool> {".into(),
                "        Ok(true)".into(),
                "    }".into(),
            ],
            state: Resolution::Unresolved,
        },
        Chunk::Unchanged(vec![
            "    role: RoleType,".into(),
            "}".into(),
        ]),
    ];

    AppState {
        repo_root: "~/dev/git-resolve-tui".into(),
        current_branch: "feature/payment-v2".into(),
        show_sidebar: false,
        conflicted_files: vec![
            "session.rs".into(),
            "routes.rs".into(),
            "Cargo.toml".into(),
        ],
        selected_file_idx: 0,
        active_file_path: Some("session.rs".into()),
        chunks,
        pure_local_text: vec![
            "pub struct UserSession {".into(),
            "    token: String,".into(),
            "    user_id: Uuid,".into(),
            "    pub fn is_valid(&self) -> bool {".into(),
            "        true".into(),
            "    }".into(),
            "    role: RoleType,".into(),
            "}".into(),
        ],
        pure_remote_text: vec![
            "pub struct UserSession {".into(),
            "    token: String,".into(),
            "    user_id: Uuid,".into(),
            "    pub async fn is_valid(&self) -> Result<bool> {".into(),
            "        Ok(true)".into(),
            "    }".into(),
            "    role: RoleType,".into(),
            "}".into(),
        ],
        active_conflict_idx: 0,
        scroll_offset: 0,
        scroll_x: 0,
        status_toast: None,
        should_quit: false,
    }
}

#[test]
fn test_ui_3pane_diff_snapshot() {
    let backend = TestBackend::new(110, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    let app = create_mock_app();
    terminal.draw(|f| ui::draw(f, &app)).unwrap();

    insta::assert_snapshot!(terminal.backend());
}

#[test]
fn test_ui_sidebar_visible_snapshot() {
    let backend = TestBackend::new(120, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = create_mock_app();
    app.show_sidebar = true;
    terminal.draw(|f| ui::draw(f, &app)).unwrap();

    insta::assert_snapshot!(terminal.backend());
}

#[test]
fn test_ui_resolved_local_snapshot() {
    let backend = TestBackend::new(110, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = create_mock_app();
    app.resolve_active(Resolution::Local);
    terminal.draw(|f| ui::draw(f, &app)).unwrap();

    insta::assert_snapshot!(terminal.backend());
}

#[test]
fn test_ui_horizontal_scroll_snapshot() {
    let backend = TestBackend::new(110, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = create_mock_app();
    app.scroll_x = 12;
    terminal.draw(|f| ui::draw(f, &app)).unwrap();

    insta::assert_snapshot!(terminal.backend());
}

