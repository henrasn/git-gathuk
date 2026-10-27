use crate::app::AppState;
use crate::parser::{self, Chunk, Resolution};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

// Catppuccin Macchiato Palette Constants
#[allow(dead_code)]
pub const CRUST: Color = Color::Rgb(24, 25, 38);
#[allow(dead_code)]
pub const MANTLE: Color = Color::Rgb(30, 32, 48);
#[allow(dead_code)]
pub const BASE: Color = Color::Rgb(36, 39, 58);
#[allow(dead_code)]
pub const SURFACE0: Color = Color::Rgb(54, 58, 79);
#[allow(dead_code)]
pub const SURFACE1: Color = Color::Rgb(73, 77, 100);
#[allow(dead_code)]
pub const SURFACE2: Color = Color::Rgb(91, 96, 120);
#[allow(dead_code)]
pub const OVERLAY0: Color = Color::Rgb(110, 115, 141);
#[allow(dead_code)]
pub const SUBTEXT0: Color = Color::Rgb(165, 173, 203);
#[allow(dead_code)]
pub const TEXT: Color = Color::Rgb(202, 211, 245);
#[allow(dead_code)]
pub const BLUE: Color = Color::Rgb(138, 173, 244);
#[allow(dead_code)]
pub const GREEN: Color = Color::Rgb(166, 218, 149);
#[allow(dead_code)]
pub const PEACH: Color = Color::Rgb(245, 169, 127);
#[allow(dead_code)]
pub const YELLOW: Color = Color::Rgb(238, 212, 159);
#[allow(dead_code)]
pub const RED: Color = Color::Rgb(237, 135, 150);
#[allow(dead_code)]
pub const MAUVE: Color = Color::Rgb(198, 160, 246);
#[allow(dead_code)]
pub const SAPPHIRE: Color = Color::Rgb(125, 196, 228);
#[allow(dead_code)]
pub const TEAL: Color = Color::Rgb(139, 213, 202);

// Active block background tints
pub const LOCAL_BG_TINT: Color = Color::Rgb(35, 45, 68);
pub const REMOTE_BG_TINT: Color = Color::Rgb(35, 55, 48);
pub const CENTER_BG_TINT: Color = Color::Rgb(55, 45, 42);

pub fn draw(f: &mut Frame, app: &AppState) {
    let size = f.area();

    let root_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top Header
            Constraint::Min(5),    // Main Diff Area
            Constraint::Length(1), // Bottom Statusbar
        ])
        .split(size);

    draw_header(f, app, root_layout[0]);
    draw_workspace(f, app, root_layout[1]);
    draw_footer(f, app, root_layout[2]);
}

fn draw_header(f: &mut Frame, app: &AppState, area: Rect) {
    let repo_display = if app.repo_root.is_empty() {
        "~/workspace".to_string()
    } else {
        app.repo_root.clone()
    };

    let left_spans = vec![
        Span::styled(format!(" {} ", repo_display), Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled("[merge: conflict]", Style::default().fg(RED).add_modifier(Modifier::BOLD)),
    ];

    let right_spans = vec![
        Span::styled("git:", Style::default().fg(OVERLAY0)),
        Span::styled(
            format!("({}* -> main)", app.current_branch),
            Style::default().fg(MAUVE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(SURFACE1)),
        Span::styled("GIT-GATHUK v0.1.0 ", Style::default().fg(SUBTEXT0)),
    ];
    let right_len = right_spans.iter().map(|s| s.content.len()).sum::<usize>() as u16;

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(right_len.min(area.width)),
        ])
        .split(area);

    let left_header = Paragraph::new(Line::from(left_spans)).style(Style::default().bg(CRUST));
    f.render_widget(left_header, layout[0]);

    let right_header = Paragraph::new(Line::from(right_spans))
        .alignment(Alignment::Right)
        .style(Style::default().bg(CRUST));
    f.render_widget(right_header, layout[1]);
}

fn draw_workspace(f: &mut Frame, app: &AppState, area: Rect) {
    let sidebar_width = 30;
    let main_layout = if app.show_sidebar {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(sidebar_width), Constraint::Min(20)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(0), Constraint::Min(20)])
            .split(area)
    };

    if app.show_sidebar {
        draw_sidebar(f, app, main_layout[0]);
    }

    draw_diff_panes(f, app, main_layout[1]);
}

