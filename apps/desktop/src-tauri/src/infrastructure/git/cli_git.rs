use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::domain::error::GitError;
use crate::domain::model::review::{Diff, PushRef};
use crate::domain::ports::GitRepo;

const MAX_DIFF_BYTES: usize = 200_000;
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
/// Tidak ikut direview: lockfile dan berkas hasil build.
const DEFAULT_EXCLUDES: &[&str] = &[
    "*.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "*.min.js",
    "*.min.css",
    "*.map",
];

pub struct CliGit;

pub struct Output {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn git(repo: &Path, args: &[&str], stdin: Option<&str>) -> Result<Output, GitError> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo).args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let mut child = cmd
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| GitError::Command(format!("tidak bisa menjalankan git: {e}")))?;
    if let (Some(text), Some(mut pipe)) = (stdin, child.stdin.take()) {
        pipe.write_all(text.as_bytes())
            .map_err(|e| GitError::Command(e.to_string()))?;
    }
    let out = child
        .wait_with_output()
        .map_err(|e| GitError::Command(e.to_string()))?;
    Ok(Output {
        ok: out.status.success(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
    })
}

fn must(repo: &Path, args: &[&str]) -> Result<String, GitError> {
    let out = git(repo, args, None)?;
    if out.ok {
        Ok(out.stdout)
    } else {
        Err(GitError::Command(out.stderr))
    }
}

fn is_zero(sha: &str) -> bool {
    sha.chars().all(|c| c == '0')
}

/// Titik awal diff: SHA remote, atau induk commit tertua yang belum ada di remote manapun.
fn base_for(repo: &Path, r: &PushRef) -> Result<String, GitError> {
    if !is_zero(&r.remote_sha) {
        return Ok(r.remote_sha.clone());
    }
    let revs = must(
        repo,
        &["rev-list", "--reverse", &r.local_sha, "--not", "--remotes"],
    )?;
    Ok(match revs.lines().next() {
        Some(oldest) => format!("{oldest}^"),
        None => r.local_sha.clone(),
    })
}

fn excludes(repo: &Path) -> Vec<String> {
    let custom = std::fs::read_to_string(repo.join(".islaignore")).unwrap_or_default();
    let custom = custom
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'));
    DEFAULT_EXCLUDES
        .iter()
        .copied()
        .chain(custom)
        .map(|p| format!(":(exclude){p}"))
        .collect()
}

impl GitRepo for CliGit {
    fn diff_for_push(&self, repo: &Path, refs: &[PushRef]) -> Result<Diff, GitError> {
        let mut text = String::new();
        for r in refs.iter().filter(|r| !is_zero(&r.local_sha)) {
            let mut base = base_for(repo, r)?;
            // `X^` gagal untuk commit akar; diff terhadap pohon kosong.
            if base.ends_with('^') && !git(repo, &["rev-parse", "--verify", "-q", &base], None)?.ok
            {
                base = EMPTY_TREE.into();
            }
            let ex = excludes(repo);
            let mut args = vec![
                "diff",
                "--no-color",
                "--no-ext-diff",
                &base,
                &r.local_sha,
                "--",
                ".",
            ];
            args.extend(ex.iter().map(String::as_str));
            text.push_str(&must(repo, &args)?);
        }
        let truncated = text.len() > MAX_DIFF_BYTES;
        if truncated {
            let mut cut = MAX_DIFF_BYTES;
            while !text.is_char_boundary(cut) {
                cut -= 1;
            }
            text.truncate(text[..cut].rfind('\n').unwrap_or(cut));
            text.push('\n');
        }
        Ok(Diff { text, truncated })
    }

    fn toplevel(&self, path: &Path) -> Result<String, GitError> {
        Ok(must(path, &["rev-parse", "--show-toplevel"])?
            .trim()
            .to_string())
    }

    fn check_patch(&self, repo: &Path, patch: &str) -> Result<(), GitError> {
        let out = git(repo, &["apply", "--check", "-"], Some(patch))?;
        if out.ok {
            Ok(())
        } else {
            Err(GitError::BadPatch(out.stderr))
        }
    }

