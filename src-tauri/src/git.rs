use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::command_log;
use crate::models::{
    BranchOverview, CommitFile, CommitNode, LocalBranch, StashEntry, WorkingTreeFile,
};

pub struct GitOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

pub fn resolve_git_binary() -> Option<PathBuf> {
    if let Some(path) = look_on_path("git") {
        return Some(path);
    }

    for candidate in [
        "/opt/homebrew/bin/git",
        "/usr/local/bin/git",
        "/usr/bin/git",
    ] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }

    if let Ok(output) = Command::new("/bin/zsh")
        .args(["-l", "-c", "command -v git"])
        .output()
    {
        if output.status.success() {
            let resolved = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !resolved.is_empty() {
                let path = PathBuf::from(resolved);
                if path.is_file() {
                    return Some(path);
                }
            }
        }
    }

    None
}

fn look_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|dir| {
        let candidate = dir.join(name);
        candidate.is_file().then_some(candidate)
    })
}

pub fn run_git(git: &Path, repo: &Path, args: &[&str]) -> Result<GitOutput, String> {
    let started = Instant::now();
    let result = Command::new(git)
        .args(args)
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|err| format!("Failed to run git: {err}"));

    match &result {
        Ok(output) => {
            command_log::record(
                repo,
                git,
                args,
                output.status.success(),
                started.elapsed(),
                &String::from_utf8_lossy(&output.stdout),
                &String::from_utf8_lossy(&output.stderr),
            );
        }
        Err(err) => {
            command_log::record(repo, git, args, false, started.elapsed(), "", err);
        }
    }

    let output = result?;
    Ok(GitOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

pub fn combined_message(output: &GitOutput) -> String {
    let stdout = output.stdout.trim();
    let stderr = output.stderr.trim();
    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => String::new(),
        (false, true) => stdout.to_string(),
        (true, false) => stderr.to_string(),
        (false, false) => format!("{stdout}\n{stderr}"),
    }
}

pub fn validate_ref(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 255 {
        return Err("Invalid branch name".into());
    }
    if name.starts_with('/')
        || name.ends_with('/')
        || name.starts_with('.')
        || name.contains("..")
        || name.contains('\\')
    {
        return Err("Invalid branch name".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/' | '+'))
    {
        return Err(format!("Invalid branch name: {name}"));
    }
    Ok(())
}

pub fn repo_root(git: &Path, path: &Path) -> Result<String, String> {
    let output = run_git(git, path, &["rev-parse", "--show-toplevel"])?;
    if !output.success {
        return Err("That folder is not a git repository.".into());
    }
    let root = output.stdout.trim();
    if root.is_empty() {
        return Err("That folder is not a git repository.".into());
    }
    Ok(root.to_string())
}

pub struct LiveStatus {
    pub branch: String,
    pub ahead: u32,
    pub behind: u32,
    pub dirty: bool,
    pub insertions: u32,
    pub deletions: u32,
    pub changed_files: u32,
}

pub fn fetch_remote(git: &Path, repo: &Path) {
    let args = ["fetch", "--prune", "--no-tags"];
    let started = Instant::now();
    let mut child = match Command::new(git)
        .args(args)
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "Never")
        .env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes -o ConnectTimeout=8")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            command_log::record(repo, git, &args, false, started.elapsed(), "", &err.to_string());
            return;
        }
    };

    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                command_log::record(
                    repo,
                    git,
                    &args,
                    status.success(),
                    started.elapsed(),
                    "",
                    if status.success() {
                        ""
                    } else {
                        "fetch exited with a non-zero status"
                    },
                );
                return;
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                command_log::record(
                    repo,
                    git,
                    &args,
                    false,
                    started.elapsed(),
                    "",
                    "fetch timed out after 20s",
                );
                return;
            }
            Ok(None) => thread::sleep(Duration::from_millis(40)),
            Err(err) => {
                command_log::record(repo, git, &args, false, started.elapsed(), "", &err.to_string());
                return;
            }
        }
    }
}

pub fn live_status(git: &Path, repo: &Path) -> Result<LiveStatus, String> {
    let output = run_git(
        git,
        repo,
        &["status", "--porcelain=v2", "--branch", "--untracked-files=all"],
    )?;
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            "Could not read repository status.",
        ));
    }

    let mut branch = String::from("HEAD");
    let mut ahead = 0;
    let mut behind = 0;
    let mut dirty = false;
    let mut changed_files = 0;
    let mut saw_ab = false;

    for line in output.stdout.lines() {
        if let Some(name) = line.strip_prefix("# branch.head ") {
            branch = name.trim().to_string();
            continue;
        }
        if let Some(counts) = line.strip_prefix("# branch.ab ") {
            saw_ab = true;
            let mut parts = counts.split_whitespace();
            ahead = parse_count(parts.next(), '+');
            behind = parse_count(parts.next(), '-');
            continue;
        }
        if !line.is_empty() && !line.starts_with('#') {
            dirty = true;
            changed_files += 1;
        }
    }

    if branch == "HEAD" {
        if let Ok(short) = run_git(git, repo, &["rev-parse", "--short", "HEAD"]) {
            if short.success {
                branch = format!("detached {}", short.stdout.trim());
            } else {
                branch = "detached HEAD".into();
            }
        }
    } else if !saw_ab {
        (ahead, behind) = ahead_behind_for_ref(git, repo, &format!("origin/{branch}"));
    }

    let (insertions, deletions) = if dirty {
        working_tree_line_counts(git, repo)
    } else {
        (0, 0)
    };

    Ok(LiveStatus {
        branch,
        ahead,
        behind,
        dirty,
        insertions,
        deletions,
        changed_files,
    })
}

fn working_tree_line_counts(git: &Path, repo: &Path) -> (u32, u32) {
    let mut insertions = 0;
    let mut deletions = 0;

    if let Ok(output) = run_git(git, repo, &["diff", "--numstat", "HEAD"]) {
        if output.success {
            add_numstat(&output.stdout, &mut insertions, &mut deletions);
        }
    }

    if let Ok(output) = run_git(git, repo, &["ls-files", "--others", "--exclude-standard", "-z"]) {
        if output.success {
            for rel in output.stdout.split('\0').filter(|path| !path.is_empty()) {
                insertions += count_text_lines(&repo.join(rel));
            }
        }
    }

    (insertions, deletions)
}