fn draw_sidebar(f: &mut Frame, app: &AppState, area: Rect) {
    let border_style = if app.show_sidebar {
        Style::default().fg(BLUE).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(SURFACE1)
    };

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(border_style)
        .style(Style::default().bg(CRUST));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split inner into Header row (height 1) and Items list
    let sidebar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    let files_str = format!("{} files ", app.conflicted_files.len());
    let files_len = files_str.len() as u16;

    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(files_len)])
        .split(sidebar_layout[0]);

    let full_title = " CONFLICT REPOSITORY";
    let short_title = " CONFLICTS";
    let title_str = if header_layout[0].width >= full_title.len() as u16 {
        full_title
    } else {
        short_title
    };

    let header_left = Paragraph::new(Line::from(vec![
        Span::styled(title_str, Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
    ])).style(Style::default().bg(MANTLE));
    f.render_widget(header_left, header_layout[0]);

    let header_right = Paragraph::new(Line::from(vec![
        Span::styled(files_str, Style::default().fg(SUBTEXT0)),
    ]))
    .alignment(Alignment::Right)
    .style(Style::default().bg(MANTLE));
    f.render_widget(header_right, header_layout[1]);

    // Items list (bg: CRUST)
    let inner_width = sidebar_layout[1].width as usize;
    let mut lines = Vec::new();
    for (idx, file) in app.conflicted_files.iter().enumerate() {
        let is_selected = idx == app.selected_file_idx;
        let prefix = if is_selected { "▶ " } else { "  " };

        let (resolved, total) = if is_selected {
            parser::count_hunks(&app.chunks)
        } else {
            (0, 1)
        };

        let file_text = format!("{}{}", prefix, file);
        let badge_text = format!("[{}/{} hunks] ", resolved, total);

        let pad_len = inner_width.saturating_sub(file_text.len() + badge_text.len());
        let pad_str = " ".repeat(pad_len);

        let file_span = if is_selected {
            Span::styled(file_text, Style::default().fg(BLUE).bg(SURFACE0).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(file_text, Style::default().fg(SUBTEXT0))
        };

        let pad_span = if is_selected {
            Span::styled(pad_str, Style::default().bg(SURFACE0))
        } else {
            Span::raw(pad_str)
        };

        let badge_span = Span::styled(
            badge_text,
            if is_selected {
                Style::default().fg(PEACH).bg(SURFACE0)
            } else {
                Style::default().fg(OVERLAY0)
            },
        );

        let line = Line::from(vec![file_span, pad_span, badge_span]);
        lines.push(line);
    }

    let p = Paragraph::new(lines).style(Style::default().bg(CRUST));
    f.render_widget(p, sidebar_layout[1]);
}

fn draw_diff_panes(f: &mut Frame, app: &AppState, area: Rect) {
    let pane_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(area);

    let conflict_indices = app.conflict_chunk_indices();
    let targeted_chunk_idx = conflict_indices.get(app.active_conflict_idx).copied();
    let scroll_y = app.scroll_offset;

    draw_local_pane(f, app, pane_cols[0], targeted_chunk_idx, scroll_y);
    draw_merged_pane(f, app, pane_cols[1], targeted_chunk_idx, scroll_y);
    draw_remote_pane(f, app, pane_cols[2], targeted_chunk_idx, scroll_y);
}


fn draw_local_pane(
    f: &mut Frame,
    app: &AppState,
    area: Rect,
    targeted_chunk_idx: Option<usize>,
    scroll_y: u16,
) {
    // Single right border dividing Local from Merged
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(SURFACE1))
        .style(Style::default().bg(CRUST));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split into Header row (height 1) and Code area (remaining)
    let pane_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    let total_lines = app.chunks.iter().map(|c| match c {
        Chunk::Unchanged(l) => l.len(),
        Chunk::Conflict { local, .. } => local.len(),
    }).sum::<usize>();

    let range_str = format!("main:1..{} ", total_lines);
    let range_len = range_str.len() as u16;

    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(range_len)])
        .split(pane_split[0]);

    let full_title = " [1] LOCAL (HEAD)";
    let short_title = " [1] LOCAL";
    let title_str = if header_layout[0].width >= full_title.len() as u16 {
        full_title
    } else {
        short_title
    };

    let header_left = Paragraph::new(Line::from(vec![
        Span::styled(title_str, Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
    ])).style(Style::default().bg(MANTLE));
    f.render_widget(header_left, header_layout[0]);

    let header_right = Paragraph::new(Line::from(vec![
        Span::styled(range_str, Style::default().fg(OVERLAY0)),
    ]))
    .alignment(Alignment::Right)
    .style(Style::default().bg(MANTLE));
    f.render_widget(header_right, header_layout[1]);

    // Split pane_split[1] into fixed Gutter (width 7) and Code area (remainder)
    let code_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(pane_split[1]);

    let mut gutter_lines = Vec::new();
    let mut code_lines = Vec::new();
    let mut line_num = 1;

    for (chunk_idx, chunk) in app.chunks.iter().enumerate() {
        let is_targeted = Some(chunk_idx) == targeted_chunk_idx;

        match chunk {
            Chunk::Unchanged(lines) => {
                for line in lines {
                    gutter_lines.push(Line::from(vec![
                        Span::styled(format!("{:4} │ ", line_num), Style::default().fg(SURFACE2)),
                    ]));
                    code_lines.push(Line::from(vec![
                        Span::styled(line.clone(), Style::default().fg(TEXT)),
                    ]));
                    line_num += 1;
                }
            }
            Chunk::Conflict { local, remote, state } => {
                let border_color = BLUE;
                let bg = if is_targeted { LOCAL_BG_TINT } else { CRUST };

                // Row 0: Top marker (aligned with merged pane)
                gutter_lines.push(Line::from(vec![
                    Span::styled("       ", Style::default().bg(bg)),
                ]));
                code_lines.push(Line::from(vec![
                    Span::styled("<<<<<<< [1] LOCAL (HEAD)", Style::default().fg(border_color).bg(bg).add_modifier(Modifier::BOLD)),
                ]));

                // Local lines (aligned 1:1 with local lines in merged pane)
                for line in local {
                    gutter_lines.push(Line::from(vec![
                        Span::styled(
                            format!("{:4}   ", line_num),
                            Style::default().fg(BLUE).bg(bg).add_modifier(if is_targeted { Modifier::BOLD } else { Modifier::empty() }),
                        ),
                    ]));
                    code_lines.push(Line::from(vec![
                        Span::styled(line.clone(), Style::default().fg(SAPPHIRE).bg(bg)),
                    ]));
                    line_num += 1;
                }

                // Mid divider row (aligned with merged mid marker)
                gutter_lines.push(Line::from(vec![
                    Span::styled("   ~   ", Style::default().fg(SURFACE1).bg(bg)),
                ]));
                code_lines.push(Line::from(vec![
                    Span::styled("~", Style::default().fg(SURFACE1).bg(bg)),
                ]));

                // Filler lines for remote lines (aligned 1:1 with remote lines in merged pane)
                for _ in 0..remote.len() {
                    gutter_lines.push(Line::from(vec![
                        Span::styled("   ~   ", Style::default().fg(SURFACE1).bg(bg)),
                    ]));
                    code_lines.push(Line::from(vec![
                        Span::styled("~", Style::default().fg(SURFACE1).bg(bg)),
                    ]));
                }

                // Bottom row (aligned with merged bottom marker)
                gutter_lines.push(Line::from(vec![
                    Span::styled("   ~   ", Style::default().fg(SURFACE1).bg(bg)),
                ]));
                code_lines.push(Line::from(vec![
                    Span::styled("~", Style::default().fg(SURFACE1).bg(bg)),
                ]));

                // Targeted help spacer row if active conflict is unresolved
                if is_targeted && *state == Resolution::Unresolved {
                    gutter_lines.push(Line::from(vec![
                        Span::styled("       ", Style::default().fg(SURFACE1).bg(bg)),
                    ]));
                    code_lines.push(Line::from(vec![
                        Span::styled("", Style::default().bg(bg)),
                    ]));
                }
            }
        }
    }

    let gutter_p = Paragraph::new(gutter_lines)
        .style(Style::default().bg(CRUST))
        .scroll((scroll_y, 0));
    f.render_widget(gutter_p, code_split[0]);

    let code_p = Paragraph::new(code_lines)
        .style(Style::default().bg(CRUST))
        .scroll((scroll_y, app.scroll_x));
    f.render_widget(code_p, code_split[1]);
}

