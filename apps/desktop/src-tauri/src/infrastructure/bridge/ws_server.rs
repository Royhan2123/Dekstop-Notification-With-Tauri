//! Server WebSocket di loopback untuk ekstensi Isla Link. Hanya satu ekstensi aktif;
//! sambungan baru menggantikan yang lama. Semua keputusan otorisasi ada di `BridgeHandler`.
use std::io;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tungstenite::{Message, WebSocket};

use crate::domain::bridge::{
    BridgeCommand, BrowserMedia, MAX_BROWSER_SESSIONS, is_extension_origin, sanitize_media,
    tab_of_session,
};
use crate::domain::error::BridgeError;
use crate::domain::model::MediaCommand;
use crate::domain::ports::{BridgeHandler, BrowserBridge};

/// Ekstensi mencoba port-port ini berurutan; satu yang bebas dipakai aplikasi.
pub const PORT_RANGE: std::ops::RangeInclusive<u16> = 47821..=47830;
const AUTH_DEADLINE: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(100);

enum Outgoing {
    Text(String),
    Close,
}

struct Client {
    id: u64,
    tx: Sender<Outgoing>,
}

#[derive(Default)]
pub struct WsBridge {
    client: Mutex<Option<Client>>,
    next_id: AtomicU64,
}

fn forbidden(reason: &str) -> ErrorResponse {
    let mut res = ErrorResponse::new(Some(reason.to_string()));
    *res.status_mut() = tungstenite::http::StatusCode::FORBIDDEN;
    res
}

fn is_idle(e: &tungstenite::Error) -> bool {
    matches!(e, tungstenite::Error::Io(io) if matches!(io.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut))
}

fn command_json(cmd: &BridgeCommand) -> String {
    match cmd {
        BridgeCommand::SetRate(rate) => json!({ "type": "set_rate", "rate": rate }),
        BridgeCommand::Search(query) => json!({ "type": "search", "query": query }),
        BridgeCommand::MediaControl { tab_id, action } => json!({
            "type": "media_control",
            "id": crate::domain::bridge::session_id(*tab_id),
            "action": match action {
                MediaCommand::Previous => "previous",
                MediaCommand::TogglePlay => "toggle",
                MediaCommand::Next => "next",
            },
        }),
    }
    .to_string()
}

/// Pesan `media` dari ekstensi -> daftar sesi yang sah (maks `MAX_BROWSER_SESSIONS`); entri rusak
/// dilewati. Data ini tidak dipercaya, jadi semuanya lewat `sanitize_media`.
pub(crate) fn parse_media(msg: &Value) -> Vec<BrowserMedia> {
    let Some(list) = msg.get("sessions").and_then(Value::as_array) else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|s| {
            let text = |k: &str| s.get(k).and_then(Value::as_str).unwrap_or("");
            let num = |k: &str| s.get(k).and_then(Value::as_f64).unwrap_or(0.0);
            let tab_id = tab_of_session(text("id"))?;
            sanitize_media(
                tab_id,
                text("title"),
                text("artist"),
                s.get("playing").and_then(Value::as_bool).unwrap_or(false),
                num("position_ms"),
                num("duration_ms"),
                s.get("rate").and_then(Value::as_f64).unwrap_or(1.0),
                text("artwork"),
            )
        })
        .take(MAX_BROWSER_SESSIONS)
        .collect()
}

fn send_json(ws: &mut WebSocket<TcpStream>, value: &Value) -> bool {
    ws.send(Message::text(value.to_string())).is_ok()
}