fn add_numstat(stdout: &str, insertions: &mut u32, deletions: &mut u32) {
    for line in stdout.lines() {
        let mut parts = line.split('\t');
        if let Some(added) = parts.next().and_then(|value| value.parse::<u32>().ok()) {
            *insertions += added;
        }
        if let Some(removed) = parts.next().and_then(|value| value.parse::<u32>().ok()) {
            *deletions += removed;
        }
    }
}

fn count_text_lines(path: &Path) -> u32 {
    match fs::read(path) {
        Ok(bytes) if !bytes.contains(&0) => String::from_utf8_lossy(&bytes).lines().count() as u32,
        _ => 0,
    }
}

fn parse_count(value: Option<&str>, prefix: char) -> u32 {
    value
        .unwrap_or("0")
        .trim_start_matches(prefix)
        .parse()
        .unwrap_or(0)
}

fn ahead_behind_for_ref(git: &Path, repo: &Path, other: &str) -> (u32, u32) {
    let spec = format!("HEAD...{other}");
    let output = match run_git(git, repo, &["rev-list", "--left-right", "--count", &spec]) {
        Ok(output) if output.success => output,
        _ => return (0, 0),
    };
    let mut parts = output.stdout.split_whitespace();
    let ahead = parts.next().and_then(|value| value.parse().ok()).unwrap_or(0);
    let behind = parts.next().and_then(|value| value.parse().ok()).unwrap_or(0);
    (ahead, behind)
}

pub fn current_branch(git: &Path, repo: &Path) -> Result<String, String> {
    let abbrev = run_git(git, repo, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if !abbrev.success {
        return Err(or_fallback(
            &combined_message(&abbrev),
            "Could not read the current branch.",
        ));
    }
    let name = abbrev.stdout.trim().to_string();
    if name != "HEAD" {
        return Ok(name);
    }

    let short = run_git(git, repo, &["rev-parse", "--short", "HEAD"])?;
    if short.success {
        Ok(format!("detached {}", short.stdout.trim()))
    } else {
        Ok("detached HEAD".into())
    }
}

pub fn folder_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.to_string())
}

pub fn ref_exists(git: &Path, repo: &Path, git_ref: &str) -> bool {
    run_git(git, repo, &["show-ref", "--verify", "--quiet", git_ref])
        .map(|output| output.success)
        .unwrap_or(false)
}

pub fn local_branches(git: &Path, repo: &Path) -> Result<Vec<String>, String> {
    let output = run_git(
        git,
        repo,
        &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
    )?;
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            "Could not list local branches.",
        ));
    }
    let mut branches: Vec<String> = output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect();
    branches.sort();
    Ok(branches)
}

fn short_branch_name(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}

fn is_protected_branch(name: &str) -> bool {
    matches!(name, "develop" | "main" | "master")
}

fn candidate_git_ref(name: &str) -> String {
    if let Some(remote) = name.strip_prefix("origin/") {
        format!("refs/remotes/origin/{remote}")
    } else {
        format!("refs/heads/{name}")
    }
}

fn resolve_merge_target(git: &Path, repo: &Path, preferred: Option<&str>) -> Option<String> {
    let mut candidates = Vec::new();
    if let Some(name) = preferred.map(str::trim).filter(|value| !value.is_empty()) {
        candidates.push(format!("origin/{name}"));
        candidates.push(name.to_string());
    }
    for name in ["develop", "main", "master"] {
        candidates.push(format!("origin/{name}"));
        candidates.push(name.to_string());
    }
    let mut seen = std::collections::HashSet::new();
    for candidate in candidates {
        if !seen.insert(candidate.clone()) {
            continue;
        }
        if ref_exists(git, repo, &candidate_git_ref(&candidate)) {
            return Some(candidate);
        }
    }
    None
}

fn ancestor_merged_names(git: &Path, repo: &Path, target: &str) -> std::collections::HashSet<String> {
    let spec = format!("--merged={target}");
    match run_git(
        git,
        repo,
        &[
            "for-each-ref",
            "--format=%(refname:short)",
            &spec,
            "refs/heads",
        ],
    ) {
        Ok(output) if output.success => output
            .stdout
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToString::to_string)
            .collect(),
        _ => std::collections::HashSet::new(),
    }
}

fn branch_patches_in_target(git: &Path, repo: &Path, branch_ref: &str, target: &str) -> bool {
    let output = match run_git(git, repo, &["cherry", target, branch_ref]) {
        Ok(output) if output.success => output,
        _ => return false,
    };
    let mut saw_commit = false;
    for line in output.stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        saw_commit = true;
        if line.starts_with('+') {
            return false;
        }
    }
    saw_commit
}

pub fn branch_overview(
    git: &Path,
    repo: &Path,
    preferred: Option<&str>,
) -> Result<BranchOverview, String> {
    let names = local_branches(git, repo)?;
    let current = current_branch(git, repo).unwrap_or_default();
    let merge_target = resolve_merge_target(git, repo, preferred);
    let target_ref = merge_target
        .as_deref()
        .map(candidate_git_ref);
    let target_short = merge_target
        .as_deref()
        .map(short_branch_name)
        .unwrap_or_default();

    let ancestor_merged = target_ref
        .as_deref()
        .map(|target| ancestor_merged_names(git, repo, target))
        .unwrap_or_default();

    let mut branches: Vec<LocalBranch> = names
        .into_iter()
        .map(|name| {
            let protected_branch = is_protected_branch(&name) || name == target_short;
            let merged = target_ref.as_deref().is_some_and(|target| {
                ancestor_merged.contains(&name)
                    || (!protected_branch
                        && branch_patches_in_target(
                            git,
                            repo,
                            &format!("refs/heads/{name}"),
                            target,
                        ))
            });
            LocalBranch {
                current: name == current,
                merged,
                protected_branch,
                name,
            }
        })
        .collect();
    branches.sort_by(|left, right| {
        right
            .current
            .cmp(&left.current)
            .then(left.merged.cmp(&right.merged))
            .then(left.name.cmp(&right.name))
    });
    Ok(BranchOverview {
        merge_target,
        branches,
    })
}

pub fn delete_local_branch(git: &Path, repo: &Path, branch: &str, force: bool) -> Result<String, String> {
    validate_ref(branch)?;
    let current = current_branch(git, repo)?;
    if current == branch {
        return Err("Cannot delete the branch that is currently checked out.".into());
    }
    if !ref_exists(git, repo, &format!("refs/heads/{branch}")) {
        return Err(format!("Local branch {branch} does not exist."));
    }
    let flag = if force { "-D" } else { "-d" };
    let output = run_git(git, repo, &["branch", flag, branch])?;
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            &format!("Failed to delete {branch}"),
        ));
    }
    Ok(or_fallback(
        &combined_message(&output),
        &format!("Deleted {branch}"),
    ))
}

