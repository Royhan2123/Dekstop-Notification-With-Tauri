use std::io::Write;
use std::process::{Command, Stdio};

use crate::domain::error::AiError;
use crate::domain::ports::AiClient;

/// Memakai langganan Claude Code lewat `claude -p`; prompt dikirim lewat stdin.
pub struct ClaudeCli {
    model: String,
}

impl ClaudeCli {
    pub fn new(model: &str) -> Self {
        Self {
            model: model.into(),
        }
    }

    fn command(&self) -> Command {
        // `claude` dari npm adalah `claude.cmd` di Windows; cmd.exe yang bisa menemukannya.
        #[cfg(windows)]
        let mut cmd = {
            use std::os::windows::process::CommandExt;
            let mut c = Command::new("cmd");
            c.args(["/C", "claude"]).creation_flags(0x0800_0000); // CREATE_NO_WINDOW
            c
        };
        #[cfg(not(windows))]
        let mut cmd = Command::new("claude");
        cmd.arg("-p");
        if !self.model.is_empty() {
            cmd.args(["--model", &self.model]);
        }
        cmd
    }
}

impl AiClient for ClaudeCli {
    fn complete(&self, system: &str, user: &str) -> Result<String, AiError> {
        let mut child = self
            .command()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AiError::Unavailable(format!("Claude CLI: {e}")))?;
        let prompt = format!("{system}\n\n{user}");
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .map_err(|e| AiError::Unavailable(format!("Claude CLI: {e}")))?;
        }
        let out = child
            .wait_with_output()
            .map_err(|e| AiError::Unavailable(format!("Claude CLI: {e}")))?;
        if !out.status.success() {
            return Err(AiError::Unavailable(
                "Claude CLI gagal; pastikan `claude` terpasang dan sudah login".into(),
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}
