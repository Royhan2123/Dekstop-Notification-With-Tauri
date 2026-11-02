//! Adapter Windows: membaca "kartu media" sistem (SMTC) secara event-driven.
//! Satu thread pekerja memiliki semua objek WinRT; event hanya mengirim sinyal `Dirty`.
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use windows::Foundation::TypedEventHandler;
use windows::Media::Control::{
    CurrentSessionChangedEventArgs, GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionMediaProperties as Properties,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status, SessionsChangedEventArgs,
};
use windows::Storage::Streams::{DataReader, IInputStream};
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize};
use windows::core::{Interface, RuntimeType};

use super::artwork;
use super::position_now;
use crate::domain::error::MediaError;
use crate::domain::model::{MediaCommand, MediaInfo};
use crate::domain::ports::MediaSource;

/// Jeda penggabungan event beruntun (satu lagu baru memicu banyak event sekaligus).
const DEBOUNCE: Duration = Duration::from_millis(100);
const CONTROL_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_ARTWORK_BYTES: u64 = 8 * 1024 * 1024;
/// Selisih detik antara epoch Windows (1601) dan Unix (1970), dalam satuan 100 ns.
const FILETIME_UNIX_EPOCH: i64 = 116_444_736_000_000_000;

enum Msg {
    Dirty,
    Control {
        session_id: String,
        cmd: MediaCommand,
        reply: Sender<Result<(), MediaError>>,
    },
}

#[derive(Default)]
pub struct WindowsSmtc {
    tx: Mutex<Option<Sender<Msg>>>,
}

impl MediaSource for WindowsSmtc {
    fn start(&self, on_change: Box<dyn Fn(Vec<MediaInfo>) + Send + Sync>) {
        let mut slot = self.tx.lock().unwrap_or_else(|e| e.into_inner());
        if slot.is_some() {
            return;
        }
        let (tx, rx) = channel();
        let worker_tx = tx.clone();
        match thread::Builder::new()
            .name("smtc".into())
            .spawn(move || run(rx, worker_tx, on_change))
        {
            Ok(_) => *slot = Some(tx),
            Err(e) => tracing::error!("gagal memulai thread SMTC: {e}"),
        }
    }

    fn control(&self, session_id: &str, cmd: MediaCommand) -> Result<(), MediaError> {
        let tx = self
            .tx
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .ok_or(MediaError::NoSession)?;
        let (reply, answer) = channel();
        tx.send(Msg::Control {
            session_id: session_id.to_string(),
            cmd,
            reply,
        })
        .map_err(|_| MediaError::Failed("thread SMTC berhenti".into()))?;
        answer
            .recv_timeout(CONTROL_TIMEOUT)
            .map_err(|_| MediaError::Failed("SMTC tidak merespons".into()))?
    }
}

fn run(rx: Receiver<Msg>, tx: Sender<Msg>, on_change: Box<dyn Fn(Vec<MediaInfo>) + Send + Sync>) {
    // SAFETY: dipanggil sekali di thread pekerja sebelum memakai WinRT; error "sudah diinisialisasi" aman diabaikan.
    unsafe {
        let _ = RoInitialize(RO_INIT_MULTITHREADED);
    }
    let manager = match Manager::RequestAsync().and_then(|op| op.join()) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("SMTC tidak tersedia: {e}");
            return;
        }
    };
    if let Err(e) = manager
        .SessionsChanged(&dirty::<Manager, SessionsChangedEventArgs>(tx.clone()))
        .and(
            manager.CurrentSessionChanged(&dirty::<Manager, CurrentSessionChangedEventArgs>(
                tx.clone(),
            )),
        )
    {
        tracing::error!("gagal mendaftar event SMTC: {e}");
        return;
    }

    let mut worker = Worker {
        manager,
        tx,
        on_change,
        registered: Vec::new(),
        artwork: HashMap::new(),
    };
    worker.refresh();

    while let Ok(first) = rx.recv() {
        if matches!(first, Msg::Dirty) {
            thread::sleep(DEBOUNCE);
        }
        let mut dirty = false;
        for msg in std::iter::once(first).chain(rx.try_iter()) {
            match msg {
                Msg::Dirty => dirty = true,
                Msg::Control {
                    session_id,
                    cmd,
                    reply,
                } => {
                    let _ = reply.send(worker.control(&session_id, cmd));
                }
            }
        }
        if dirty {
            worker.refresh();
        }
    }
}

fn dirty<S: RuntimeType + 'static, A: RuntimeType + 'static>(
    tx: Sender<Msg>,
) -> TypedEventHandler<S, A> {
    TypedEventHandler::new(move |_, _| {
        let _ = tx.send(Msg::Dirty);
        Ok(())
    })
}

struct Registered {
    session: Session,
    tokens: [i64; 3],
}

struct Worker {
    manager: Manager,
    tx: Sender<Msg>,
    on_change: Box<dyn Fn(Vec<MediaInfo>) + Send + Sync>,
    registered: Vec<Registered>,
    /// Sampul per sesi: (kunci "judul|artis", data URL, warna).
    artwork: HashMap<String, (String, String, String)>,
}