pub fn delete_merged_branches(
    git: &Path,
    repo: &Path,
    preferred: Option<&str>,
) -> Result<String, String> {
    let overview = branch_overview(git, repo, preferred)?;
    let victims: Vec<String> = overview
        .branches
        .iter()
        .filter(|branch| branch.merged && !branch.current && !branch.protected_branch)
        .map(|branch| branch.name.clone())
        .collect();
    if victims.is_empty() {
        return Ok("No merged local branches to delete.".into());
    }
    let mut deleted = 0;
    let mut errors = Vec::new();
    for name in &victims {
        match delete_local_branch(git, repo, name, false) {
            Ok(_) => deleted += 1,
            Err(err) => errors.push(err),
        }
    }
    if deleted == 0 {
        return Err(errors.join("\n"));
    }
    let mut message = format!(
        "Deleted {deleted} merged {}",
        if deleted == 1 { "branch" } else { "branches" }
    );
    if !errors.is_empty() {
        message.push_str(". ");
        message.push_str(&errors.join(" "));
    }
    Ok(message)
}

pub fn checkout_local_branch(git: &Path, repo: &Path, branch: &str) -> Result<String, String> {
    validate_ref(branch)?;
    let current = current_branch(git, repo)?;
    if current == branch {
        return Ok(format!("Already on {branch}"));
    }
    if !ref_exists(git, repo, &format!("refs/heads/{branch}")) {
        return Err(format!("Local branch {branch} does not exist."));
    }
    let output = run_git(git, repo, &["checkout", branch])?;
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            &format!("Failed to check out {branch}"),
        ));
    }
    Ok(or_fallback(
        &combined_message(&output),
        &format!("Checked out {branch}"),
    ))
}

pub fn create_and_checkout_branch(git: &Path, repo: &Path, branch: &str) -> Result<String, String> {
    validate_ref(branch)?;
    if ref_exists(git, repo, &format!("refs/heads/{branch}")) {
        return Err(format!("Branch {branch} already exists."));
    }
    let output = run_git(git, repo, &["checkout", "-b", branch])?;
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            &format!("Failed to create branch {branch}"),
        ));
    }
    Ok(or_fallback(
        &combined_message(&output),
        &format!("Created and checked out {branch}"),
    ))
}

pub fn rename_local_branch(
    git: &Path,
    repo: &Path,
    branch: &str,
    new_name: &str,
) -> Result<String, String> {
    validate_ref(branch)?;
    validate_ref(new_name)?;
    if branch == new_name {
        return Ok(format!("Already named {new_name}"));
    }
    if !ref_exists(git, repo, &format!("refs/heads/{branch}")) {
        return Err(format!("Local branch {branch} does not exist."));
    }
    if ref_exists(git, repo, &format!("refs/heads/{new_name}")) {
        return Err(format!("Branch {new_name} already exists."));
    }
    let output = run_git(git, repo, &["branch", "-m", branch, new_name])?;
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            &format!("Failed to rename {branch}"),
        ));
    }
    Ok(or_fallback(
        &combined_message(&output),
        &format!("Renamed {branch} to {new_name}"),
    ))
}

/// Standard `git pull` only. Never add `--force` or other overwrite flags.
pub fn pull(git: &Path, repo: &Path) -> Result<String, String> {
    let output = run_git(git, repo, &["pull"])?;
    if !output.success {
        return Err(or_fallback(&combined_message(&output), "Pull failed."));
    }
    Ok(or_fallback(
        &combined_message(&output),
        "Pulled current branch",
    ))
}

/// Standard `git push` only. Never add `--force`, `--force-with-lease`, or `+` refspecs.
pub fn push(git: &Path, repo: &Path) -> Result<String, String> {
    let output = run_git(git, repo, &["push"])?;
    if !output.success {
        return Err(or_fallback(&combined_message(&output), "Push failed."));
    }
    Ok(or_fallback(
        &combined_message(&output),
        "Pushed current branch",
    ))
}

pub fn checkout_with_fallbacks(
    git: &Path,
    repo: &Path,
    branches: &[String],
) -> Result<String, String> {
    let current = current_branch(git, repo)?;
    if branches.first().is_some_and(|target| target == &current) {
        return Ok(format!("Already on {current}"));
    }

    let fetch = run_git(git, repo, &["fetch", "--all", "--prune"])?;
    let fetch_note = if fetch.success {
        String::new()
    } else {
        format!("Fetch warning: {}\n", combined_message(&fetch))
    };

    for branch in branches {
        validate_ref(branch)?;
        if branch == &current {
            return Ok(format!("{fetch_note}Already on {branch}"));
        }
        let local = format!("refs/heads/{branch}");
        if ref_exists(git, repo, &local) {
            let output = run_git(git, repo, &["checkout", branch])?;
            if output.success {
                return Ok(format!("{fetch_note}Checked out {branch}"));
            }
            return Err(format!(
                "{fetch_note}{}",
                or_fallback(&combined_message(&output), &format!("Failed to check out {branch}"))
            ));
        }

        let remote = format!("refs/remotes/origin/{branch}");
        if ref_exists(git, repo, &remote) {
            let remote_ref = format!("origin/{branch}");
            let output = run_git(git, repo, &["checkout", "-B", branch, &remote_ref])?;
            if output.success {
                return Ok(format!(
                    "{fetch_note}Checked out {branch} from origin/{branch}"
                ));
            }
            return Err(format!(
                "{fetch_note}{}",
                or_fallback(
                    &combined_message(&output),
                    &format!("Failed to check out origin/{branch}")
                )
            ));
        }
    }

    Err(format!(
        "{fetch_note}None of these branches exist locally or on origin: {}",
        branches.join(", ")
    ))
}

pub fn log_graph(git: &Path, repo: &Path) -> Result<Vec<CommitNode>, String> {
    let output = run_git(
        git,
        repo,
        &[
            "log",
            "--all",
            "--max-count=400",
            "--pretty=format:%H%x1f%P%x1f%s%x1f%an%x1f%aI%x1f%D",
        ],
    )?;
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            "Could not read the commit log.",
        ));
    }

    let commits = output
        .stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let mut parts = line.split('\u{1f}');
            let hash = parts.next()?.to_string();
            let parents = parts
                .next()?
                .split_whitespace()
                .filter(|parent| !parent.is_empty())
                .map(ToOwned::to_owned)
                .collect();
            Some(CommitNode {
                hash,
                parents,
                subject: parts.next().unwrap_or_default().to_string(),
                author: parts.next().unwrap_or_default().to_string(),
                date: parts.next().unwrap_or_default().to_string(),
                refs: parts.next().unwrap_or_default().to_string(),
            })
        })
        .collect();

    Ok(commits)
}

