#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    Unresolved,
    Local,
    Remote,
    Both, // Local first, then Remote concatenated
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chunk {
    Unchanged(Vec<String>), // Normal code lines
    Conflict {
        local: Vec<String>,     // Lines between <<<<<<< and =======
        remote: Vec<String>,    // Lines between ======= and >>>>>>>
        state: Resolution,      // Current resolution choice
    },
}

enum ParseState {
    Normal,
    InLocal,
    InAncestor, // diff3 support: lines between ||||||| and =======
    InRemote,
}

/// Parses the text of a file that may contain Git merge conflict markers.
pub fn parse_conflicted_content(content: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut current_unchanged: Vec<String> = Vec::new();
    let mut current_local: Vec<String> = Vec::new();
    let mut current_remote: Vec<String> = Vec::new();

    let mut state = ParseState::Normal;

    for line in content.lines() {
        match state {
            ParseState::Normal => {
                if line.starts_with("<<<<<<<") {
                    if !current_unchanged.is_empty() {
                        chunks.push(Chunk::Unchanged(std::mem::take(&mut current_unchanged)));
                    }
                    current_local.clear();
                    current_remote.clear();
                    state = ParseState::InLocal;
                } else {
                    current_unchanged.push(line.to_string());
                }
            }
            ParseState::InLocal => {
                if line.starts_with("|||||||") {
                    // Diff3 ancestor marker - ignore ancestor section
                    state = ParseState::InAncestor;
                } else if line.starts_with("=======") {
                    state = ParseState::InRemote;
                } else {
                    current_local.push(line.to_string());
                }
            }
            ParseState::InAncestor => {
                if line.starts_with("=======") {
                    state = ParseState::InRemote;
                }
                // Skip ancestor lines
            }
            ParseState::InRemote => {
                if line.starts_with(">>>>>>>") {
                    chunks.push(Chunk::Conflict {
                        local: std::mem::take(&mut current_local),
                        remote: std::mem::take(&mut current_remote),
                        state: Resolution::Unresolved,
                    });
                    state = ParseState::Normal;
                } else {
                    current_remote.push(line.to_string());
                }
            }
        }
    }

    // Flush any trailing content
    if !current_unchanged.is_empty() {
        chunks.push(Chunk::Unchanged(current_unchanged));
    } else if matches!(state, ParseState::InLocal | ParseState::InAncestor | ParseState::InRemote) {
        // Unterminated conflict marker, preserve what we parsed as unchanged lines
        let mut recovered = current_local;
        recovered.extend(current_remote);
        if !recovered.is_empty() {
            chunks.push(Chunk::Unchanged(recovered));
        }
    }

    chunks
}

/// Serializes chunks back into a file string according to current resolutions.
pub fn serialize_chunks(chunks: &[Chunk]) -> String {
    let mut output = Vec::new();

    for chunk in chunks {
        match chunk {
            Chunk::Unchanged(lines) => {
                output.extend(lines.clone());
            }
            Chunk::Conflict { local, remote, state } => match state {
                Resolution::Unresolved => {
                    output.push("<<<<<<< LOCAL (HEAD)".to_string());
                    output.extend(local.clone());
                    output.push("=======".to_string());
                    output.extend(remote.clone());
                    output.push(">>>>>>> REMOTE (Incoming)".to_string());
                }
                Resolution::Local => {
                    output.extend(local.clone());
                }
                Resolution::Remote => {
                    output.extend(remote.clone());
                }
                Resolution::Both => {
                    output.extend(local.clone());
                    output.extend(remote.clone());
                }
            },
        }
    }

    if output.is_empty() {
        String::new()
    } else {
        let mut s = output.join("\n");
        s.push('\n');
        s
    }
}

/// Returns (resolved_hunks, total_hunks)
pub fn count_hunks(chunks: &[Chunk]) -> (usize, usize) {
    let mut resolved = 0;
    let mut total = 0;

    for chunk in chunks {
        if let Chunk::Conflict { state, .. } = chunk {
            total += 1;
            if *state != Resolution::Unresolved {
                resolved += 1;
            }
        }
    }

    (resolved, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_conflicted_content_standard() {
        let content = r#"fn main() {
<<<<<<< HEAD
    println!("Hello from local");
=======
    println!("Hello from remote");
>>>>>>> branch
}
"#;
        let chunks = parse_conflicted_content(content);
        assert_eq!(chunks.len(), 3);

        match &chunks[0] {
            Chunk::Unchanged(lines) => assert_eq!(lines, &["fn main() {"]),
            _ => panic!("Expected Unchanged"),
        }

        match &chunks[1] {
            Chunk::Conflict { local, remote, state } => {
                assert_eq!(local, &["    println!(\"Hello from local\");"]);
                assert_eq!(remote, &["    println!(\"Hello from remote\");"]);
                assert_eq!(*state, Resolution::Unresolved);
            }
            _ => panic!("Expected Conflict"),
        }

        match &chunks[2] {
            Chunk::Unchanged(lines) => assert_eq!(lines, &["}"]),
            _ => panic!("Expected Unchanged"),
        }
    }

    #[test]
    fn test_parse_diff3_style() {
        let content = r#"start
<<<<<<< HEAD
local line
||||||| ancestor
ancestor line
=======
remote line
>>>>>>> branch
end"#;
        let chunks = parse_conflicted_content(content);
        assert_eq!(chunks.len(), 3);
        if let Chunk::Conflict { local, remote, .. } = &chunks[1] {
            assert_eq!(local, &["local line"]);
            assert_eq!(remote, &["remote line"]);
        } else {
            panic!("Expected Conflict chunk");
        }
    }

    #[test]
    fn test_serialize_resolutions() {
        let mut chunks = vec![
            Chunk::Unchanged(vec!["let x = 1;".into()]),
            Chunk::Conflict {
                local: vec!["let y = 2;".into()],
                remote: vec!["let y = 3;".into()],
                state: Resolution::Unresolved,
            },
        ];

        // Unresolved
        let unres = serialize_chunks(&chunks);
        assert!(unres.contains("<<<<<<< LOCAL (HEAD)"));
        assert!(unres.contains(">>>>>>> REMOTE (Incoming)"));

        // Pick Local
        if let Chunk::Conflict { ref mut state, .. } = chunks[1] {
            *state = Resolution::Local;
        }
        let loc = serialize_chunks(&chunks);
        assert_eq!(loc, "let x = 1;\nlet y = 2;\n");

        // Pick Remote
        if let Chunk::Conflict { ref mut state, .. } = chunks[1] {
            *state = Resolution::Remote;
        }
        let rem = serialize_chunks(&chunks);
        assert_eq!(rem, "let x = 1;\nlet y = 3;\n");

        // Pick Both
        if let Chunk::Conflict { ref mut state, .. } = chunks[1] {
            *state = Resolution::Both;
        }
        let both = serialize_chunks(&chunks);
        assert_eq!(both, "let x = 1;\nlet y = 2;\nlet y = 3;\n");
    }

    #[test]
    fn test_count_hunks() {
        let chunks = vec![
            Chunk::Unchanged(vec!["A".into()]),
            Chunk::Conflict {
                local: vec!["B1".into()],
                remote: vec!["B2".into()],
                state: Resolution::Unresolved,
            },
            Chunk::Conflict {
                local: vec!["C1".into()],
                remote: vec!["C2".into()],
                state: Resolution::Local,
            },
        ];
        assert_eq!(count_hunks(&chunks), (1, 2));
    }
}
