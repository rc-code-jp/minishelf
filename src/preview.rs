use std::path::{Path, PathBuf};

use git2::{DiffFlags, DiffFormat, DiffOptions, Repository};

const MAX_PREVIEW_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, Copy)]
pub enum PreviewKind {
    Text,
    Message,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewRenderMode {
    Markdown,
    Raw,
    Diff,
}

impl PreviewRenderMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Raw => "raw",
            Self::Diff => "diff",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreviewState {
    pub kind: PreviewKind,
    pub render_mode: PreviewRenderMode,
    pub lines: Vec<String>,
    pub scroll: usize,
    available_modes: Vec<PreviewRenderMode>,
}

impl PreviewState {
    pub fn from_path(
        startup_root: &Path,
        path: &Path,
        preferred_mode: Option<PreviewRenderMode>,
    ) -> Self {
        if path.is_dir() {
            return Self::message("directory");
        }

        let raw_lines = match load_raw_file(path) {
            Ok(lines) => lines,
            Err(msg) => return Self::message(msg),
        };

        let is_markdown = is_markdown_file(path);
        let diff_lines = collect_diff_lines(startup_root, path);

        let mut available_modes = Vec::with_capacity(3);
        if is_markdown {
            available_modes.push(PreviewRenderMode::Markdown);
        }
        available_modes.push(PreviewRenderMode::Raw);
        if diff_lines.is_some() {
            available_modes.push(PreviewRenderMode::Diff);
        }

        let default_mode = if is_markdown {
            PreviewRenderMode::Markdown
        } else if diff_lines.is_some() {
            PreviewRenderMode::Diff
        } else {
            PreviewRenderMode::Raw
        };

        let render_mode = preferred_mode
            .filter(|mode| available_modes.contains(mode))
            .unwrap_or(default_mode);

        let lines = match render_mode {
            PreviewRenderMode::Diff => diff_lines.unwrap_or_default(),
            PreviewRenderMode::Raw | PreviewRenderMode::Markdown => raw_lines,
        };

        Self {
            kind: PreviewKind::Text,
            render_mode,
            lines,
            scroll: 0,
            available_modes,
        }
    }

    pub fn message(msg: impl Into<String>) -> Self {
        Self {
            kind: PreviewKind::Message,
            render_mode: PreviewRenderMode::Raw,
            lines: vec![msg.into()],
            scroll: 0,
            available_modes: vec![PreviewRenderMode::Raw],
        }
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_add(amount);
    }

    pub fn next_render_mode(&self) -> Option<PreviewRenderMode> {
        if self.available_modes.len() <= 1 {
            return None;
        }

        let index = self
            .available_modes
            .iter()
            .position(|mode| *mode == self.render_mode)?;
        let next_index = (index + 1) % self.available_modes.len();
        Some(self.available_modes[next_index])
    }

    pub fn mode_label(&self) -> &'static str {
        self.render_mode.label()
    }
}

fn load_raw_file(path: &Path) -> Result<Vec<String>, String> {
    let metadata = std::fs::metadata(path).map_err(|err| format!("preview read failed: {err}"))?;

    if metadata.len() > MAX_PREVIEW_BYTES {
        return Err(format!("file too large (> {} bytes)", MAX_PREVIEW_BYTES));
    }

    let bytes = std::fs::read(path).map_err(|err| format!("preview read failed: {err}"))?;

    if bytes.contains(&0) {
        return Err(String::from("Binary or non-UTF-8 text is not previewable"));
    }

    let text = std::str::from_utf8(&bytes)
        .map_err(|_| String::from("Binary or non-UTF-8 text is not previewable"))?;

    Ok(text.lines().map(std::string::ToString::to_string).collect())
}

