use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::models::{CommitNode, WorkingTreeFile};

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
    let output = Command::new(git)
        .args(args)
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|err| format!("Failed to run git: {err}"))?;

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
}

pub fn fetch_remote(git: &Path, repo: &Path) {
    let mut child = match Command::new(git)
        .args(["fetch", "--prune", "--no-tags"])
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
        Err(_) => return,
    };

    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return;
            }
            Ok(None) => thread::sleep(Duration::from_millis(40)),
            Err(_) => return,
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

    Ok(LiveStatus {
        branch,
        ahead,
        behind,
        dirty,
    })
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

pub fn is_dirty(git: &Path, repo: &Path) -> Result<bool, String> {
    let output = run_git(git, repo, &["status", "--porcelain"])?;
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            "Could not read repository status.",
        ));
    }
    Ok(!output.stdout.trim().is_empty())
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

pub fn checkout_with_fallbacks(
    git: &Path,
    repo: &Path,
    branches: &[String],
) -> Result<String, String> {
    let fetch = run_git(git, repo, &["fetch", "--all", "--prune"])?;
    let fetch_note = if fetch.success {
        String::new()
    } else {
        format!("Fetch warning: {}\n", combined_message(&fetch))
    };

    for branch in branches {
        validate_ref(branch)?;
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

pub fn working_tree(git: &Path, repo: &Path) -> Result<Vec<WorkingTreeFile>, String> {
    let output = run_git(git, repo, &["status", "--porcelain=v1", "-uall"])?;
    if !output.success {
        return Err(or_fallback(
            &combined_message(&output),
            "Could not read the working tree.",
        ));
    }

    let files = output
        .stdout
        .lines()
        .filter_map(|line| {
            if line.len() < 4 {
                return None;
            }
            let code = line[..2].to_string();
            let rest = line[3..].trim();
            if rest.is_empty() {
                return None;
            }
            let path = rest
                .split(" -> ")
                .last()
                .unwrap_or(rest)
                .trim_matches('"')
                .to_string();
            Some(WorkingTreeFile {
                untracked: code == "??",
                status: describe_status(&code),
                path,
            })
        })
        .collect();

    Ok(files)
}

pub fn file_diff(git: &Path, repo: &Path, file: &str) -> Result<String, String> {
    if file.is_empty() || file.contains('\0') {
        return Err("Invalid file path".into());
    }

    let status = run_git(git, repo, &["status", "--porcelain=v1", "--", file])?;
    let untracked = status.stdout.lines().any(|line| line.starts_with("??"));

    if untracked {
        return untracked_diff(repo, file);
    }

    let output = run_git(git, repo, &["diff", "HEAD", "--", file])?;
    if !output.success && output.stdout.trim().is_empty() {
        return Err(or_fallback(
            &combined_message(&output),
            "Could not read the file diff.",
        ));
    }

    if output.stdout.trim().is_empty() {
        return Ok("No changes versus HEAD.".into());
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

fn describe_status(code: &str) -> String {
    match code {
        "??" => "Untracked".into(),
        "A " | "A?" => "Added".into(),
        "M " | " M" | "MM" => "Modified".into(),
        "D " | " D" => "Deleted".into(),
        "R " | "RM" => "Renamed".into(),
        "C " => "Copied".into(),
        "UU" | "AA" | "DD" => "Conflicted".into(),
        other => other.trim().to_string(),
    }
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
            "krakdown-test-{}-{}",
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
        git(&repo, &["config", "user.name", "Krakdown Test"]);
        git(&repo, &["config", "user.email", "test@krakdown.local"]);
        fs::write(repo.join("README.md"), "hello\n").unwrap();
        git(&repo, &["add", "README.md"]);
        git(&repo, &["commit", "-m", "initial"]);
        repo
    }

    #[test]
    fn reads_branch_and_dirty_status() {
        let repo = init_repo();
        assert_eq!(current_branch(&git_bin(), &repo).unwrap(), "develop");
        assert!(!is_dirty(&git_bin(), &repo).unwrap());
        fs::write(repo.join("README.md"), "changed\n").unwrap();
        assert!(is_dirty(&git_bin(), &repo).unwrap());
        let files = working_tree(&git_bin(), &repo).unwrap();
        assert!(files.iter().any(|file| file.path == "README.md"));
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
        let diff = file_diff(&git_bin(), &repo, "new.txt").unwrap();
        assert!(diff.contains("+fresh"));
        let commits = log_graph(&git_bin(), &repo).unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].subject, "initial");
    }

    #[test]
    fn reports_ahead_and_behind_counts() {
        let upstream = init_repo();
        let work = temp_dir();
        git(&work, &["clone", upstream.to_str().unwrap(), "."]);
        git(&work, &["config", "user.name", "Krakdown Test"]);
        git(&work, &["config", "user.email", "test@krakdown.local"]);
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
