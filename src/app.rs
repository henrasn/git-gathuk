use crate::git;
use crate::parser::{self, Chunk, Resolution};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FocusMode {
    Sidebar,
    ThreePane,
}

pub struct AppState {
    pub repo_root: String,
    pub current_branch: String,

    // Sidebar & Focus
    pub show_sidebar: bool,
    pub conflicted_files: Vec<String>,
    pub selected_file_idx: usize,

    // Active File State
    pub active_file_path: Option<String>,
    pub chunks: Vec<Chunk>,            // The parsed AST of the active file
    pub pure_local_text: Vec<String>,  // Fetched via `git show :2:<file>`
    pub pure_remote_text: Vec<String>, // Fetched via `git show :3:<file>`
    pub active_conflict_idx: usize,    // Which Conflict chunk is currently highlighted (n/p navigation)
    pub scroll_offset: u16,            // Vertical scroll offset for 3-pane diff view
    pub scroll_x: u16,                 // Horizontal scroll offset for long lines

    // UI feedback & lifecycle
    pub status_toast: Option<(String, bool)>, // (message, is_success)
    pub should_quit: bool,
}

impl AppState {
    pub fn new(repo_root: String, current_branch: String, conflicted_files: Vec<String>) -> Self {
        let mut app = Self {
            repo_root,
            current_branch,
            show_sidebar: false,
            conflicted_files,
            selected_file_idx: 0,
            active_file_path: None,
            chunks: Vec::new(),
            pure_local_text: Vec::new(),
            pure_remote_text: Vec::new(),
            active_conflict_idx: 0,
            scroll_offset: 0,
            scroll_x: 0,
            status_toast: None,
            should_quit: false,
        };

        if !app.conflicted_files.is_empty() {
            let _ = app.load_file_at(0);
        }

        app
    }

    /// Focus is on Sidebar when sidebar is shown, otherwise on 3-Pane
    #[allow(dead_code)]
    pub fn focus(&self) -> FocusMode {
        if self.show_sidebar {
            FocusMode::Sidebar
        } else {
            FocusMode::ThreePane
        }
    }

    /// Helper to find indices of all `Chunk::Conflict` in `self.chunks`
    pub fn conflict_chunk_indices(&self) -> Vec<usize> {
        self.chunks
            .iter()
            .enumerate()
            .filter_map(|(i, c)| match c {
                Chunk::Conflict { .. } => Some(i),
                _ => None,
            })
            .collect()
    }

    pub fn total_conflicts(&self) -> usize {
        self.conflict_chunk_indices().len()
    }

    /// Load the file at the specified index in `conflicted_files`
    pub fn load_file_at(&mut self, idx: usize) -> Result<()> {
        if idx >= self.conflicted_files.len() {
            anyhow::bail!("File index out of bounds: {}", idx);
        }

        let rel_path = &self.conflicted_files[idx];
        let full_path = if Path::new(rel_path).is_absolute() {
            rel_path.clone()
        } else {
            format!("{}/{}", self.repo_root, rel_path)
        };

        let content = fs::read_to_string(&full_path)
            .with_context(|| format!("Failed to read file: {}", full_path))?;

        self.chunks = parser::parse_conflicted_content(&content);
        self.selected_file_idx = idx;
        self.active_file_path = Some(rel_path.clone());
        self.active_conflict_idx = 0;
        self.scroll_offset = 0;
        self.scroll_x = 0;

        // Fetch pristine Local (:2:) and Remote (:3:) content
        let local_raw = git::get_local_content(rel_path).unwrap_or_default();
        self.pure_local_text = local_raw.lines().map(|s| s.to_string()).collect();

        let remote_raw = git::get_remote_content(rel_path).unwrap_or_default();
        self.pure_remote_text = remote_raw.lines().map(|s| s.to_string()).collect();

        self.center_scroll_on_active_conflict();

        Ok(())
    }

    /// Ctrl+f toggles sidebar: when shown focus is in sidebar, when hidden focus is on 3-pane
    pub fn toggle_sidebar(&mut self) {
        self.show_sidebar = !self.show_sidebar;
        if self.show_sidebar {
            self.set_toast("[KEY] 'Ctrl+f' -> Sidebar Visible (Focus: Sidebar)".to_string(), false);
        } else {
            self.set_toast("[KEY] 'Ctrl+f' -> Sidebar Hidden (Focus: 3-Pane)".to_string(), false);
        }
    }