fn draw_merged_pane(
    f: &mut Frame,
    app: &AppState,
    area: Rect,
    targeted_chunk_idx: Option<usize>,
    scroll_y: u16,
) {
    let is_focused = !app.show_sidebar;
    let border_style = if is_focused {
        Style::default().fg(PEACH).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(SURFACE1)
    };

    // Single right border dividing Merged from Remote
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(border_style)
        .style(Style::default().bg(CRUST));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split into Header row (height 1) and Code area (remaining)
    let pane_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    let total_hunks = app.total_conflicts();
    let active_hunk_num = if total_hunks > 0 { app.active_conflict_idx + 1 } else { 0 };

    let (badge_text, badge_color) = if let Some(target) = targeted_chunk_idx {
        if let Some(Chunk::Conflict { state, .. }) = app.chunks.get(target) {
            match state {
                Resolution::Unresolved => ("UNRESOLVED ", PEACH),
                Resolution::Local => ("RESOLVED [LOCAL] ", BLUE),
                Resolution::Remote => ("RESOLVED [REMOTE] ", GREEN),
                Resolution::Both => ("RESOLVED [BOTH] ", PEACH),
            }
        } else {
            ("", PEACH)
        }
    } else {
        ("", PEACH)
    };

    let badge_len = badge_text.len() as u16;
    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(badge_len)])
        .split(pane_split[0]);

    let full_title = format!(" [*] MERGED BUFFER [Hunk {}/{} Active]", active_hunk_num, total_hunks);
    let med_title = format!(" [*] MERGED [{}/{}]", active_hunk_num, total_hunks);
    let short_title = format!(" [*] [{}/{}]", active_hunk_num, total_hunks);

    let title_str = if header_layout[0].width >= full_title.len() as u16 {
        full_title
    } else if header_layout[0].width >= med_title.len() as u16 {
        med_title
    } else {
        short_title
    };

    let header_left = Paragraph::new(Line::from(vec![
        Span::styled(title_str, Style::default().fg(PEACH).add_modifier(Modifier::BOLD)),
    ])).style(Style::default().bg(SURFACE0));
    f.render_widget(header_left, header_layout[0]);

    let header_right = Paragraph::new(Line::from(vec![
        Span::styled(badge_text, Style::default().fg(badge_color).add_modifier(Modifier::BOLD)),
    ]))
    .alignment(Alignment::Right)
    .style(Style::default().bg(SURFACE0));
    f.render_widget(header_right, header_layout[1]);

    // Split pane_split[1] into fixed Gutter (width 7) and Code area (remainder)
    let code_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(pane_split[1]);

    let mut gutter_lines = Vec::new();
    let mut code_lines = Vec::new();
    let mut line_num = 1;
    let mut hunk_counter = 0;

    for (chunk_idx, chunk) in app.chunks.iter().enumerate() {
        let is_targeted = Some(chunk_idx) == targeted_chunk_idx;

        match chunk {
            Chunk::Unchanged(lines) => {
                for line in lines {
                    gutter_lines.push(Line::from(vec![
                        Span::styled(format!("{:4} │ ", line_num), Style::default().fg(SURFACE2)),
                    ]));
                    code_lines.push(Line::from(vec![
                        Span::styled(line.clone(), Style::default().fg(TEXT)),
                    ]));
                    line_num += 1;
                }
            }
            Chunk::Conflict { local, remote, state } => {
                hunk_counter += 1;
                let bg = if is_targeted { CENTER_BG_TINT } else { CRUST };

                match state {
                    Resolution::Unresolved => {
                        // Top marker
                        gutter_lines.push(Line::from(vec![
                            Span::styled("       ", Style::default().bg(bg)),
                        ]));
                        code_lines.push(Line::from(vec![
                            Span::styled("<<<<<<< [1] LOCAL (HEAD)", Style::default().fg(BLUE).bg(bg).add_modifier(Modifier::BOLD)),
                        ]));

                        // Local lines
                        for line in local {
                            gutter_lines.push(Line::from(vec![
                                Span::styled("  ??   ", Style::default().fg(PEACH).bg(bg)),
                            ]));
                            code_lines.push(Line::from(vec![
                                Span::styled(line.clone(), Style::default().fg(SAPPHIRE).bg(bg)),
                            ]));
                        }

                        // Mid divider
                        gutter_lines.push(Line::from(vec![
                            Span::styled("       ", Style::default().bg(bg)),
                        ]));
                        code_lines.push(Line::from(vec![
                            Span::styled(
                                format!("======= CONFLICT (Hunk {} of {})", hunk_counter, total_hunks),
                                Style::default().fg(PEACH).bg(bg).add_modifier(Modifier::BOLD),
                            ),
                        ]));

                        // Remote lines
                        for line in remote {
                            gutter_lines.push(Line::from(vec![
                                Span::styled("  ??   ", Style::default().fg(PEACH).bg(bg)),
                            ]));
                            code_lines.push(Line::from(vec![
                                Span::styled(line.clone(), Style::default().fg(GREEN).bg(bg)),
                            ]));
                        }

                        // Bottom marker
                        gutter_lines.push(Line::from(vec![
                            Span::styled("       ", Style::default().bg(bg)),
                        ]));
                        code_lines.push(Line::from(vec![
                            Span::styled(">>>>>>> [2] REMOTE (Incoming)", Style::default().fg(GREEN).bg(bg).add_modifier(Modifier::BOLD)),
                        ]));

                        // Targeted help hint
                        if is_targeted {
                            gutter_lines.push(Line::from(vec![
                                Span::styled("       ", Style::default().fg(PEACH).bg(bg)),
                            ]));
                            code_lines.push(Line::from(vec![
                                Span::styled(
                                    "[Ctrl+l: pick local │ Ctrl+r: pick remote │ Ctrl+b: both]",
                                    Style::default().fg(OVERLAY0).bg(bg),
                                ),
                            ]));
                        }
                    }
                    Resolution::Local => {
                        // Top marker
                        gutter_lines.push(Line::from(vec![
                            Span::styled("       ", Style::default().bg(bg)),
                        ]));
                        code_lines.push(Line::from(vec![
                            Span::styled("✓ RESOLVED: [1] LOCAL (HEAD)", Style::default().fg(BLUE).bg(bg).add_modifier(Modifier::BOLD)),
                        ]));

                        // Local lines
                        for line in local {
                            gutter_lines.push(Line::from(vec![
                                Span::styled(format!("{:4}   ", line_num), Style::default().fg(BLUE).bg(bg)),
                            ]));
                            code_lines.push(Line::from(vec![
                                Span::styled(line.clone(), Style::default().fg(SAPPHIRE).bg(bg)),
                            ]));
                            line_num += 1;
                        }

                        // Mid divider row
                        gutter_lines.push(Line::from(vec![
                            Span::styled("   ~   ", Style::default().fg(SURFACE1).bg(bg)),
                        ]));
                        code_lines.push(Line::from(vec![
                            Span::styled("~", Style::default().fg(SURFACE1).bg(bg)),
                        ]));

                        // Filler for remote
                        for _ in 0..remote.len() {
                            gutter_lines.push(Line::from(vec![
                                Span::styled("   ~   ", Style::default().fg(SURFACE1).bg(bg)),
                            ]));
                            code_lines.push(Line::from(vec![
                                Span::styled("~", Style::default().fg(SURFACE1).bg(bg)),
                            ]));
                        }

                        // Bottom row
                        gutter_lines.push(Line::from(vec![
                            Span::styled("   ~   ", Style::default().fg(SURFACE1).bg(bg)),
                        ]));
                        code_lines.push(Line::from(vec![
                            Span::styled("~", Style::default().fg(SURFACE1).bg(bg)),
                        ]));
                    }
                    Resolution::Remote => {
                        // Top row
                        gutter_lines.push(Line::from(vec![
                            Span::styled("   ~   ", Style::default().fg(SURFACE1).bg(bg)),
                        ]));
                        code_lines.push(Line::from(vec![
                            Span::styled("~", Style::default().fg(SURFACE1).bg(bg)),
                        ]));

                        // Filler for local
                        for _ in 0..local.len() {
                            gutter_lines.push(Line::from(vec![
                                Span::styled("   ~   ", Style::default().fg(SURFACE1).bg(bg)),
                            ]));
                            code_lines.push(Line::from(vec![
                                Span::styled("~", Style::default().fg(SURFACE1).bg(bg)),
                            ]));
                        }

                        // Mid divider row
                        gutter_lines.push(Line::from(vec![
                            Span::styled("   ~   ", Style::default().fg(SURFACE1).bg(bg)),
                        ]));
                        code_lines.push(Line::from(vec![
                            Span::styled("~", Style::default().fg(SURFACE1).bg(bg)),
                        ]));

                        // Remote lines
                        for line in remote {
                            gutter_lines.push(Line::from(vec![
                                Span::styled(format!("{:4}   ", line_num), Style::default().fg(GREEN).bg(bg)),
                            ]));
                            code_lines.push(Line::from(vec![
                                Span::styled(line.clone(), Style::default().fg(TEAL).bg(bg)),
                            ]));
                            line_num += 1;
                        }

                        // Bottom marker
                        gutter_lines.push(Line::from(vec![
                            Span::styled("       ", Style::default().bg(bg)),
                        ]));
                        code_lines.push(Line::from(vec![
                            Span::styled("✓ RESOLVED: [2] REMOTE (Incoming)", Style::default().fg(GREEN).bg(bg).add_modifier(Modifier::BOLD)),
                        ]));
                    }
                    Resolution::Both => {
                        // Top marker
                        gutter_lines.push(Line::from(vec![
                            Span::styled("       ", Style::default().bg(bg)),
                        ]));
                        code_lines.push(Line::from(vec![
                            Span::styled("✓ RESOLVED: [1] LOCAL (HEAD)", Style::default().fg(BLUE).bg(bg).add_modifier(Modifier::BOLD)),
                        ]));

                        // Local lines
                        for line in local {
                            gutter_lines.push(Line::from(vec![
                                Span::styled(format!("{:4}   ", line_num), Style::default().fg(BLUE).bg(bg)),
                            ]));
                            code_lines.push(Line::from(vec![
                                Span::styled(line.clone(), Style::default().fg(SAPPHIRE).bg(bg)),
                            ]));
                            line_num += 1;
                        }

                        // Mid marker
                        gutter_lines.push(Line::from(vec![
                            Span::styled("       ", Style::default().bg(bg)),
                        ]));
                        code_lines.push(Line::from(vec![
                            Span::styled("✓ RESOLVED: BOTH (UNIFIED)", Style::default().fg(PEACH).bg(bg).add_modifier(Modifier::BOLD)),
                        ]));

                        // Remote lines
                        for line in remote {
                            gutter_lines.push(Line::from(vec![
                                Span::styled(format!("{:4}   ", line_num), Style::default().fg(GREEN).bg(bg)),
                            ]));
                            code_lines.push(Line::from(vec![
                                Span::styled(line.clone(), Style::default().fg(TEAL).bg(bg)),
                            ]));
                            line_num += 1;
                        }

                        // Bottom marker
                        gutter_lines.push(Line::from(vec![
                            Span::styled("       ", Style::default().bg(bg)),
                        ]));
                        code_lines.push(Line::from(vec![
                            Span::styled("✓ RESOLVED: [2] REMOTE (Incoming)", Style::default().fg(GREEN).bg(bg).add_modifier(Modifier::BOLD)),
                        ]));
                    }
                }
            }
        }
    }

    let gutter_p = Paragraph::new(gutter_lines)
        .style(Style::default().bg(CRUST))
        .scroll((scroll_y, 0));
    f.render_widget(gutter_p, code_split[0]);

    let code_p = Paragraph::new(code_lines)
        .style(Style::default().bg(CRUST))
        .scroll((scroll_y, app.scroll_x));
    f.render_widget(code_p, code_split[1]);
}

