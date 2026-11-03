//! Jalur antara proses CLI (`isla-desk hook` / `isla-desk notify`) dan aplikasi yang sedang berjalan:
//! TCP di loopback dengan token acak yang ditulis ke berkas yang hanya bisa dibaca pemilik akun.
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::application::review::service::Verdict;
use crate::domain::model::review::PushRef;

const MAX_REQUEST_BYTES: u64 = 1 << 20;

#[derive(Serialize, Deserialize)]
struct Endpoint {
    port: u16,
    token: String,
}

#[derive(Serialize, Deserialize)]
struct Request {
    token: String,
    #[serde(default)]
    repo: String,
    #[serde(default)]
    refs: Vec<String>,
    /// Bila ada, ini pemberitahuan dari terminal, bukan permintaan review.
    #[serde(default)]
    notify: Option<String>,
    /// Bila benar, minta saran pesan commit untuk diff staged di `repo`.
    #[serde(default)]
    commit_message: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Response {
    pub allow: bool,
    pub message: String,
}

pub type Handler = Arc<dyn Fn(&str, &[PushRef]) -> Verdict + Send + Sync>;
pub type Notifier = Arc<dyn Fn(&str) + Send + Sync>;
/// Saran pesan commit untuk sebuah repo, atau teks galat untuk pengguna.
pub type Suggester = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

/// Buka pendengar di port acak dan tulis `ipc_file`; tiap sambungan dilayani thread sendiri
/// karena jawaban baru ada setelah pengguna memutuskan.
pub fn serve(
    ipc_file: &Path,
    handler: Handler,
    notifier: Notifier,
    suggester: Suggester,
) -> std::io::Result<()> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let endpoint = Endpoint {
        port: listener.local_addr()?.port(),
        token: uuid::Uuid::new_v4().simple().to_string(),
    };
    if let Some(dir) = ipc_file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(ipc_file, serde_json::to_vec(&endpoint)?)?;
    let token = endpoint.token;
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let (handler, notifier, suggester, token) = (
                handler.clone(),
                notifier.clone(),
                suggester.clone(),
                token.clone(),
            );
            thread::spawn(move || {
                let _ = serve_one(stream, &token, &handler, &notifier, &suggester);
            });
        }
    });
    Ok(())
}

fn serve_one(
    stream: TcpStream,
    token: &str,
    handler: &Handler,
    notifier: &Notifier,
    suggester: &Suggester,
) -> std::io::Result<()> {
    let mut line = String::new();
    BufReader::new((&stream).take(MAX_REQUEST_BYTES)).read_line(&mut line)?;
    // Token salah atau permintaan rusak: tutup tanpa jawaban.
    let Ok(req) = serde_json::from_str::<Request>(&line) else {
        return Ok(());
    };
    if req.token != token {
        return Ok(());
    }
    if let Some(text) = &req.notify {
        notifier(text);
        return (&stream).write_all(
            b"{\"allow\":true,\"message\":\"\"}
",
        );
    }
    if req.commit_message {
        let res = match suggester(&req.repo) {
            Ok(message) => Response {
                allow: true,
                message,
            },
            Err(message) => Response {
                allow: false,
                message,
            },
        };
        let mut out = serde_json::to_vec(&res)?;
        out.push(b'\n');
        return (&stream).write_all(&out);
    }
    let refs: Vec<PushRef> = req.refs.iter().filter_map(|l| PushRef::parse(l)).collect();
    let res = match handler(&req.repo, &refs) {
        Verdict::Allow => Response {
            allow: true,
            message: String::new(),
        },
        Verdict::Block(message) => Response {
            allow: false,
            message,
        },
    };
    let mut out = serde_json::to_vec(&res)?;
    out.push(b'\n');
    (&stream).write_all(&out)
}

/// Tanya aplikasi; `None` bila aplikasi tidak berjalan atau tidak menjawab (push dilanjutkan).
pub fn ask(ipc_file: &Path, repo: &str, refs: Vec<String>) -> Option<Response> {
    exchange(ipc_file, |token| Request {
        token,
        repo: repo.into(),
        refs,
        notify: None,
        commit_message: false,
    })
}

/// Minta saran pesan commit; `None` bila aplikasi tidak berjalan.
pub fn commit_message(ipc_file: &Path, repo: &str) -> Option<Response> {
    exchange(ipc_file, |token| Request {
        token,
        repo: repo.into(),
        refs: Vec::new(),
        notify: None,
        commit_message: true,
    })
}

