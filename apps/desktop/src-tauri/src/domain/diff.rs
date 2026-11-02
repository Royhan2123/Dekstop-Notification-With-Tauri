//! Membaca unified diff: baris yang ditambahkan beserta berkas dan nomor barisnya.

pub struct AddedLine<'a> {
    pub file: &'a str,
    pub line: u32,
    pub text: &'a str,
}

pub fn added_lines(diff: &str) -> Vec<AddedLine<'_>> {
    let mut out = Vec::new();
    let mut file = "";
    let mut next_line = 0u32;
    for raw in diff.lines() {
        if let Some(path) = raw.strip_prefix("+++ ") {
            file = path.strip_prefix("b/").unwrap_or(path);
        } else if raw.starts_with("--- ") || raw.starts_with("diff --git") {
            // header berkas; nomor baris diatur ulang oleh hunk berikutnya
        } else if let Some(rest) = raw.strip_prefix("@@") {
            next_line = hunk_new_start(rest).unwrap_or(0);
        } else if let Some(text) = raw.strip_prefix('+') {
            out.push(AddedLine {
                file,
                line: next_line,
                text,
            });
            next_line += 1;
        } else if raw.starts_with(' ') {
            next_line += 1;
        }
    }
    out
}

/// "@@ -a,b +c,d @@ ..." -> c
fn hunk_new_start(header: &str) -> Option<u32> {
    let plus = header.split_whitespace().find(|p| p.starts_with('+'))?;
    plus[1..].split(',').next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIFF: &str = "diff --git a/src/a.rs b/src/a.rs\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -1,3 +10,4 @@ fn x\n ctx\n+baru1\n-lama\n+baru2\n ctx2\ndiff --git a/b.txt b/b.txt\n--- a/b.txt\n+++ b/b.txt\n@@ -0,0 +1 @@\n+isi\n";

    #[test]
    fn baris_tambahan_punya_berkas_dan_nomor_yang_benar() {
        let lines = added_lines(DIFF);
        let got: Vec<_> = lines.iter().map(|l| (l.file, l.line, l.text)).collect();
        assert_eq!(
            got,
            vec![
                ("src/a.rs", 11, "baru1"),
                ("src/a.rs", 12, "baru2"),
                ("b.txt", 1, "isi"),
            ]
        );
    }
}