fn draw_remote_pane(
    f: &mut Frame,
    app: &AppState,
    area: Rect,
    targeted_chunk_idx: Option<usize>,
    scroll_y: u16,
) {
    // Pane 3 is right-most: Borders::NONE
    let pane_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let total_lines = app.chunks.iter().map(|c| match c {
        Chunk::Unchanged(l) => l.len(),
        Chunk::Conflict { remote, .. } => remote.len(),
    }).sum::<usize>();

    let range_str = format!("incoming:1..{} ", total_lines);
    let range_len = range_str.len() as u16;

    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(range_len)])
        .split(pane_split[0]);

    let full_title = " [2] REMOTE (Incoming)";
    let short_title = " [2] REMOTE";
    let title_str = if header_layout[0].width >= full_title.len() as u16 {
        full_title
    } else {
        short_title
    };

    let header_left = Paragraph::new(Line::from(vec![
        Span::styled(title_str, Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
    ])).style(Style::default().bg(MANTLE));
    f.render_widget(header_left, header_layout[0]);

    let header_right = Paragraph::new(Line::from(vec![
        Span::styled(range_str, Style::default().fg(OVERLAY0)),
    ]))
    .alignment(Alignment::Right)
    .style(Style::default().bg(MANTLE));
    f.render_widget(header_right, header_layout[1]);

    // Split pane_split[1] into fixed Gutter (width 7) and Code area (remainder)
    let code_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(pane_split[1]);

    let mut gutter_lines = Vec::new();
    let mut code_lines = Vec::new();
    let mut line_num = 1;

    for (chunk_idx, chunk) in app.chunks.iter().enumerate() {
        let is_targeted = Some(chunk_idx) == targeted_chunk_idx;

        match chunk {
            Chunk::Unchanged(lines) => {
                for line in lines {
                    gutter_lines.push(Line::from(vec![
                        Span::styled(format!("{:4} │ ", line_num), Style::default().fg(SURFACE2)),
                    ]));
                    code_lines.push(Line::from(vec![
                        Span::styled(line.clone(), Style::default().fg(TEXT)),
                    ]));
                    line_num += 1;
                }
            }
            Chunk::Conflict { local, remote, state } => {
                let border_color = GREEN;
                let bg = if is_targeted { REMOTE_BG_TINT } else { CRUST };

                // Row 0: Top row (aligned with merged Row 0)
                gutter_lines.push(Line::from(vec![
                    Span::styled("   ~   ", Style::default().fg(SURFACE1).bg(bg)),
                ]));
                code_lines.push(Line::from(vec![
                    Span::styled("~", Style::default().fg(SURFACE1).bg(bg)),
                ]));

                // Filler lines for local lines (aligned 1:1 with local lines in merged pane)
                for _ in 0..local.len() {
                    gutter_lines.push(Line::from(vec![
                        Span::styled("   ~   ", Style::default().fg(SURFACE1).bg(bg)),
                    ]));
                    code_lines.push(Line::from(vec![
                        Span::styled("~", Style::default().fg(SURFACE1).bg(bg)),
                    ]));
                }

                // Mid divider row (aligned with merged mid marker)
                gutter_lines.push(Line::from(vec![
                    Span::styled("   ~   ", Style::default().fg(SURFACE1).bg(bg)),
                ]));
                code_lines.push(Line::from(vec![
                    Span::styled("~", Style::default().fg(SURFACE1).bg(bg)),
                ]));

                // Remote lines (aligned 1:1 with remote lines in merged pane)
                for line in remote {
                    gutter_lines.push(Line::from(vec![
                        Span::styled(
                            format!("{:4}   ", line_num),
                            Style::default().fg(GREEN).bg(bg).add_modifier(if is_targeted { Modifier::BOLD } else { Modifier::empty() }),
                        ),
                    ]));
                    code_lines.push(Line::from(vec![
                        Span::styled(line.clone(), Style::default().fg(TEAL).bg(bg)),
                    ]));
                    line_num += 1;
                }

                // Bottom row: >>>>>>> [2] REMOTE (Incoming)
                gutter_lines.push(Line::from(vec![
                    Span::styled("       ", Style::default().bg(bg)),
                ]));
                code_lines.push(Line::from(vec![
                    Span::styled(">>>>>>> [2] REMOTE (Incoming)", Style::default().fg(border_color).bg(bg).add_modifier(Modifier::BOLD)),
                ]));

                // Targeted help spacer row if active conflict is unresolved
                if is_targeted && *state == Resolution::Unresolved {
                    gutter_lines.push(Line::from(vec![
                        Span::styled("       ", Style::default().fg(SURFACE1).bg(bg)),
                    ]));
                    code_lines.push(Line::from(vec![
                        Span::styled("", Style::default().bg(bg)),
                    ]));
                }
            }
        }
    }

    let gutter_p = Paragraph::new(gutter_lines)
        .style(Style::default().bg(CRUST))
        .scroll((scroll_y, 0));
    f.render_widget(gutter_p, code_split[0]);

    let code_p = Paragraph::new(code_lines)
        .style(Style::default().bg(CRUST))
        .scroll((scroll_y, app.scroll_x));
    f.render_widget(code_p, code_split[1]);
}