pub fn discard_all_changes(git: &Path, repo: &Path) -> Result<(), String> {
    let reset = run_git(git, repo, &["reset", "--hard", "HEAD"])?;
    if !reset.success {
        return Err(or_fallback(
            &combined_message(&reset),
            "Failed to discard tracked changes.",
        ));
    }
    let clean = run_git(git, repo, &["clean", "-fd"])?;
    if !clean.success {
        return Err(or_fallback(
            &combined_message(&clean),
            "Failed to remove untracked files.",
        ));
    }
    Ok(())
}

pub fn commit(git: &Path, repo: &Path, title: &str, description: &str) -> Result<String, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("Enter a commit title.".into());
    }
    if title.chars().count() > 72 {
        return Err("Commit title must be 72 characters or fewer.".into());
    }
    if title.contains('\0') || description.contains('\0') {
        return Err("Invalid commit message.".into());
    }

    let staged = run_git(git, repo, &["diff", "--cached", "--quiet"])?;
    if staged.success {
        return Err("Nothing is staged to commit.".into());
    }

    let description = description.trim();
    let output = if description.is_empty() {
        run_git(git, repo, &["commit", "-m", title])?
    } else {
        run_git(git, repo, &["commit", "-m", title, "-m", description])?
    };
    if !output.success {
        return Err(or_fallback(&combined_message(&output), "Commit failed."));
    }
    Ok(or_fallback(
        &combined_message(&output),
        &format!("Committed {title}"),
    ))
}

fn porcelain_path(rest: &str) -> Option<String> {
    let path = rest
        .trim()
        .split(" -> ")
        .last()
        .unwrap_or(rest)
        .trim_matches('"')
        .to_string();
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

fn describe_letter(letter: char) -> String {
    match letter {
        'M' => "Modified".into(),
        'A' => "Added".into(),
        'D' => "Deleted".into(),
        'R' => "Renamed".into(),
        'C' => "Copied".into(),
        'T' => "Type changed".into(),
        'U' => "Conflicted".into(),
        '?' => "Untracked".into(),
        other => other.to_string(),
    }
}

fn validate_commit_hash(hash: &str) -> Result<(), String> {
    if hash.len() < 7 || hash.len() > 64 {
        return Err("Invalid commit.".into());
    }
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid commit.".into());
    }
    Ok(())
}

fn require_commit(git: &Path, repo: &Path, hash: &str) -> Result<String, String> {
    validate_commit_hash(hash)?;
    let spec = format!("{hash}^{{commit}}");
    let output = run_git(git, repo, &["rev-parse", "--verify", "--quiet", &spec])?;
    let resolved = output.stdout.trim();
    if !output.success || resolved.is_empty() {
        return Err("Commit not found.".into());
    }
    Ok(resolved.to_string())
}

fn first_parent(git: &Path, repo: &Path, hash: &str) -> Result<Option<String>, String> {
    let spec = format!("{hash}^");
    let output = run_git(git, repo, &["rev-parse", "--verify", "--quiet", &spec])?;
    let resolved = output.stdout.trim();
    if !output.success || resolved.is_empty() {
        return Ok(None);
    }
    Ok(Some(resolved.to_string()))
}

fn parse_name_status_line(line: &str) -> Option<CommitFile> {
    let mut parts = line.split('\t');
    let status = parts.next()?.trim();
    let letter = status.chars().find(|c| c.is_ascii_alphabetic())?;
    let first_path = name_status_path(parts.next()?);
    if first_path.is_empty() {
        return None;
    }
    let second_path = parts.next().map(name_status_path).filter(|path| !path.is_empty());
    let (path, old_path) = match second_path {
        Some(new_path) => (new_path, Some(first_path)),
        None => (first_path, None),
    };
    Some(CommitFile {
        path,
        old_path,
        status: describe_letter(letter),
    })
}

fn name_status_path(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

pub fn commit_files(git: &Path, repo: &Path, hash: &str) -> Result<Vec<CommitFile>, String> {
    let hash = require_commit(git, repo, hash)?;
    let parent = first_parent(git, repo, &hash)?;
    let output = if let Some(parent) = parent.as_deref() {
        run_git(
            git,
            repo,
            &[
                "diff-tree",
                "--no-commit-id",
                "--name-status",
                "-r",
                "--find-renames",
                parent,
                &hash,
            ],
        )?
    } else {
        run_git(
            git,
            repo,
            &[
                "diff-tree",
                "--no-commit-id",
                "--name-status",
                "-r",
                "--find-renames",
                "--root",
                &hash,
            ],
        )?
    };
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            "Could not read the commit files.",
        ));
    }

    Ok(output
        .stdout
        .lines()
        .filter_map(parse_name_status_line)
        .collect())
}

fn stash_ref(index: u32) -> String {
    format!("stash@{{{index}}}")
}

fn parse_stash_index(selector: &str) -> Option<u32> {
    let start = selector.rfind('{')?;
    let end = selector.rfind('}')?;
    if end <= start + 1 {
        return None;
    }
    selector[start + 1..end].parse().ok()
}

fn parse_stash_line(line: &str) -> Option<StashEntry> {
    let mut parts = line.split('\u{1f}');
    let selector = parts.next()?.trim();
    let index = parse_stash_index(selector)?;
    Some(StashEntry {
        index,
        message: parts.next().unwrap_or_default().to_string(),
        date: parts.next().unwrap_or_default().to_string(),
    })
}

fn missing_stash_ref(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("refs/stash") && (lower.contains("unknown revision") || lower.contains("bad revision") || lower.contains("does not exist") || lower.contains("needed a single revision"))
}

pub fn stash_list(git: &Path, repo: &Path) -> Result<Vec<StashEntry>, String> {
    let output = run_git(
        git,
        repo,
        &["stash", "list", "--pretty=format:%gd%x1f%gs%x1f%aI"],
    )?;
    if !output.success {
        if missing_stash_ref(&combined_message(&output)) {
            return Ok(Vec::new());
        }
        return Err(or_fallback(
            &combined_message(&output),
            "Could not read the stash list.",
        ));
    }
    Ok(output
        .stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(parse_stash_line)
        .collect())
}