    /// In sidebar: navigate down to next file
    pub fn select_next_file(&mut self) {
        if !self.conflicted_files.is_empty() {
            let next = (self.selected_file_idx + 1) % self.conflicted_files.len();
            let _ = self.load_file_at(next);
        }
    }

    /// In sidebar: navigate up to previous file
    pub fn select_prev_file(&mut self) {
        if !self.conflicted_files.is_empty() {
            let prev = if self.selected_file_idx == 0 {
                self.conflicted_files.len() - 1
            } else {
                self.selected_file_idx - 1
            };
            let _ = self.load_file_at(prev);
        }
    }

    /// In 3-pane: scroll down by line
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// In 3-pane: scroll up by line
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// In 3-pane: scroll right horizontally across long lines
    pub fn scroll_right(&mut self, cols: u16) {
        self.scroll_x = self.scroll_x.saturating_add(cols);
    }

    /// In 3-pane: scroll left horizontally
    pub fn scroll_left(&mut self, cols: u16) {
        self.scroll_x = self.scroll_x.saturating_sub(cols);
    }

    /// Calculate the exact number of visual rows a chunk occupies across all 3 panes
    pub fn chunk_visual_height(chunk: &Chunk, is_targeted: bool) -> usize {
        match chunk {
            Chunk::Unchanged(lines) => lines.len(),
            Chunk::Conflict { local, remote, state } => {
                let hint_row = if is_targeted && *state == Resolution::Unresolved { 1 } else { 0 };
                3 + local.len() + remote.len() + hint_row
            }
        }
    }

    /// Center scroll offset around the currently active conflict hunk
    pub fn center_scroll_on_active_conflict(&mut self) {
        let indices = self.conflict_chunk_indices();
        if let Some(&target) = indices.get(self.active_conflict_idx) {
            let mut line_count = 0;
            for (i, chunk) in self.chunks.iter().enumerate() {
                if i == target {
                    break;
                }
                line_count += Self::chunk_visual_height(chunk, false);
            }
            self.scroll_offset = line_count.saturating_sub(4) as u16;
            self.scroll_x = 0;
        }
    }

    /// n: select next conflict hunk
    pub fn next_conflict(&mut self) {
        let total = self.total_conflicts();
        if total > 0 && self.active_conflict_idx + 1 < total {
            self.active_conflict_idx += 1;
            self.center_scroll_on_active_conflict();
            self.set_toast(
                format!(
                    "[KEY] 'n' -> Selected Hunk {}/{}",
                    self.active_conflict_idx + 1,
                    total
                ),
                false,
            );
        }
    }

    /// p: select previous conflict hunk
    pub fn prev_conflict(&mut self) {
        let total = self.total_conflicts();
        if total > 0 && self.active_conflict_idx > 0 {
            self.active_conflict_idx -= 1;
            self.center_scroll_on_active_conflict();
            self.set_toast(
                format!(
                    "[KEY] 'p' -> Selected Hunk {}/{}",
                    self.active_conflict_idx + 1,
                    total
                ),
                false,
            );
        }
    }

    /// Mutate the targeted conflict hunk's resolution state
    pub fn resolve_active(&mut self, resolution: Resolution) {
        let indices = self.conflict_chunk_indices();
        if let Some(&chunk_idx) = indices.get(self.active_conflict_idx) {
            let hunk_num = self.active_conflict_idx + 1;
            if let Chunk::Conflict { ref mut state, .. } = self.chunks[chunk_idx] {
                *state = resolution.clone();
            }

            match resolution {
                Resolution::Local => {
                    self.set_toast(
                        format!("[KEY] 'Ctrl+l' -> Applied Local to Hunk {}", hunk_num),
                        true,
                    );
                }
                Resolution::Remote => {
                    self.set_toast(
                        format!("[KEY] 'Ctrl+r' -> Applied Remote to Hunk {}", hunk_num),
                        true,
                    );
                }
                Resolution::Both => {
                    self.set_toast(
                        format!("[KEY] 'Ctrl+b' -> Applied Both to Hunk {}", hunk_num),
                        true,
                    );
                }
                Resolution::Unresolved => {
                    self.set_toast(
                        format!("[KEY] 'u' -> Reverted Hunk {}", hunk_num),
                        false,
                    );
                }
            }
        }
    }