fn collect_diff_lines(startup_root: &Path, path: &Path) -> Option<Vec<String>> {
    let repo = Repository::discover(startup_root).ok()?;
    let workdir = repo.workdir()?;
    let relative_path = relative_to_workdir(workdir, path)?;

    let mut options = DiffOptions::new();
    options
        .pathspec(relative_path)
        .include_untracked(true)
        .recurse_untracked_dirs(true);

    let head_tree = repo.head().ok().and_then(|head| head.peel_to_tree().ok());

    let diff = repo
        .diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut options))
        .ok()?;

    for delta in diff.deltas() {
        if delta.flags().contains(DiffFlags::BINARY) {
            return None;
        }
    }

    let mut non_utf8 = false;
    let mut lines = Vec::new();
    let print_result = diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let content = line.content();
        let text = match std::str::from_utf8(content) {
            Ok(t) => t.trim_end_matches('\n').trim_end_matches('\r').to_string(),
            Err(_) => {
                non_utf8 = true;
                return false;
            }
        };

        if let Some(renderable) = to_renderable_diff_line(line.origin(), &text) {
            lines.push(renderable);
        }
        true
    });

    if print_result.is_err() || non_utf8 || lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}

fn relative_to_workdir(workdir: &Path, path: &Path) -> Option<PathBuf> {
    if let Ok(rel) = path.strip_prefix(workdir) {
        return Some(rel.to_path_buf());
    }

    let canonical_workdir = workdir.canonicalize().ok()?;

    if let Ok(canonical_path) = path.canonicalize() {
        if let Ok(rel) = canonical_path.strip_prefix(&canonical_workdir) {
            return Some(rel.to_path_buf());
        }
    }

    // Deleted/renamed targets may not exist anymore; resolve parent and rebuild.
    let canonical_parent = path.parent()?.canonicalize().ok()?;
    let file_name = path.file_name()?;
    let rel_parent = canonical_parent.strip_prefix(&canonical_workdir).ok()?;
    Some(rel_parent.join(file_name))
}

fn is_markdown_file(path: &Path) -> bool {
    let Some(ext) = path.extension() else {
        return false;
    };

    matches!(
        ext.to_string_lossy().to_ascii_lowercase().as_str(),
        "md" | "markdown"
    )
}