    fn apply_and_commit(&self, repo: &Path, patch: &str, message: &str) -> Result<(), GitError> {
        if !git(repo, &["diff", "--cached", "--quiet"], None)?.ok {
            return Err(GitError::StagedChanges);
        }
        self.check_patch(repo, patch)?;
        let applied = git(repo, &["apply", "--index", "-"], Some(patch))?;
        if !applied.ok {
            return Err(GitError::BadPatch(applied.stderr));
        }
        if let Err(e) = must(repo, &["commit", "-m", message]) {
            // Jangan tinggalkan perubahan staged bila commit gagal.
            let _ = git(repo, &["reset", "--hard", "-q", "HEAD"], None);
            return Err(e);
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Repo sementara dengan satu commit; dipakai juga oleh tes lain.
    pub fn temp_repo(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("isla-{name}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.email", "t@t"],
            vec!["config", "user.name", "t"],
            vec!["config", "commit.gpgsign", "false"],
            vec!["config", "core.autocrlf", "false"],
        ] {
            assert!(git(&dir, &args, None).unwrap().ok);
        }
        dir
    }

    pub fn commit_file(dir: &Path, file: &str, content: &str) -> String {
        std::fs::write(dir.join(file), content).unwrap();
        must(dir, &["add", "-A"]).unwrap();
        must(dir, &["commit", "-qm", "c"]).unwrap();
        must(dir, &["rev-parse", "HEAD"])
            .unwrap()
            .trim()
            .to_string()
    }

    fn push_ref(local: &str, remote: &str) -> PushRef {
        PushRef {
            local_ref: "refs/heads/main".into(),
            local_sha: local.into(),
            remote_ref: "refs/heads/main".into(),
            remote_sha: remote.into(),
        }
    }

    #[test]
    fn diff_cabang_baru_dan_pembaruan_tanpa_lockfile() {
        let dir = temp_repo("diff");
        let first = commit_file(&dir, "a.txt", "satu\n");
        let new_branch = CliGit
            .diff_for_push(&dir, &[push_ref(&first, &"0".repeat(40))])
            .unwrap();
        assert!(new_branch.text.contains("+satu"));

        std::fs::write(dir.join("Cargo.lock"), "abaikan\n").unwrap();
        let second = commit_file(&dir, "a.txt", "satu\ndua\n");
        let update = CliGit
            .diff_for_push(&dir, &[push_ref(&second, &first)])
            .unwrap();
        assert!(update.text.contains("+dua") && !update.text.contains("-satu"));
        assert!(!update.text.contains("abaikan"));
        assert!(!update.truncated);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn islaignore_dan_pemotongan_diff_besar() {
        let dir = temp_repo("ignore");
        let base = commit_file(&dir, "x.txt", "x\n");
        std::fs::write(dir.join(".islaignore"), "# komentar\ngenerated/*\n").unwrap();
        std::fs::create_dir_all(dir.join("generated")).unwrap();
        std::fs::write(dir.join("generated/g.txt"), "rahasia-generated\n").unwrap();
        let big = "baris panjang sekali\n".repeat(20_000);
        std::fs::write(dir.join("big.txt"), big).unwrap();
        let head = commit_file(&dir, "y.txt", "y\n");
        let d = CliGit
            .diff_for_push(&dir, &[push_ref(&head, &base)])
            .unwrap();
        assert!(!d.text.contains("rahasia-generated"));
        assert!(d.truncated && d.text.len() <= MAX_DIFF_BYTES + 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn patch_diterapkan_dan_di_commit_atau_ditolak() {
        let dir = temp_repo("patch");
        commit_file(&dir, "a.txt", "satu\n");
        let patch = "--- a/a.txt\n+++ b/a.txt\n@@ -1 +1 @@\n-satu\n+SATU\n";
        CliGit.check_patch(&dir, patch).unwrap();
        CliGit.apply_and_commit(&dir, patch, "fix").unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("a.txt")).unwrap(),
            "SATU\n"
        );
        assert!(matches!(
            CliGit.check_patch(&dir, patch),
            Err(GitError::BadPatch(_))
        ));

        std::fs::write(dir.join("b.txt"), "b\n").unwrap();
        must(&dir, &["add", "b.txt"]).unwrap();
        let again = "--- a/a.txt\n+++ b/a.txt\n@@ -1 +1 @@\n-SATU\n+Satu\n";
        assert!(matches!(
            CliGit.apply_and_commit(&dir, again, "fix"),
            Err(GitError::StagedChanges)
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn toplevel_dan_bukan_repo() {
        let dir = temp_repo("top");
        assert!(!CliGit.toplevel(&dir).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
        assert!(CliGit.toplevel(&dir).is_err(), "folder yang sudah hilang");
    }
}