    /// Writes serialized AST to file and stages to Git index
    pub fn save_and_stage(&mut self) -> Result<()> {
        let rel_path = match &self.active_file_path {
            Some(p) => p.clone(),
            None => {
                self.set_toast("No active file to save".to_string(), false);
                return Ok(());
            }
        };

        let full_path = if Path::new(&rel_path).is_absolute() {
            rel_path.clone()
        } else {
            format!("{}/{}", self.repo_root, rel_path)
        };

        let serialized = parser::serialize_chunks(&self.chunks);
        fs::write(&full_path, serialized)
            .with_context(|| format!("Failed to write to file: {}", full_path))?;

        git::stage_file(&rel_path)
            .with_context(|| format!("Failed to stage file: {}", rel_path))?;

        self.set_toast(
            format!("[KEY] ':w' -> Staged {} changes to Git index", rel_path),
            true,
        );

        // Refresh conflicted files list
        if let Ok(updated) = git::get_conflicted_files() {
            self.conflicted_files = updated;
            if self.selected_file_idx >= self.conflicted_files.len() {
                self.selected_file_idx = self.conflicted_files.len().saturating_sub(1);
            }
        }

        Ok(())
    }

    pub fn set_toast(&mut self, msg: String, is_success: bool) {
        self.status_message_internal(msg, is_success);
    }

    fn status_message_internal(&mut self, msg: String, is_success: bool) {
        self.status_toast = Some((msg, is_success));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_mutations() {
        let chunks = vec![
            Chunk::Unchanged(vec!["fn test() {".into()]),
            Chunk::Conflict {
                local: vec!["let a = 1;".into()],
                remote: vec!["let a = 2;".into()],
                state: Resolution::Unresolved,
            },
            Chunk::Conflict {
                local: vec!["let b = 1;".into()],
                remote: vec!["let b = 2;".into()],
                state: Resolution::Unresolved,
            },
        ];

        let mut app = AppState {
            repo_root: "/test".into(),
            current_branch: "feature".into(),
            show_sidebar: false,
            conflicted_files: vec!["test.rs".into()],
            selected_file_idx: 0,
            active_file_path: Some("test.rs".into()),
            chunks,
            pure_local_text: Vec::new(),
            pure_remote_text: Vec::new(),
            active_conflict_idx: 0,
            scroll_offset: 0,
            scroll_x: 0,
            status_toast: None,
            should_quit: false,
        };

        assert_eq!(app.total_conflicts(), 2);
        assert_eq!(app.active_conflict_idx, 0);
        assert_eq!(app.focus(), FocusMode::ThreePane);

        // n / p hunk navigation
        app.next_conflict();
        assert_eq!(app.active_conflict_idx, 1);
        app.next_conflict(); // Should stay at 1
        assert_eq!(app.active_conflict_idx, 1);
        app.prev_conflict();
        assert_eq!(app.active_conflict_idx, 0);

        // Resolve Local
        app.resolve_active(Resolution::Local);
        let indices = app.conflict_chunk_indices();
        if let Chunk::Conflict { state, .. } = &app.chunks[indices[0]] {
            assert_eq!(*state, Resolution::Local);
        } else {
            panic!("Expected conflict chunk");
        }

        // Toggle sidebar & focus behavior
        assert!(!app.show_sidebar);
        assert_eq!(app.focus(), FocusMode::ThreePane);

        app.toggle_sidebar();
        assert!(app.show_sidebar);
        assert_eq!(app.focus(), FocusMode::Sidebar);

        app.toggle_sidebar();
        assert!(!app.show_sidebar);
        assert_eq!(app.focus(), FocusMode::ThreePane);

        // Line scroll navigation
        assert_eq!(app.scroll_offset, 0);
        app.scroll_down();
        assert_eq!(app.scroll_offset, 1);
        app.scroll_up();
        assert_eq!(app.scroll_offset, 0);
    }
}