fn draw_footer(f: &mut Frame, app: &AppState, area: Rect) {
    let right_width = if let Some((msg, _)) = &app.status_toast {
        let desired = msg.len() as u16 + 2;
        desired.min(area.width.saturating_sub(64))
    } else {
        32
    };
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(right_width)])
        .split(area);

    let left_spans = if app.show_sidebar {
        vec![
            Span::styled(" NORMAL [SIDEBAR] ", Style::default().bg(BLUE).fg(CRUST).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(SURFACE1)),
            Span::styled("j/k:", Style::default().fg(OVERLAY0)),
            Span::styled("Files ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("Enter/l:", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
            Span::styled("Focus 3-Pane ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("Ctrl+f/h:", Style::default().fg(OVERLAY0)),
            Span::styled("Hide Sidebar ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("q:", Style::default().fg(OVERLAY0)),
            Span::styled("Quit", Style::default().fg(TEXT)),
        ]
    } else if layout[0].width >= 120 {
        vec![
            Span::styled(" NORMAL [3-PANE] ", Style::default().bg(PEACH).fg(CRUST).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(SURFACE1)),
            Span::styled("n/p:", Style::default().fg(OVERLAY0)),
            Span::styled("Hunk ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("j/k:", Style::default().fg(OVERLAY0)),
            Span::styled("Scroll ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("h/l:", Style::default().fg(OVERLAY0)),
            Span::styled("Pan ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("Ctrl+l:", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
            Span::styled("Local ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("Ctrl+r:", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("Remote ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("Ctrl+b:", Style::default().fg(PEACH).add_modifier(Modifier::BOLD)),
            Span::styled("Both ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("u:", Style::default().fg(OVERLAY0)),
            Span::styled("Undo ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("Ctrl+f:", Style::default().fg(OVERLAY0)),
            Span::styled("Files ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled(":w:", Style::default().fg(OVERLAY0)),
            Span::styled("Save ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("q:", Style::default().fg(OVERLAY0)),
            Span::styled("Quit", Style::default().fg(TEXT)),
        ]
    } else if layout[0].width >= 85 {
        vec![
            Span::styled(" NORMAL ", Style::default().bg(PEACH).fg(CRUST).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("n/p:", Style::default().fg(OVERLAY0)),
            Span::styled("Hunk ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("h/l:", Style::default().fg(OVERLAY0)),
            Span::styled("Pan ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("^l:", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
            Span::styled("Loc ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("^r:", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("Rem ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("^b:", Style::default().fg(PEACH).add_modifier(Modifier::BOLD)),
            Span::styled("Both ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("u:", Style::default().fg(OVERLAY0)),
            Span::styled("Undo ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("^f:", Style::default().fg(OVERLAY0)),
            Span::styled("Files ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled(":w:", Style::default().fg(OVERLAY0)),
            Span::styled("Save ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("q:", Style::default().fg(OVERLAY0)),
            Span::styled("Quit", Style::default().fg(TEXT)),
        ]
    } else {
        vec![
            Span::styled(" NORMAL ", Style::default().bg(PEACH).fg(CRUST).add_modifier(Modifier::BOLD)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("^l:", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
            Span::styled("Loc ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("^r:", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("Rem ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("^b:", Style::default().fg(PEACH).add_modifier(Modifier::BOLD)),
            Span::styled("Both ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("u:", Style::default().fg(OVERLAY0)),
            Span::styled("Undo ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled(":w:", Style::default().fg(OVERLAY0)),
            Span::styled("Save ", Style::default().fg(TEXT)),
            Span::styled("│ ", Style::default().fg(SURFACE1)),
            Span::styled("q:", Style::default().fg(OVERLAY0)),
            Span::styled("Quit", Style::default().fg(TEXT)),
        ]
    };
    let left_p = Paragraph::new(Line::from(left_spans)).style(Style::default().bg(MANTLE));
    f.render_widget(left_p, layout[0]);

    // Right status toast or cursor info
    let right_spans = if let Some((msg, is_success)) = &app.status_toast {
        vec![
            Span::styled(
                format!(" {} ", msg),
                Style::default()
                    .fg(if *is_success { GREEN } else { BLUE })
                    .bg(SURFACE0)
                    .add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        let mut spans = Vec::new();
        if app.scroll_x > 0 {
            spans.push(Span::styled(
                format!("[Col +{}] ", app.scroll_x),
                Style::default().fg(YELLOW).bg(SURFACE0).add_modifier(Modifier::BOLD),
            ));
        }
        spans.push(Span::styled("rustc 1.79.0 ", Style::default().fg(OVERLAY0)));
        spans.push(Span::styled(
            format!("Hunk {}/{} ", app.active_conflict_idx + 1, app.total_conflicts()),
            Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
        ));
        spans
    };

    let right_p = Paragraph::new(Line::from(right_spans))
        .alignment(Alignment::Right)
        .style(Style::default().bg(MANTLE));
    f.render_widget(right_p, layout[1]);
}