/// Kirim pemberitahuan ke pulau; `false` bila aplikasi tidak berjalan.
pub fn notify(ipc_file: &Path, text: &str) -> bool {
    exchange(ipc_file, |token| Request {
        token,
        repo: String::new(),
        refs: Vec::new(),
        notify: Some(text.into()),
        commit_message: false,
    })
    .is_some()
}

fn exchange(ipc_file: &Path, build: impl FnOnce(String) -> Request) -> Option<Response> {
    let endpoint: Endpoint = serde_json::from_slice(&std::fs::read(ipc_file).ok()?).ok()?;
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, endpoint.port));
    let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).ok()?;
    let req = build(endpoint.token);
    let mut out = serde_json::to_vec(&req).ok()?;
    out.push(b'\n');
    (&stream).write_all(&out).ok()?;
    let mut line = String::new();
    BufReader::new(&stream).read_line(&mut line).ok()?;
    serde_json::from_str(&line).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_suggest() -> Suggester {
        Arc::new(|_| Err("tidak dipakai".into()))
    }

    fn temp_file() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("isla-ipc-{}.json", uuid::Uuid::new_v4()))
    }

    #[test]
    fn permintaan_sampai_ke_handler_dan_jawabannya_kembali() {
        let file = temp_file();
        let handler: Handler = Arc::new(|repo, refs| {
            if repo == "/blok" {
                Verdict::Block(format!("{} ref", refs.len()))
            } else {
                Verdict::Allow
            }
        });
        serve(&file, handler, Arc::new(|_| {}), no_suggest()).unwrap();
        let line = "refs/heads/m aaa refs/heads/m bbb".to_string();
        let ok = ask(&file, "/ok", vec![line.clone()]).unwrap();
        assert_eq!(
            ok,
            Response {
                allow: true,
                message: String::new()
            }
        );
        let blocked = ask(&file, "/blok", vec![line, "rusak".into()]).unwrap();
        assert_eq!(
            blocked,
            Response {
                allow: false,
                message: "1 ref".into()
            }
        );
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn pemberitahuan_terminal_sampai_ke_notifier_tanpa_memanggil_review() {
        let file = temp_file();
        let got = Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = got.clone();
        let handler: Handler = Arc::new(|_, _| Verdict::Block("bukan review".into()));
        let notifier: Notifier = Arc::new(move |t| sink.lock().unwrap().push(t.to_string()));
        serve(&file, handler, notifier, no_suggest()).unwrap();
        assert!(notify(&file, "build selesai"));
        assert_eq!(*got.lock().unwrap(), vec!["build selesai".to_string()]);
        assert!(!notify(&temp_file(), "x"), "aplikasi mati");
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn saran_pesan_commit_dan_galatnya_sampai_ke_terminal() {
        let file = temp_file();
        let handler: Handler = Arc::new(|_, _| Verdict::Block("bukan review".into()));
        let suggester: Suggester = Arc::new(|repo| {
            if repo == "/ada" {
                Ok("feat: sesuatu".into())
            } else {
                Err("tidak ada perubahan staged".into())
            }
        });
        serve(&file, handler, Arc::new(|_| {}), suggester).unwrap();
        let ok = commit_message(&file, "/ada").unwrap();
        assert_eq!(
            ok,
            Response {
                allow: true,
                message: "feat: sesuatu".into()
            }
        );
        let err = commit_message(&file, "/kosong").unwrap();
        assert!(!err.allow);
        assert_eq!(err.message, "tidak ada perubahan staged");
        assert!(
            commit_message(&temp_file(), "/ada").is_none(),
            "aplikasi mati"
        );
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn token_salah_dan_aplikasi_mati_tidak_menghasilkan_jawaban() {
        let file = temp_file();
        let handler: Handler = Arc::new(|_, _| Verdict::Block("harusnya tak dipanggil".into()));
        serve(&file, handler, Arc::new(|_| {}), no_suggest()).unwrap();
        let mut e: Endpoint = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        e.token = "salah".into();
        std::fs::write(&file, serde_json::to_vec(&e).unwrap()).unwrap();
        assert_eq!(ask(&file, "/r", vec![]), None);
        assert_eq!(
            ask(&temp_file(), "/r", vec![]),
            None,
            "tanpa berkas endpoint"
        );
        let _ = std::fs::remove_file(&file);
    }
}