impl Worker {
    fn refresh(&mut self) {
        let sessions = match self.manager.GetSessions() {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("gagal membaca sesi media: {e}");
                return;
            }
        };
        self.unregister();
        let mut out = Vec::new();
        for session in sessions {
            self.register(&session);
            match self.read(&session) {
                Ok(info) => out.push(info),
                Err(e) => tracing::debug!("sesi media dilewati: {e}"),
            }
        }
        let alive: Vec<&String> = out.iter().map(|i| &i.session_id).collect();
        self.artwork.retain(|id, _| alive.contains(&id));
        (self.on_change)(out);
    }

    fn unregister(&mut self) {
        for r in self.registered.drain(..) {
            let _ = r.session.RemoveMediaPropertiesChanged(r.tokens[0]);
            let _ = r.session.RemovePlaybackInfoChanged(r.tokens[1]);
            let _ = r.session.RemoveTimelinePropertiesChanged(r.tokens[2]);
        }
    }

    fn register(&mut self, session: &Session) {
        let tokens = || -> windows::core::Result<[i64; 3]> {
            Ok([
                session.MediaPropertiesChanged(&dirty(self.tx.clone()))?,
                session.PlaybackInfoChanged(&dirty(self.tx.clone()))?,
                session.TimelinePropertiesChanged(&dirty(self.tx.clone()))?,
            ])
        };
        match tokens() {
            Ok(tokens) => self.registered.push(Registered {
                session: session.clone(),
                tokens,
            }),
            Err(e) => tracing::warn!("gagal mendaftar event sesi: {e}"),
        }
    }

    fn read(&mut self, session: &Session) -> windows::core::Result<MediaInfo> {
        let session_id = session.SourceAppUserModelId()?.to_string();
        let props = session.TryGetMediaPropertiesAsync()?.join()?;
        let title = props.Title()?.to_string();
        let artist = props.Artist()?.to_string();

        let playback = session.GetPlaybackInfo()?;
        let playing = playback.PlaybackStatus()? == Status::Playing;
        let rate = playback
            .PlaybackRate()
            .and_then(|r| r.Value())
            .unwrap_or(1.0);

        let timeline = session.GetTimelineProperties()?;
        let ms = |ticks: i64| ticks / 10_000; // TimeSpan: satuan 100 ns
        let start = ms(timeline.StartTime()?.Duration);
        let duration = (ms(timeline.EndTime()?.Duration) - start).max(0);
        let reported = ms(timeline.Position()?.Duration) - start;
        let updated = timeline.LastUpdatedTime()?.UniversalTime;
        let elapsed = if updated > 0 {
            ms(now_filetime() - updated)
        } else {
            0
        };

        let key = format!("{title}|{artist}");
        let (thumbnail, color) = self.artwork_for(&session_id, key, &props);
        Ok(MediaInfo {
            session_id,
            title,
            artist,
            playing,
            position_ms: position_now(reported, elapsed, rate, playing, duration),
            duration_ms: duration.clamp(0, u32::MAX as i64) as u32,
            rate,
            thumbnail,
            color,
            ..Default::default()
        })
    }

    fn artwork_for(
        &mut self,
        session_id: &str,
        key: String,
        props: &Properties,
    ) -> (Option<String>, Option<String>) {
        if let Some((k, url, color)) = self.artwork.get(session_id)
            && *k == key
        {
            return (Some(url.clone()), Some(color.clone()));
        }
        // Sampul bisa belum siap saat judul baru berganti; hanya yang berhasil disimpan.
        match read_thumbnail(props).and_then(|b| artwork::process(&b)) {
            Some(art) => {
                self.artwork.insert(
                    session_id.to_string(),
                    (key, art.data_url.clone(), art.color.clone()),
                );
                (Some(art.data_url), Some(art.color))
            }
            None => {
                self.artwork.remove(session_id);
                (None, None)
            }
        }
    }

    fn control(&self, session_id: &str, cmd: MediaCommand) -> Result<(), MediaError> {
        let fail = |e: windows::core::Error| MediaError::Failed(e.to_string());
        let session = self
            .manager
            .GetSessions()
            .map_err(fail)?
            .into_iter()
            .find(|s| s.SourceAppUserModelId().is_ok_and(|id| id == session_id))
            .ok_or(MediaError::NoSession)?;
        let op = match cmd {
            MediaCommand::Previous => session.TrySkipPreviousAsync(),
            MediaCommand::TogglePlay => session.TryTogglePlayPauseAsync(),
            MediaCommand::Next => session.TrySkipNextAsync(),
        }
        .map_err(fail)?;
        if op.join().map_err(fail)? {
            Ok(())
        } else {
            Err(MediaError::Failed("aplikasi menolak perintah".into()))
        }
    }
}

fn read_thumbnail(props: &Properties) -> Option<Vec<u8>> {
    let stream = props.Thumbnail().ok()?.OpenReadAsync().ok()?.join().ok()?;
    let size = stream.Size().ok()?;
    if size == 0 || size > MAX_ARTWORK_BYTES {
        return None;
    }
    let input: IInputStream = stream.cast().ok()?;
    let reader = DataReader::CreateDataReader(&input).ok()?;
    reader.LoadAsync(size as u32).ok()?.join().ok()?;
    let mut buf = vec![0u8; size as usize];
    reader.ReadBytes(&mut buf).ok()?;
    Some(buf)
}

fn now_filetime() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| (d.as_nanos() / 100) as i64 + FILETIME_UNIX_EPOCH)
        .unwrap_or(0)
}