fn to_renderable_diff_line(origin: char, line: &str) -> Option<String> {
    match origin {
        '+' => Some(format!("+{line}")),
        '-' => Some(format!("-{line}")),
        ' ' => Some(format!(" {line}")),
        '\\' => Some(format!("\\{line}")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use git2::{IndexAddOption, Repository, Signature};
    use tempfile::tempdir;

    use super::{PreviewKind, PreviewRenderMode, PreviewState};

    #[test]
    fn preview_directory_returns_message() {
        let tmp = tempdir().expect("tmpdir should exist");
        let preview = PreviewState::from_path(tmp.path(), tmp.path(), None);
        assert!(matches!(preview.kind, PreviewKind::Message));
    }

    #[test]
    fn preview_non_git_text_uses_raw_mode() {
        let tmp = tempdir().expect("tmpdir should exist");
        let path = tmp.path().join("file.txt");
        fs::write(&path, "text").expect("write should succeed");

        let preview = PreviewState::from_path(tmp.path(), &path, None);
        assert!(matches!(preview.kind, PreviewKind::Text));
        assert_eq!(preview.render_mode, PreviewRenderMode::Raw);
        assert_eq!(preview.lines, vec!["text".to_string()]);
    }

    #[test]
    fn markdown_defaults_to_markdown_mode() {
        let tmp = tempdir().expect("tmpdir should exist");
        let path = tmp.path().join("README.md");
        fs::write(&path, "# title\n").expect("write should succeed");

        let preview = PreviewState::from_path(tmp.path(), &path, None);
        assert_eq!(preview.render_mode, PreviewRenderMode::Markdown);
        assert_eq!(preview.next_render_mode(), Some(PreviewRenderMode::Raw));
    }

    #[test]
    fn preview_no_diff_falls_back_to_raw_file() {
        let tmp = tempdir().expect("tmpdir should exist");
        let repo = Repository::init(tmp.path()).expect("git init should succeed");
        let path = tmp.path().join("file.txt");
        fs::write(&path, "line1\n").expect("write should succeed");
        commit_all(&repo, "initial");

        let preview = PreviewState::from_path(tmp.path(), &path, None);
        assert!(matches!(preview.kind, PreviewKind::Text));
        assert_eq!(preview.render_mode, PreviewRenderMode::Raw);
        assert_eq!(preview.lines, vec!["line1".to_string()]);
    }

    #[test]
    fn preview_modified_file_can_use_patch_mode() {
        let tmp = tempdir().expect("tmpdir should exist");
        let repo = Repository::init(tmp.path()).expect("git init should succeed");
        let path = tmp.path().join("file.txt");
        fs::write(&path, "line1\n").expect("write should succeed");
        commit_all(&repo, "initial");
        fs::write(&path, "line1\nline2\n").expect("write should succeed");

        let preview = PreviewState::from_path(tmp.path(), &path, Some(PreviewRenderMode::Diff));
        assert!(matches!(preview.kind, PreviewKind::Text));
        assert_eq!(preview.render_mode, PreviewRenderMode::Diff);
        assert!(preview.lines.iter().any(|line| line.starts_with("+line2")));
        assert!(!preview.lines.iter().any(|line| line.starts_with("@@")));
    }

    #[test]
    fn markdown_cycles_markdown_raw_diff_when_available() {
        let tmp = tempdir().expect("tmpdir should exist");
        let repo = Repository::init(tmp.path()).expect("git init should succeed");
        let path = tmp.path().join("README.markdown");
        fs::write(&path, "# Title\n").expect("write should succeed");
        commit_all(&repo, "initial");
        fs::write(&path, "# Title\n\nnew\n").expect("write should succeed");

        let markdown_preview = PreviewState::from_path(tmp.path(), &path, None);
        assert_eq!(markdown_preview.render_mode, PreviewRenderMode::Markdown);
        assert_eq!(
            markdown_preview.next_render_mode(),
            Some(PreviewRenderMode::Raw)
        );

        let raw_preview = PreviewState::from_path(
            tmp.path(),
            &path,
            Some(
                markdown_preview
                    .next_render_mode()
                    .expect("raw mode should exist"),
            ),
        );
        assert_eq!(raw_preview.render_mode, PreviewRenderMode::Raw);
        assert_eq!(
            raw_preview.next_render_mode(),
            Some(PreviewRenderMode::Diff)
        );

        let diff_preview = PreviewState::from_path(
            tmp.path(),
            &path,
            Some(
                raw_preview
                    .next_render_mode()
                    .expect("diff mode should exist"),
            ),
        );
        assert_eq!(diff_preview.render_mode, PreviewRenderMode::Diff);
        assert_eq!(
            diff_preview.next_render_mode(),
            Some(PreviewRenderMode::Markdown)
        );
    }

    #[test]
    fn preview_binary_returns_message() {
        let tmp = tempdir().expect("tmpdir should exist");
        let path = tmp.path().join("file.bin");
        fs::write(&path, vec![0xff, 0xfe, 0xfd]).expect("write should succeed");

        let preview = PreviewState::from_path(tmp.path(), &path, None);
        assert!(matches!(preview.kind, PreviewKind::Message));
    }

    fn commit_all(repo: &Repository, message: &str) {
        let mut index = repo.index().expect("index should open");
        index
            .add_all([Path::new(".")], IndexAddOption::DEFAULT, None)
            .expect("add_all should succeed");
        index.write().expect("index write should succeed");

        let tree_id = index.write_tree().expect("write_tree should succeed");
        let tree = repo.find_tree(tree_id).expect("tree should exist");

        let sig = Signature::now("test", "test@example.com").expect("signature should build");
        let parent_commit = repo
            .head()
            .ok()
            .and_then(|h| h.target())
            .and_then(|oid| repo.find_commit(oid).ok());

        if let Some(parent) = parent_commit.as_ref() {
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[parent])
                .expect("commit should succeed");
        } else {
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])
                .expect("commit should succeed");
        }
    }
}
