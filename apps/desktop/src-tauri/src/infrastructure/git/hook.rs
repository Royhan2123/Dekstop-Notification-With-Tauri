use std::path::{Path, PathBuf};

use crate::domain::error::GitError;
use crate::domain::ports::HookInstaller;

use super::cli_git::git;

const MARKER: &str = "# isla-desk-hook";

/// Memasang `.git/hooks/pre-push` yang memanggil `isla-desk hook pre-push`.
pub struct GitHookInstaller {
    exe: PathBuf,
    /// Berkas endpoint IPC aplikasi; hook memakainya untuk menghubungi Isla.
    ipc_file: PathBuf,
}

/// Git di Windows menjalankan hook lewat sh: pakai garis miring depan dan kutip tunggal.
fn sh_quote(path: &Path) -> String {
    let p = path
        .to_string_lossy()
        .replace('\\', "/")
        .replace('\'', r"'\''");
    format!("'{p}'")
}

impl GitHookInstaller {
    pub fn new(exe: PathBuf, ipc_file: PathBuf) -> Self {
        Self { exe, ipc_file }
    }

    fn hook_path(repo: &Path) -> Result<PathBuf, GitError> {
        let out = git(repo, &["rev-parse", "--git-path", "hooks/pre-push"], None)?;
        if !out.ok {
            return Err(GitError::Command(out.stderr));
        }
        let p = PathBuf::from(out.stdout.trim());
        Ok(if p.is_absolute() { p } else { repo.join(p) })
    }

    fn script(&self) -> String {
        let (exe, ipc) = (sh_quote(&self.exe), sh_quote(&self.ipc_file));
        format!("#!/bin/sh\n{MARKER}\nexec {exe} hook pre-push --ipc {ipc} \"$@\"\n")
    }
}

fn is_ours(path: &Path) -> bool {
    std::fs::read_to_string(path).is_ok_and(|s| s.contains(MARKER))
}

impl HookInstaller for GitHookInstaller {
    fn install(&self, repo: &Path) -> Result<(), GitError> {
        let path = Self::hook_path(repo)?;
        if path.exists() && !is_ours(&path) {
            return Err(GitError::HookExists);
        }
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| GitError::Command(e.to_string()))?;
        }
        std::fs::write(&path, self.script()).map_err(|e| GitError::Command(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755));
        }
        Ok(())
    }

    fn uninstall(&self, repo: &Path) -> Result<(), GitError> {
        let path = Self::hook_path(repo)?;
        if is_ours(&path) {
            std::fs::remove_file(&path).map_err(|e| GitError::Command(e.to_string()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::git::cli_git::tests::temp_repo;

    #[test]
    fn pasang_menolak_menimpa_hook_lain_dan_copot_hanya_milik_isla() {
        let repo = temp_repo("hook");
        let inst = GitHookInstaller::new(
            PathBuf::from(r"C:\Program Files\Isla's\isla-desk.exe"),
            PathBuf::from(r"C:\Users\x\AppData\ipc.json"),
        );
        inst.install(&repo).unwrap();
        inst.install(&repo).unwrap(); // idempoten
        let path = GitHookInstaller::hook_path(&repo).unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains(
            r#"exec 'C:/Program Files/Isla'\''s/isla-desk.exe' hook pre-push --ipc 'C:/Users/x/AppData/ipc.json' "$@""#
        ));
        inst.uninstall(&repo).unwrap();
        assert!(!path.exists());

        std::fs::write(&path, "#!/bin/sh\necho milik-orang\n").unwrap();
        assert!(matches!(inst.install(&repo), Err(GitError::HookExists)));
        inst.uninstall(&repo).unwrap();
        assert!(path.exists(), "hook milik orang lain tidak boleh dihapus");
        let _ = std::fs::remove_dir_all(&repo);
    }
}