fn stash_action(
    git: &Path,
    repo: &Path,
    action: &str,
    index: u32,
    fallback: &str,
    success: &str,
) -> Result<String, String> {
    let spec = stash_ref(index);
    let output = run_git(git, repo, &["stash", action, &spec])?;
    if !output.success {
        return Err(or_fallback(&combined_message(&output), fallback));
    }
    Ok(or_fallback(
        &combined_message(&output),
        &format!("{success} {spec}"),
    ))
}

pub fn stash_apply(git: &Path, repo: &Path, index: u32) -> Result<String, String> {
    stash_action(
        git,
        repo,
        "apply",
        index,
        "Failed to apply the stash.",
        "Applied",
    )
}

pub fn stash_pop(git: &Path, repo: &Path, index: u32) -> Result<String, String> {
    stash_action(
        git,
        repo,
        "pop",
        index,
        "Failed to pop the stash.",
        "Popped",
    )
}

pub fn stash_drop(git: &Path, repo: &Path, index: u32) -> Result<String, String> {
    stash_action(
        git,
        repo,
        "drop",
        index,
        "Failed to drop the stash.",
        "Dropped",
    )
}

pub fn stash_push(git: &Path, repo: &Path, message: &str) -> Result<String, String> {
    if message.contains('\0') {
        return Err("Invalid stash message.".into());
    }
    let files = working_tree(git, repo)?;
    if files.is_empty() {
        return Err("Nothing to stash.".into());
    }
    let message = message.trim();
    let output = if message.is_empty() {
        run_git(git, repo, &["stash", "push", "--include-untracked"])?
    } else {
        run_git(
            git,
            repo,
            &["stash", "push", "--include-untracked", "-m", message],
        )?
    };
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            "Failed to stash changes.",
        ));
    }
    let combined = combined_message(&output);
    if combined.to_ascii_lowercase().contains("no local changes") {
        return Err("Nothing to stash.".into());
    }
    Ok(or_fallback(&combined, "Stashed changes"))
}

pub fn commit_file_diff(git: &Path, repo: &Path, hash: &str, file: &str) -> Result<String, String> {
    require_file_path(file)?;
    let hash = require_commit(git, repo, hash)?;
    let parent = first_parent(git, repo, &hash)?;
    let output = if let Some(parent) = parent.as_deref() {
        run_git(
            git,
            repo,
            &["diff", "--find-renames", parent, &hash, "--", file],
        )?
    } else {
        run_git(
            git,
            repo,
            &["show", "--pretty=format:", "--find-renames", &hash, "--", file],
        )?
    };
    if !output.success && output.stdout.trim().is_empty() {
        return Err(or_fallback(
            &combined_message(&output),
            "Could not read the commit diff.",
        ));
    }
    if output.stdout.trim().is_empty() {
        return Ok("No changes.".into());
    }
    Ok(output.stdout)
}

pub fn working_tree(git: &Path, repo: &Path) -> Result<Vec<WorkingTreeFile>, String> {
    let output = run_git(git, repo, &["status", "--porcelain=v1", "-uall"])?;
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            "Could not read the working tree.",
        ));
    }

    let mut files = Vec::new();
    for line in output.stdout.lines() {
        if line.len() < 4 {
            continue;
        }
        let index = line.as_bytes()[0] as char;
        let worktree = line.as_bytes()[1] as char;
        let Some(path) = porcelain_path(&line[3..]) else {
            continue;
        };
        if index == '?' && worktree == '?' {
            files.push(WorkingTreeFile {
                path,
                status: "Untracked".into(),
                untracked: true,
                staged: false,
            });
            continue;
        }
        if index != ' ' && index != '?' {
            files.push(WorkingTreeFile {
                path: path.clone(),
                status: describe_letter(index),
                untracked: false,
                staged: true,
            });
        }
        if worktree != ' ' && worktree != '?' {
            files.push(WorkingTreeFile {
                path,
                status: describe_letter(worktree),
                untracked: false,
                staged: false,
            });
        }
    }

    Ok(files)
}

fn require_file_path(file: &str) -> Result<(), String> {
    if file.is_empty() || file.contains('\0') {
        return Err("Invalid file path".into());
    }
    Ok(())
}

pub fn stage_file(git: &Path, repo: &Path, file: &str) -> Result<(), String> {
    require_file_path(file)?;
    let output = run_git(git, repo, &["add", "--", file])?;
    if !output.success {
        return Err(or_fallback(&combined_message(&output), "Failed to stage file."));
    }
    Ok(())
}

pub fn stage_all(git: &Path, repo: &Path) -> Result<(), String> {
    let output = run_git(git, repo, &["add", "-A"])?;
    if !output.success {
        return Err(or_fallback(&combined_message(&output), "Failed to stage all files."));
    }
    Ok(())
}

pub fn unstage_file(git: &Path, repo: &Path, file: &str) -> Result<(), String> {
    require_file_path(file)?;
    let output = run_git(git, repo, &["restore", "--staged", "--", file])?;
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            "Failed to unstage file.",
        ));
    }
    Ok(())
}

pub fn unstage_all(git: &Path, repo: &Path) -> Result<(), String> {
    let output = run_git(git, repo, &["restore", "--staged", "."])?;
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            "Failed to unstage all files.",
        ));
    }
    Ok(())
}

pub fn file_diff(git: &Path, repo: &Path, file: &str, staged: bool) -> Result<String, String> {
    require_file_path(file)?;

    if !staged {
        let status = run_git(git, repo, &["status", "--porcelain=v1", "--", file])?;
        let untracked = status.stdout.lines().any(|line| line.starts_with("??"));
        if untracked {
            return untracked_diff(repo, file);
        }
    }

    let output = if staged {
        run_git(git, repo, &["diff", "--cached", "--", file])?
    } else {
        run_git(git, repo, &["diff", "--", file])?
    };
    if !output.success && output.stdout.trim().is_empty() {
        return Err(or_fallback(
            &combined_message(&output),
            "Could not read the file diff.",
        ));
    }

    if output.stdout.trim().is_empty() {
        return Ok(if staged {
            "No staged changes.".into()
        } else {
            "No unstaged changes.".into()
        });
    }

    Ok(output.stdout)
}