impl WsBridge {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Buka server di port bebas pertama dari `PORT_RANGE`; mengembalikan port yang dipakai.
    pub fn start(self: &Arc<Self>, handler: Arc<dyn BridgeHandler>) -> io::Result<u16> {
        let listener = PORT_RANGE
            .clone()
            .find_map(|p| TcpListener::bind((Ipv4Addr::LOCALHOST, p)).ok())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::AddrInUse, "semua port Isla Link terpakai")
            })?;
        let port = listener.local_addr()?.port();
        let this = Arc::clone(self);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let (this, handler) = (Arc::clone(&this), handler.clone());
                thread::spawn(move || this.serve(stream, handler));
            }
        });
        Ok(port)
    }

    // Tipe galat callback jabat tangan ditetapkan tungstenite (ErrorResponse besar).
    #[allow(clippy::result_large_err)]
    fn serve(&self, stream: TcpStream, handler: Arc<dyn BridgeHandler>) {
        let mut origin = String::new();
        let accepted = tungstenite::accept_hdr(stream, |req: &Request, res: Response| {
            let value = req
                .headers()
                .get("origin")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            // Halaman web biasa tidak boleh bicara ke aplikasi lewat localhost.
            if !is_extension_origin(value) {
                return Err(forbidden("origin ditolak"));
            }
            origin = value.to_string();
            Ok(res)
        });
        let Ok(mut ws) = accepted else { return };
        let _ = ws.get_ref().set_read_timeout(Some(POLL));
        if !Self::authenticate(&mut ws, &origin, handler.as_ref()) {
            let _ = ws.close(None);
            return;
        }

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        if let Some(old) = self.client().replace(Client { id, tx }) {
            let _ = old.tx.send(Outgoing::Close);
        }
        handler.connection_changed(true);
        Self::pump(&mut ws, &rx, handler.as_ref());
        let _ = ws.close(None);
        let mut slot = self.client();
        // Yang sudah digantikan sambungan baru tidak boleh mematikan status.
        if slot.as_ref().is_some_and(|c| c.id == id) {
            *slot = None;
            drop(slot);
            handler.media(Vec::new());
            handler.connection_changed(false);
        }
    }

    /// Pesan pertama harus `pair` (kode) atau `hello` (token) dalam batas waktu.
    fn authenticate(
        ws: &mut WebSocket<TcpStream>,
        origin: &str,
        handler: &dyn BridgeHandler,
    ) -> bool {
        let deadline = Instant::now() + AUTH_DEADLINE;
        while Instant::now() < deadline {
            let text = match ws.read() {
                Ok(Message::Text(t)) => t,
                Ok(Message::Ping(_) | Message::Pong(_)) => continue,
                Ok(_) => return false,
                Err(e) if is_idle(&e) => continue,
                Err(_) => return false,
            };
            let msg: Value = serde_json::from_str(text.as_str()).unwrap_or_default();
            let field = |k: &str| msg.get(k).and_then(Value::as_str).unwrap_or("");
            return match field("type") {
                "pair" => match handler.pair(origin, field("code")) {
                    Some(token) => send_json(ws, &json!({ "type": "paired", "token": token })),
                    None => {
                        let _ = send_json(ws, &json!({ "type": "error", "reason": "pairing" }));
                        false
                    }
                },
                "hello" if handler.authenticate(origin, field("token")) => {
                    send_json(ws, &json!({ "type": "ready" }))
                }
                _ => {
                    let _ = send_json(ws, &json!({ "type": "error", "reason": "auth" }));
                    false
                }
            };
        }
        false
    }

    /// Kirim perintah antrean, balas `ping` agar service worker MV3 tetap hidup.
    fn pump(ws: &mut WebSocket<TcpStream>, rx: &Receiver<Outgoing>, handler: &dyn BridgeHandler) {
        loop {
            loop {
                match rx.try_recv() {
                    Ok(Outgoing::Text(t)) => {
                        if ws.send(Message::text(t)).is_err() {
                            return;
                        }
                    }
                    Ok(Outgoing::Close) | Err(TryRecvError::Disconnected) => return,
                    Err(TryRecvError::Empty) => break,
                }
            }
            match ws.read() {
                Ok(Message::Text(t)) => {
                    let msg = serde_json::from_str::<Value>(t.as_str()).unwrap_or_default();
                    match msg.get("type").and_then(Value::as_str) {
                        Some("ping") => {
                            if !send_json(ws, &json!({ "type": "pong" })) {
                                return;
                            }
                        }
                        Some("media") => handler.media(parse_media(&msg)),
                        _ => {}
                    }
                }
                Ok(Message::Close(_)) => return,
                Ok(_) => {}
                Err(e) if is_idle(&e) => {}
                Err(_) => return,
            }
        }
    }

    fn client(&self) -> std::sync::MutexGuard<'_, Option<Client>> {
        self.client.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl BrowserBridge for WsBridge {
    fn send(&self, cmd: &BridgeCommand) -> Result<(), BridgeError> {
        self.client()
            .as_ref()
            .ok_or(BridgeError::NotConnected)?
            .tx
            .send(Outgoing::Text(command_json(cmd)))
            .map_err(|_| BridgeError::NotConnected)
    }

    fn disconnect(&self) {
        if let Some(c) = self.client().as_ref() {
            let _ = c.tx.send(Outgoing::Close);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tungstenite::client::IntoClientRequest;

    const ORIGIN: &str = "chrome-extension://abcdef";

    #[derive(Default)]
    struct Handler {
        events: Mutex<Vec<bool>>,
    }
    impl BridgeHandler for Handler {
        fn pair(&self, origin: &str, code: &str) -> Option<String> {
            (origin == ORIGIN && code == "123456").then(|| "tok".into())
        }
        fn authenticate(&self, origin: &str, token: &str) -> bool {
            origin == ORIGIN && token == "tok"
        }
        fn connection_changed(&self, connected: bool) {
            self.events.lock().unwrap().push(connected);
        }
    }

    #[test]
    fn pesan_media_diurai_dibatasi_dan_entri_rusak_dilewati() {
        let msg = json!({"type": "media", "sessions": [
            {"id": "browser:5", "title": "Lagu", "artist": "Band", "playing": true,
             "position_ms": 1500.0, "duration_ms": 200000.0, "rate": 1.0,
             "artwork": "https://i.ytimg.com/a.jpg"},
            {"id": "tab:9", "title": "Salah id"},
            {"id": "browser:6", "title": "   "},
            {"id": "browser:7", "title": "Tanpa angka"},
        ]});
        let got = parse_media(&msg);
        assert_eq!(got.len(), 2);
        assert_eq!(
            (got[0].tab_id, got[0].playing, got[0].position_ms),
            (5, true, 1500)
        );
        assert_eq!((got[1].tab_id, got[1].duration_ms), (7, 0));
        let many: Vec<Value> = (0..30)
            .map(|i| json!({"id": format!("browser:{i}"), "title": "x"}))
            .collect();
        assert_eq!(
            parse_media(&json!({"sessions": many})).len(),
            MAX_BROWSER_SESSIONS
        );
        assert!(parse_media(&json!({"sessions": "bukan daftar"})).is_empty());
    }

    #[test]
    fn perintah_kontrol_media_menjadi_json_untuk_ekstensi() {
        let v: Value = serde_json::from_str(&command_json(&BridgeCommand::MediaControl {
            tab_id: 12,
            action: MediaCommand::TogglePlay,
        }))
        .unwrap();
        assert_eq!(v["type"], "media_control");
        assert_eq!(v["id"], "browser:12");
        assert_eq!(v["action"], "toggle");
    }

    type Client = WebSocket<tungstenite::stream::MaybeTlsStream<TcpStream>>;

    fn start() -> (Arc<WsBridge>, Arc<Handler>, u16) {
        let (bridge, handler) = (WsBridge::new(), Arc::new(Handler::default()));
        let port = bridge.start(handler.clone()).unwrap();
        (bridge, handler, port)
    }

    fn connect(port: u16, origin: Option<&str>) -> Result<Client, tungstenite::Error> {
        let mut req = format!("ws://127.0.0.1:{port}")
            .into_client_request()
            .unwrap();
        if let Some(o) = origin {
            req.headers_mut().insert("Origin", o.parse().unwrap());
        }
        tungstenite::connect(req).map(|(ws, _)| ws)
    }

    fn say(ws: &mut Client, v: Value) {
        ws.send(Message::text(v.to_string())).unwrap();
    }

    fn hear(ws: &mut Client) -> Value {
        match ws.read().unwrap() {
            Message::Text(t) => serde_json::from_str(t.as_str()).unwrap(),
            other => panic!("{other:?}"),
        }
    }

    fn wait(mut cond: impl FnMut() -> bool) {
        for _ in 0..100 {
            if cond() {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("kondisi tidak tercapai");
    }

    #[test]
    fn pairing_lalu_hello_menerima_perintah_dan_ping_dibalas() {
        let (bridge, handler, port) = start();
        let mut a = connect(port, Some(ORIGIN)).unwrap();
        say(&mut a, json!({"type": "pair", "code": "123456"}));
        assert_eq!(hear(&mut a), json!({"type": "paired", "token": "tok"}));
        wait(|| *handler.events.lock().unwrap() == vec![true]);

        bridge.send(&BridgeCommand::SetRate(1.5)).unwrap();
        assert_eq!(hear(&mut a), json!({"type": "set_rate", "rate": 1.5}));
        say(&mut a, json!({"type": "ping"}));
        assert_eq!(hear(&mut a), json!({"type": "pong"}));

        // Sambungan baru (hello) menggantikan yang lama tanpa status "putus" palsu.
        let mut b = connect(port, Some(ORIGIN)).unwrap();
        say(&mut b, json!({"type": "hello", "token": "tok"}));
        assert_eq!(hear(&mut b), json!({"type": "ready"}));
        bridge.send(&BridgeCommand::Search("lofi".into())).unwrap();
        assert_eq!(hear(&mut b), json!({"type": "search", "query": "lofi"}));
        thread::sleep(Duration::from_millis(300));
        assert_eq!(*handler.events.lock().unwrap(), vec![true, true]);

        bridge.disconnect();
        wait(|| *handler.events.lock().unwrap() == vec![true, true, false]);
        assert!(matches!(
            bridge.send(&BridgeCommand::SetRate(1.0)),
            Err(BridgeError::NotConnected)
        ));
    }

    #[test]
    fn asal_non_ekstensi_dan_tanpa_asal_ditolak_saat_jabat_tangan() {
        let (_bridge, _h, port) = start();
        assert!(connect(port, Some("https://evil.example")).is_err());
        assert!(connect(port, None).is_err());
    }

    #[test]
    fn kode_atau_token_salah_dijawab_error_lalu_diputus() {
        let (bridge, handler, port) = start();
        let mut a = connect(port, Some(ORIGIN)).unwrap();
        say(&mut a, json!({"type": "pair", "code": "000000"}));
        assert_eq!(hear(&mut a), json!({"type": "error", "reason": "pairing"}));
        let mut b = connect(port, Some(ORIGIN)).unwrap();
        say(&mut b, json!({"type": "hello", "token": "salah"}));
        assert_eq!(hear(&mut b), json!({"type": "error", "reason": "auth"}));
        let mut c = connect(port, Some(ORIGIN)).unwrap();
        say(&mut c, json!({"type": "perintah-aneh"}));
        assert_eq!(hear(&mut c), json!({"type": "error", "reason": "auth"}));
        assert!(handler.events.lock().unwrap().is_empty());
        assert!(matches!(
            bridge.send(&BridgeCommand::SetRate(1.0)),
            Err(BridgeError::NotConnected)
        ));
    }
}
