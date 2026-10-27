# git-gathuk (gathuk)

> **A keyboard-driven Git companion and 3-pane merge conflict resolver built with Rust, Ratatui, and Catppuccin Macchiato.**

> *“Gathuk”* *(Javanese)* — That satisfying moment when two separate pieces match up, align, and click together seamlessly into place.

`git-gathuk` transforms messy Git merge conflicts into a calm, intuitive, keyboard-first terminal workflow. Operating directly on your Git working tree with zero external C library dependencies, it gives you full control over resolving complex multi-file, multi-hunk conflicts.

---

## ✨ Features

- **True 3-Pane Diffing**:
  - **Left Pane**: Pristine Local (`HEAD` / Current branch).
  - **Center Pane**: Interactive Merged Working Buffer with live resolution preview.
  - **Right Pane**: Pristine Remote (`Incoming` / `MERGE_HEAD`).
- **Synchronized Navigation**:
  - **Vertical Sync**: Scroll line-by-line simultaneously across all three panes (`j` / `k`).
  - **Horizontal Panning**: Pan wide lines horizontally (`h` / `l` or `←` / `→`) while **fixed line-number gutters stay pinned in place**.
  - **Hunk Jumping**: Jump between conflict hunks instantly (`n` / `p`) with auto-centering.
- **Repository Conflict Sidebar (`Ctrl+f`)**:
  - Toggle a sidebar listing all conflicted files in the repository.
  - Seamlessly switch files with `Enter` without leaving the terminal.
- **In-Memory AST Conflict Resolution**:
  - Pick Local (`Ctrl+l`), Remote (`Ctrl+r`), or Both (`Ctrl+b`).
  - Instant Undo (`u`) to revert a hunk back to its raw conflict state.
  - Atomically save to disk and stage to Git index (`:w` or `Ctrl+s`).
- **Clean Aesthetic**:
  - Catppuccin Macchiato color palette.
  - Clean vertical pane separators and flush conflict markers with zero visual noise.
- **Zero-Bloat Architecture**:
  - Pure Rust + standard Git CLI subprocesses (no heavy `libgit2` linking).
  - Instant startup and tiny binary footprint.

---

## ⌨️ Keybindings

### Navigation
| Key | Action |
| :--- | :--- |
| `n` | Jump to **next** conflict hunk (auto-centers all panes) |
| `p` | Jump to **previous** conflict hunk (auto-centers all panes) |
| `j` / `Down` | Scroll down line-by-line (or navigate files if sidebar is open) |
| `k` / `Up` | Scroll up line-by-line (or navigate files if sidebar is open) |
| `h` / `Left` | Pan horizontally left across code (gutters stay pinned) |
| `l` / `Right` | Pan horizontally right across code (gutters stay pinned) |

### Resolution & Actions
| Key | Action |
| :--- | :--- |
| `Ctrl+l` | Resolve current hunk with **Local (HEAD)** |
| `Ctrl+r` | Resolve current hunk with **Remote (Incoming)** |
| `Ctrl+b` | Resolve current hunk with **Both** (concatenated) |
| `u` | **Undo** resolution (revert current hunk to unresolved) |
| `:w` or `Ctrl+s` | **Save** resolved file to disk and stage (`git add`) |
| `Ctrl+f` | **Toggle Conflict Files Sidebar** |
| `Enter` | *(In sidebar)* Open selected file in 3-pane workspace |
| `q` or `Ctrl+c` | **Quit** application |

---

## 🚀 Installation (Linux / macOS)

### Prerequisites
Ensure you have Rust and Cargo installed (via [rustup](https://rustup.rs)):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Option A: Using Makefile (Recommended)
```bash
# Clone the repository
git clone https://github.com/your-username/git-gathuk.git
cd git-gathuk

# Build optimized release and install to ~/.local/bin
make install
```
*This installs `git-gathuk` to `~/.local/bin/git-gathuk` and automatically creates a symlink `gathuk`.*

*(Ensure `~/.local/bin` is in your `$PATH`)*

### Option B: Using Cargo Directly
```bash
cargo install --path .
```

### Option C: System-Wide Install
```bash
make install-system
# or: sudo install -Dm755 target/release/git-gathuk /usr/local/bin/git-gathuk
```

---

## 🛠️ Usage

### 1. 3 Ways to Run
Because `git-gathuk` adheres to the Git subcommand convention, you can invoke it in any of the following ways:

```bash
# 1. Shortest standalone command
gathuk

# 2. Native Git subcommand
git gathuk

# 3. Direct binary name
git-gathuk

# Open directly on a specific file:
gathuk src/session_manager.rs
```

### 2. Configure as Global Git Mergetool
Configure Git to automatically launch `git-gathuk` whenever you run `git mergetool`:

```bash
git config --global merge.tool git-gathuk
git config --global mergetool.git-gathuk.cmd 'git-gathuk "$MERGED"'
git config --global mergetool.git-gathuk.trustExitCode true
git config --global mergetool.prompt false
```

Now, during any merge conflict:
```bash
git mergetool
```

### 3. Neovim Integration
Add this to your Neovim keymaps (e.g. using `snacks.nvim` or `toggleterm.nvim`):

```lua
-- snacks.nvim example
local function toggle_gathuk()
  if os.execute("git rev-parse --is-inside-work-tree > /dev/null 2>&1") ~= 0 then
    vim.notify("Not in a git repository", vim.log.levels.WARN)
    return
  end
  Snacks.terminal.toggle("git-gathuk", {
    win = { position = "float", border = "rounded", width = 0.95, height = 0.95 },
    interactive = true,
    on_close = function()
      vim.cmd("checktime")
      vim.notify("Merge session closed. Buffers reloaded.", vim.log.levels.INFO)
    end,
  })
end

vim.keymap.set("n", "<leader>gm", toggle_gathuk, { desc = "Toggle Git Gathuk" })
```

---

## 🧪 Simulation & Automated Tests

### Complex Conflict Simulator
A simulation generator script is provided to generate complex multi-file, multi-hunk conflicts with long horizontal lines (>120 columns):

```bash
# Generate simulation at /tmp/git_gathuk_simulation
./simulate_complex_conflict.sh

# Open git-gathuk in the simulated repo
cd /tmp/git_gathuk_simulation
gathuk
```

### Running Tests
`git-gathuk` includes unit tests, integration tests against real Git working trees, and visual UI snapshot tests powered by `insta`:

```bash
# Run all tests
cargo test

# Review or update UI snapshot tests
cargo test --test ui_snapshot_test
# or with cargo-insta:
cargo insta review
```

---

## 🗺️ Roadmap: Beyond Merge Conflicts

`git-gathuk` is designed as a foundational Git companion. Future capabilities planned include:

- [ ] **Interactive Rebase Companion**: Visual reordering, squashing, and editing of commit history.
- [ ] **Hunk-Level Staging**: Interactive staging and patch splitting (`git add -p` enhanced).
- [ ] **Syntax Highlighting**: Language-aware tree-sitter or syntect token coloring.
- [ ] **Commit Graph & Branch Visualizer**: Fast, aesthetic overview of repository topology.

---

## 📄 License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