fn untracked_diff(repo: &Path, file: &str) -> Result<String, String> {
    let absolute = repo.join(file);
    let contents = std::fs::read_to_string(&absolute).map_err(|err| {
        format!("Could not read untracked file {file}: {err}")
    })?;
    let lines: Vec<&str> = contents.lines().collect();
    let count = lines.len().max(1);
    let mut diff = format!(
        "diff --git a/{file} b/{file}\nnew file mode 100644\n--- /dev/null\n+++ b/{file}\n@@ -0,0 +1,{count} @@\n"
    );
    if contents.is_empty() {
        diff.push_str("+\n");
    } else {
        for line in lines {
            diff.push('+');
            diff.push_str(line);
            diff.push('\n');
        }
    }
    Ok(diff)
}

fn or_fallback(message: &str, fallback: &str) -> String {
    if message.trim().is_empty() {
        fallback.to_string()
    } else {
        message.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn git_bin() -> PathBuf {
        resolve_git_binary().expect("git should be installed for tests")
    }

    fn temp_dir() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "shipyard-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn git(repo: &Path, args: &[&str]) -> GitOutput {
        let output = run_git(&git_bin(), repo, args).unwrap();
        assert!(
            output.success,
            "git {} failed: {}",
            args.join(" "),
            combined_message(&output)
        );
        output
    }

    fn init_repo() -> PathBuf {
        let repo = temp_dir();
        git(&repo, &["init", "-b", "develop"]);
        git(&repo, &["config", "user.name", "Shipyard Test"]);
        git(&repo, &["config", "user.email", "test@shipyard.local"]);
        fs::write(repo.join("README.md"), "hello\n").unwrap();
        git(&repo, &["add", "README.md"]);
        git(&repo, &["commit", "-m", "initial"]);
        repo
    }

    #[test]
    fn reads_branch_and_dirty_status() {
        let repo = init_repo();
        assert_eq!(current_branch(&git_bin(), &repo).unwrap(), "develop");
        assert!(!live_status(&git_bin(), &repo).unwrap().dirty);
        fs::write(repo.join("README.md"), "changed\n").unwrap();
        let dirty = live_status(&git_bin(), &repo).unwrap();
        assert!(dirty.dirty);
        assert_eq!((dirty.insertions, dirty.deletions, dirty.changed_files), (1, 1, 1));
        fs::write(repo.join("new.txt"), "one\ntwo\n").unwrap();
        let with_untracked = live_status(&git_bin(), &repo).unwrap();
        assert_eq!(
            (
                with_untracked.insertions,
                with_untracked.deletions,
                with_untracked.changed_files
            ),
            (3, 1, 2)
        );
        let files = working_tree(&git_bin(), &repo).unwrap();
        assert!(files.iter().any(|file| file.path == "README.md" && !file.staged));
        assert!(!files.iter().any(|file| file.staged));
    }

    #[test]
    fn checkout_skips_when_already_on_target() {
        let repo = init_repo();
        let message = checkout_with_fallbacks(&git_bin(), &repo, &["develop".into()]).unwrap();
        assert!(message.contains("Already on develop"));
        assert_eq!(current_branch(&git_bin(), &repo).unwrap(), "develop");
    }

    #[test]
    fn checkout_skips_fallback_when_already_on_it() {
        let repo = init_repo();
        let message = checkout_with_fallbacks(
            &git_bin(),
            &repo,
            &["missing-feature".into(), "develop".into()],
        )
        .unwrap();
        assert!(message.contains("Already on develop"));
        assert_eq!(current_branch(&git_bin(), &repo).unwrap(), "develop");
    }

    #[test]
    fn checkout_falls_back_when_target_is_missing() {
        let repo = init_repo();
        git(&repo, &["checkout", "-b", "MERP-234"]);
        git(&repo, &["checkout", "develop"]);
        let message = checkout_with_fallbacks(
            &git_bin(),
            &repo,
            &["MERP-123".into(), "MERP-234".into(), "develop".into()],
        )
        .unwrap();
        assert!(message.contains("MERP-234"));
        assert_eq!(current_branch(&git_bin(), &repo).unwrap(), "MERP-234");
    }

    #[test]
    fn diff_includes_untracked_files() {
        let repo = init_repo();
        fs::write(repo.join("new.txt"), "fresh\n").unwrap();
        let diff = file_diff(&git_bin(), &repo, "new.txt", false).unwrap();
        assert!(diff.contains("+fresh"));
        let commits = log_graph(&git_bin(), &repo).unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].subject, "initial");
    }

    #[test]
    fn stages_and_unstages_working_tree_files() {
        let repo = init_repo();
        fs::write(repo.join("new.txt"), "fresh\n").unwrap();
        fs::write(repo.join("README.md"), "changed\n").unwrap();

        let files = working_tree(&git_bin(), &repo).unwrap();
        assert!(files.iter().any(|file| file.path == "new.txt" && !file.staged && file.untracked));
        assert!(files.iter().any(|file| file.path == "README.md" && !file.staged));
        assert!(!files.iter().any(|file| file.staged));

        stage_file(&git_bin(), &repo, "new.txt").unwrap();
        let files = working_tree(&git_bin(), &repo).unwrap();
        assert!(files.iter().any(|file| file.path == "new.txt" && file.staged));
        assert!(!files.iter().any(|file| file.path == "new.txt" && !file.staged));
        assert!(files.iter().any(|file| file.path == "README.md" && !file.staged));

        stage_all(&git_bin(), &repo).unwrap();
        let files = working_tree(&git_bin(), &repo).unwrap();
        assert!(files.iter().all(|file| file.staged));
        assert!(files.iter().any(|file| file.path == "README.md" && file.staged));

        let staged_diff = file_diff(&git_bin(), &repo, "README.md", true).unwrap();
        assert!(staged_diff.contains("-hello") || staged_diff.contains("+changed"));

        unstage_file(&git_bin(), &repo, "new.txt").unwrap();
        let files = working_tree(&git_bin(), &repo).unwrap();
        assert!(files.iter().any(|file| file.path == "new.txt" && !file.staged));
        assert!(!files.iter().any(|file| file.path == "new.txt" && file.staged));

        unstage_all(&git_bin(), &repo).unwrap();
        let files = working_tree(&git_bin(), &repo).unwrap();
        assert!(files.iter().all(|file| !file.staged));
    }

    #[test]
    fn commits_staged_files_with_title_and_description() {
        let repo = init_repo();
        fs::write(repo.join("README.md"), "updated\n").unwrap();
        stage_all(&git_bin(), &repo).unwrap();
        let message = commit(
            &git_bin(),
            &repo,
            "Update readme",
            "Describe the change.",
        )
        .unwrap();
        assert!(message.to_lowercase().contains("update readme") || message.contains("develop"));
        let files = working_tree(&git_bin(), &repo).unwrap();
        assert!(files.is_empty());
        let commits = log_graph(&git_bin(), &repo).unwrap();
        assert_eq!(commits[0].subject, "Update readme");
        assert!(commit(&git_bin(), &repo, "Nothing", "").is_err());
    }

    #[test]
    fn lists_and_checks_out_local_branches() {
        let repo = init_repo();
        git(&repo, &["checkout", "-b", "feature"]);
        let branches = local_branches(&git_bin(), &repo).unwrap();
        assert!(branches.contains(&"develop".into()));
        assert!(branches.contains(&"feature".into()));
        assert_eq!(current_branch(&git_bin(), &repo).unwrap(), "feature");

        let message = checkout_local_branch(&git_bin(), &repo, "develop").unwrap();
        assert!(message.contains("develop"));
        assert_eq!(current_branch(&git_bin(), &repo).unwrap(), "develop");
        assert!(checkout_local_branch(&git_bin(), &repo, "missing").is_err());

        let created = create_and_checkout_branch(&git_bin(), &repo, "task/123").unwrap();
        assert!(created.contains("task/123"));
        assert_eq!(current_branch(&git_bin(), &repo).unwrap(), "task/123");
        assert!(local_branches(&git_bin(), &repo)
            .unwrap()
            .contains(&"task/123".into()));
        assert!(create_and_checkout_branch(&git_bin(), &repo, "task/123").is_err());

        let renamed = rename_local_branch(&git_bin(), &repo, "task/123", "task/456").unwrap();
        assert!(renamed.contains("task/456"));
        assert_eq!(current_branch(&git_bin(), &repo).unwrap(), "task/456");
        let after_rename = local_branches(&git_bin(), &repo).unwrap();
        assert!(after_rename.contains(&"task/456".into()));
        assert!(!after_rename.contains(&"task/123".into()));
        assert!(rename_local_branch(&git_bin(), &repo, "missing", "other").is_err());
        assert!(rename_local_branch(&git_bin(), &repo, "develop", "task/456").is_err());
    }

    #[test]
    fn marks_and_deletes_merged_local_branches() {
        let repo = init_repo();
        git(&repo, &["checkout", "-b", "feature"]);
        fs::write(repo.join("feature.txt"), "work\n").unwrap();
        git(&repo, &["add", "feature.txt"]);
        git(&repo, &["commit", "-m", "feature work"]);
        git(&repo, &["checkout", "develop"]);
        git(&repo, &["merge", "feature"]);

        let overview = branch_overview(&git_bin(), &repo, Some("develop")).unwrap();
        assert_eq!(overview.merge_target.as_deref(), Some("develop"));
        let feature = overview
            .branches
            .iter()
            .find(|branch| branch.name == "feature")
            .unwrap();
        assert!(feature.merged);
        assert!(!feature.current);
        let develop = overview
            .branches
            .iter()
            .find(|branch| branch.name == "develop")
            .unwrap();
        assert!(develop.protected_branch);
        assert!(develop.current);

        assert!(delete_local_branch(&git_bin(), &repo, "develop", false).is_err());
        let deleted = delete_merged_branches(&git_bin(), &repo, Some("develop")).unwrap();
        assert!(deleted.contains("1"));
        git(&repo, &["checkout", "-b", "wip"]);
        fs::write(repo.join("wip.txt"), "unmerged\n").unwrap();
        git(&repo, &["add", "wip.txt"]);
        git(&repo, &["commit", "-m", "unmerged work"]);
        git(&repo, &["checkout", "develop"]);

        let leftover = delete_merged_branches(&git_bin(), &repo, Some("develop")).unwrap();
        assert!(leftover.contains("No merged") || leftover.contains("0"));
        let names = local_branches(&git_bin(), &repo).unwrap();
        assert!(names.contains(&"wip".into()));
        assert!(names.contains(&"develop".into()));
        assert!(!names.contains(&"feature".into()));
        assert!(delete_local_branch(&git_bin(), &repo, "develop", true).is_err());
    }

    #[test]
    fn marks_squash_merged_local_branches() {
        let repo = init_repo();
        git(&repo, &["checkout", "-b", "feature"]);
        fs::write(repo.join("feature.txt"), "work\n").unwrap();
        git(&repo, &["add", "feature.txt"]);
        git(&repo, &["commit", "-m", "feature work"]);
        git(&repo, &["checkout", "develop"]);
        git(&repo, &["merge", "--squash", "feature"]);
        git(&repo, &["commit", "-m", "squash feature"]);

        let overview = branch_overview(&git_bin(), &repo, Some("develop")).unwrap();
        let feature = overview
            .branches
            .iter()
            .find(|branch| branch.name == "feature")
            .unwrap();
        assert!(feature.merged);
        assert!(!feature.current);
    }

    #[test]
    fn push_and_pull_use_standard_git_without_force() {
        let seed = init_repo();
        let origin = temp_dir();
        git(&origin, &["init", "--bare", "-b", "develop"]);
        git(&seed, &["remote", "add", "origin", origin.to_str().unwrap()]);
        git(&seed, &["push", "-u", "origin", "develop"]);

        let work = temp_dir();
        git(&work, &["clone", origin.to_str().unwrap(), "."]);
        git(&work, &["config", "user.name", "Shipyard Test"]);
        git(&work, &["config", "user.email", "test@shipyard.local"]);
        fs::write(work.join("README.md"), "from work\n").unwrap();
        git(&work, &["add", "README.md"]);
        git(&work, &["commit", "-m", "work commit"]);
        push(&git_bin(), &work).unwrap();

        let other = temp_dir();
        git(&other, &["clone", origin.to_str().unwrap(), "."]);
        git(&other, &["config", "user.name", "Shipyard Test"]);
        git(&other, &["config", "user.email", "test@shipyard.local"]);
        assert!(fs::read_to_string(other.join("README.md"))
            .unwrap()
            .contains("from work"));

        fs::write(work.join("README.md"), "from upstream\n").unwrap();
        git(&work, &["add", "README.md"]);
        git(&work, &["commit", "-m", "upstream commit"]);
        push(&git_bin(), &work).unwrap();
        pull(&git_bin(), &other).unwrap();
        assert!(fs::read_to_string(other.join("README.md"))
            .unwrap()
            .contains("from upstream"));
    }

    #[test]
    fn reports_ahead_and_behind_counts() {
        let upstream = init_repo();
        let work = temp_dir();
        git(&work, &["clone", upstream.to_str().unwrap(), "."]);
        git(&work, &["config", "user.name", "Shipyard Test"]);
        git(&work, &["config", "user.email", "test@shipyard.local"]);
        let even = live_status(&git_bin(), &work).unwrap();
        assert_eq!((even.ahead, even.behind), (0, 0));

        fs::write(upstream.join("README.md"), "upstream\n").unwrap();
        git(&upstream, &["add", "README.md"]);
        git(&upstream, &["commit", "-m", "upstream commit"]);
        git(&work, &["fetch"]);
        let behind = live_status(&git_bin(), &work).unwrap();
        assert_eq!((behind.ahead, behind.behind), (0, 1));

        fs::write(work.join("local.txt"), "mine\n").unwrap();
        git(&work, &["add", "local.txt"]);
        git(&work, &["commit", "-m", "local commit"]);
        let diverged = live_status(&git_bin(), &work).unwrap();
        assert_eq!((diverged.ahead, diverged.behind), (1, 1));
    }

    #[test]
    fn lists_commit_files_and_file_diff() {
        let repo = init_repo();
        let commits = log_graph(&git_bin(), &repo).unwrap();
        let initial = &commits[0].hash;
        let files = commit_files(&git_bin(), &repo, initial).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "README.md");
        assert_eq!(files[0].status, "Added");
        let added = commit_file_diff(&git_bin(), &repo, initial, "README.md").unwrap();
        assert!(added.contains("+hello"));

        fs::write(repo.join("README.md"), "changed\n").unwrap();
        fs::write(repo.join("new.txt"), "fresh\n").unwrap();
        git(&repo, &["add", "README.md", "new.txt"]);
        git(&repo, &["commit", "-m", "update files"]);
        let commits = log_graph(&git_bin(), &repo).unwrap();
        let update = &commits[0].hash;
        let files = commit_files(&git_bin(), &repo, update).unwrap();
        assert!(files.iter().any(|file| file.path == "README.md" && file.status == "Modified"));
        assert!(files.iter().any(|file| file.path == "new.txt" && file.status == "Added"));
        let diff = commit_file_diff(&git_bin(), &repo, update, "README.md").unwrap();
        assert!(diff.contains("-hello") || diff.contains("+changed"));

        git(&repo, &["mv", "new.txt", "renamed.txt"]);
        git(&repo, &["commit", "-m", "rename file"]);
        let commits = log_graph(&git_bin(), &repo).unwrap();
        let renamed = commit_files(&git_bin(), &repo, &commits[0].hash).unwrap();
        assert!(renamed.iter().any(|file| {
            file.path == "renamed.txt"
                && file.status == "Renamed"
                && file.old_path.as_deref() == Some("new.txt")
        }));

        git(&repo, &["rm", "renamed.txt"]);
        git(&repo, &["commit", "-m", "remove file"]);
        let commits = log_graph(&git_bin(), &repo).unwrap();
        let deleted = commit_files(&git_bin(), &repo, &commits[0].hash).unwrap();
        assert!(deleted.iter().any(|file| file.path == "renamed.txt" && file.status == "Deleted"));

        assert!(commit_files(&git_bin(), &repo, "not-a-hash").is_err());
        assert!(commit_file_diff(&git_bin(), &repo, initial, "").is_err());
    }

    #[test]
    fn lists_applies_pops_and_drops_stashes() {
        let repo = init_repo();
        assert!(stash_list(&git_bin(), &repo).unwrap().is_empty());
        assert!(stash_apply(&git_bin(), &repo, 0).is_err());
        assert!(stash_push(&git_bin(), &repo, "").is_err());

        fs::write(repo.join("README.md"), "stashed-a\n").unwrap();
        fs::write(repo.join("notes.txt"), "untracked\n").unwrap();
        stash_push(&git_bin(), &repo, "first stash").unwrap();
        assert!(working_tree(&git_bin(), &repo).unwrap().is_empty());
        assert!(!repo.join("notes.txt").exists());
        fs::write(repo.join("README.md"), "stashed-b\n").unwrap();
        git(&repo, &["stash", "push", "-m", "second stash"]);

        let stashes = stash_list(&git_bin(), &repo).unwrap();
        assert_eq!(stashes.len(), 2);
        assert_eq!(stashes[0].index, 0);
        assert_eq!(stashes[1].index, 1);
        assert!(stashes[0].message.contains("second stash"));
        assert!(stashes[1].message.contains("first stash"));

        stash_apply(&git_bin(), &repo, 0).unwrap();
        let files = working_tree(&git_bin(), &repo).unwrap();
        assert!(files.iter().any(|file| file.path == "README.md"));
        assert_eq!(stash_list(&git_bin(), &repo).unwrap().len(), 2);
        assert!(fs::read_to_string(repo.join("README.md"))
            .unwrap()
            .contains("stashed-b"));

        git(&repo, &["checkout", "--", "README.md"]);
        stash_drop(&git_bin(), &repo, 1).unwrap();
        let after_drop = stash_list(&git_bin(), &repo).unwrap();
        assert_eq!(after_drop.len(), 1);
        assert!(after_drop[0].message.contains("second stash"));

        stash_pop(&git_bin(), &repo, 0).unwrap();
        assert!(stash_list(&git_bin(), &repo).unwrap().is_empty());
        assert!(fs::read_to_string(repo.join("README.md"))
            .unwrap()
            .contains("stashed-b"));
    }

    #[test]
    fn pull_from_named_branch_into_current() {
        let upstream = init_repo();
        fs::write(upstream.join("README.md"), "upstream develop\n").unwrap();
        git(&upstream, &["add", "README.md"]);
        git(&upstream, &["commit", "-m", "develop update"]);

        let work = temp_dir();
        git(&work, &["clone", upstream.to_str().unwrap(), "."]);
        git(&work, &["checkout", "-b", "feature"]);
        let pull = run_git(&git_bin(), &work, &["pull", "origin", "develop"]).unwrap();
        assert!(pull.success, "{}", combined_message(&pull));
        assert_eq!(current_branch(&git_bin(), &work).unwrap(), "feature");
        let contents = fs::read_to_string(work.join("README.md")).unwrap();
        assert!(contents.contains("upstream develop"));
    }
}
