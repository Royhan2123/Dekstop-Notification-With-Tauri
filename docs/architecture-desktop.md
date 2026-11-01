# Architecture — Isla Desk (Dynamic Island untuk Desktop)

> **Bagaimana** aplikasi dibangun. Baca `prd-desktop.md` dulu.
> Ditulis sebagai panduan kerja langsung untuk Claude Code.

## 1. Prinsip

1. **Lokal sepenuhnya.** Tanpa server & database online. Internet hanya ke penyedia AI pilihan pengguna dan cek update.
2. **Clean Architecture (Hexagonal / Ports & Adapters) di backend Rust.** Logika bisnis tidak tahu OS, Tauri, atau penyedia AI.
3. **MVVM di frontend.** Komponen hanya menampilkan; ViewModel memegang state & aksi.
4. **Satu sumber kebenaran.** State pulau (prioritas, antrian) diputuskan di Rust; frontend hanya menggambar snapshot.
5. **Satu arah aliran data.** Event → reducer → state → UI → aksi pengguna → command.
6. **Tipe dibagi otomatis.** Tipe Rust di-generate ke TypeScript, jadi tidak ada salah ketik antar layer.
7. **Semua yang penting bisa dites tanpa OS asli** (port di-mock).

## 2. Pola yang Dipakai & Alasannya

| Pola | Dipakai? | Di mana / kenapa |
|---|---|---|
| **MVC** | Sebagian | Tauri *commands* berperan sebagai **Controller** tipis: menerima input UI, memanggil use case, mengembalikan DTO. Tanpa logika bisnis |
| **MVP** | ❌ | Presenter yang memanggil View secara manual tidak cocok dengan UI reaktif React |
| **MVVM** | ✅ Utama di frontend | View (komponen React) ↔ ViewModel (hook + store) ↔ Model (backend Rust via IPC bertipe) |
| **MVI / Redux-style** | ✅ Untuk state pulau | `IslandState` immutable + `IslandAction` + reducer murni, mudah dites & diprediksi |
| **Clean / Hexagonal** | ✅ Utama di backend | `domain` ← `application` ← `infrastructure` / `interface`. OS & AI = adapter yang bisa diganti |
| **Repository** | ✅ | Akses SQLite & setelan di balik trait |
| **Strategy** | ✅ | `AiProvider` (Claude API / Ollama / Claude CLI), `MediaSource` per OS |
| **Observer / Event Bus** | ✅ | Domain event di-broadcast ke frontend & Lumi |
| **State Machine** | ✅ | Pulau (Rust) dan ekspresi Lumi (Rive) |

## 3. Tech Stack (semua gratis)

| Bagian | Pilihan |
|---|---|
| Shell aplikasi | **Tauri 2** |
| Backend | **Rust** (stable), `tokio`, `serde`, `thiserror`, `tracing` |
| Frontend | **React + TypeScript (strict) + Vite** |
| Animasi UI | **Motion** (framer-motion) untuk spring pulau |
| Karakter | **Rive** (`@rive-app/react-canvas`), file `.riv` lokal |
| State frontend | **Zustand** (store per fitur) |
| Tipe Rust → TS | **tauri-specta** (generate binding command & event) |
| Database lokal | **SQLite** via `rusqlite` + `rusqlite_migration` |
| Setelan | `tauri-plugin-store` |
| Rahasia (API Key) | crate `keyring` (Keychain / Credential Manager / Secret Service) |
| HTTP ke AI | `reqwest` (rustls) |
| Plugin Tauri | `global-shortcut`, `autostart`, `single-instance`, `updater`, `shell`, `opener`, `positioner` |
| Ekstensi browser | TypeScript, Manifest V3, WebExtension API (Chrome/Edge/Firefox) |
| Test | `cargo test` + `mockall`; Vitest + Testing Library |
| Lint/format | `rustfmt`, `clippy -D warnings`; ESLint + Prettier |
| Paket JS | `pnpm` workspace |
| CI | GitHub Actions + `tauri-action` (build 3 OS, rilis ke GitHub Releases) |

## 4. Gambaran Besar

```
┌──────────────────────────── FRONTEND (WebView) ─────────────────────────────┐
│  VIEW (React, dumb)        VIEWMODEL (hooks + Zustand)       IPC CLIENT       │
│  IslandShell, MediaCard,  ◄ useIslandVM, useMediaVM,     ◄  bindings.ts       │
│  ReviewCard, LumiCanvas    useReviewVM, useHealthVM          (tauri-specta)   │
└──────────────────────────────────────▲───────────┬──────────────────────────┘
                     events (snapshot) │           │ commands (aksi)
┌──────────────────────────────────────┴───────────▼──────────────────────────┐
│ INTERFACE  (Controller tipis)                                                │
│  commands/*.rs  ·  event_emitter.rs  ·  dto/*.rs  ·  cli.rs (hook entry)     │
├──────────────────────────────────────────────────────────────────────────────┤
│ APPLICATION  (use case / service, orkestrasi)                                │
│  IslandService (pemegang state + timer)  MediaService  ReviewService         │
│  HealthService  SettingsService  MascotService                               │
├──────────────────────────────────────────────────────────────────────────────┤
│ DOMAIN  (Rust murni: tanpa tauri, tanpa OS, tanpa I/O)                        │
│  model/*  ·  island/{state, action, reducer, priority}  ·  events.rs          │
│  ports/: MediaSource, AiProvider, GitRepo, SecretScanner, Clock,             │
│          UsageRepository, SettingsRepository, SecretStore, DisplayInfo,      │
│          BrightnessReader, IdleDetector, BrowserBridge                       │
├──────────────────────────────────────────────────────────────────────────────┤
│ INFRASTRUCTURE  (adapter implementasi port)                                  │
│  media/{windows_smtc, macos, linux_mpris}   ai/{anthropic, ollama, claude_cli}│
│  git/cli_git   storage/{sqlite, store, keyring}   system/{display, idle,     │
│  brightness}   bridge/extension_ws   security/regex_scanner                  │
└──────────────────────────────────────────────────────────────────────────────┘
         ▲ ekstensi browser (WebSocket 127.0.0.1)    ▲ git hook pre-push (CLI → IPC lokal)
```

**Aturan dependensi (wajib):**
- `domain` tidak mengimpor apa pun dari layer lain, `tauri`, atau crate I/O.
- `application` hanya bergantung pada `domain` (memakai trait di `ports`).
- `infrastructure` mengimplementasikan `ports`; tidak memanggil `application`.
- `interface` hanya memanggil `application`; tidak menyentuh `infrastructure` langsung.
- Perakitan (dependency injection) **hanya** di `bootstrap.rs`.
- Frontend: View tidak boleh memanggil `invoke` langsung, harus lewat ViewModel.

## 5. Struktur Repo

```
isla-desk/
├── apps/
│   └── desktop/
│       ├── src-tauri/
│       │   ├── Cargo.toml
│       │   ├── tauri.conf.json
│       │   ├── capabilities/            # izin Tauri per jendela (minimal)
│       │   └── src/
│       │       ├── main.rs              # panggil lib::run()
│       │       ├── lib.rs               # builder Tauri + plugin
│       │       ├── bootstrap.rs         # DI: rakit adapter → service → state
│       │       ├── domain/
│       │       │   ├── model/           # MediaInfo, ReviewReport, Finding, UsageSession, Settings, Mood...
│       │       │   ├── island/          # state.rs, action.rs, reducer.rs, priority.rs
│       │       │   ├── events.rs        # DomainEvent enum
│       │       │   ├── ports/           # trait per kebutuhan (lihat §4)
│       │       │   └── error.rs
│       │       ├── application/
│       │       │   ├── island_service.rs
│       │       │   ├── media_service.rs
│       │       │   ├── review/          # review_service.rs, prompt.rs, patch.rs, redact.rs
│       │       │   ├── health_service.rs
│       │       │   ├── mascot_service.rs   # DomainEvent → Mood
│       │       │   └── settings_service.rs
│       │       ├── infrastructure/
│       │       │   ├── media/{mod.rs, windows_smtc.rs, macos.rs, linux_mpris.rs}
│       │       │   ├── ai/{anthropic.rs, ollama.rs, claude_cli.rs}
│       │       │   ├── git/cli_git.rs
│       │       │   ├── security/regex_scanner.rs
│       │       │   ├── storage/{sqlite.rs, migrations/, usage_repo.rs, settings_store.rs, keyring_store.rs}
│       │       │   ├── system/{display.rs, idle.rs, brightness.rs, fullscreen.rs}
│       │       │   └── bridge/extension_ws.rs
│       │       └── interface/
│       │           ├── commands/{island.rs, media.rs, review.rs, health.rs, settings.rs}
│       │           ├── events.rs        # emit snapshot ke frontend
│       │           ├── window/{island_window.rs, hit_test.rs}
│       │           ├── ipc_socket.rs    # terima koneksi dari git hook
│       │           ├── cli.rs           # `isla hook pre-push`
│       │           └── dto/
│       └── src/                         # FRONTEND
│           ├── main.tsx
│           ├── bindings.ts              # AUTO-GENERATED oleh tauri-specta, jangan edit
│           ├── windows/                 # entry per jendela
│           │   ├── island/IslandWindow.tsx
│           │   ├── settings/SettingsWindow.tsx
│           │   └── review/ReviewWindow.tsx
│           ├── features/
│           │   ├── island/  { view/, viewmodel/useIslandVM.ts, store.ts }
│           │   ├── media/   { view/MediaCompact.tsx, view/MediaExpanded.tsx, viewmodel/ }
│           │   ├── review/  { view/ReviewCard.tsx, view/DiffPanel.tsx, viewmodel/ }
│           │   ├── health/  { view/, viewmodel/ }
│           │   ├── mascot/  { view/LumiCanvas.tsx, viewmodel/useLumiVM.ts }
│           │   ├── onboarding/
│           │   └── settings/
│           ├── shared/
│           │   ├── ipc/         # wrapper bindings + subscribe event
│           │   ├── motion/      # preset spring & transisi
│           │   ├── ui/          # tombol, slider, ikon
│           │   └── i18n/        # id.json, en.json
│           └── assets/lumi.riv
├── extension/                    # "Isla Link" (MV3)
│   ├── manifest.json
│   └── src/{background.ts, content-youtube.ts, pairing.ts}
├── docs/  (prd-desktop.md, architecture-desktop.md, ide-fitur-desktop.md)
├── pnpm-workspace.yaml
└── .github/workflows/{ci.yml, release.yml}
```

## 6. Komponen Kunci

### 6.1 Jendela Pulau (`interface/window`)
- Konfigurasi: `transparent: true`, `decorations: false`, `alwaysOnTop: true`, `skipTaskbar: true`, `resizable: false`, `shadow: false`, `focus: false`.
- **Ukuran jendela tetap** = kanvas maksimum pulau (mis. 760×420 logical px). Jendela tidak di-resize per frame karena akan patah-patah; yang beranimasi adalah elemen di dalamnya.
- **Klik tembus**: default `set_ignore_cursor_events(true)`. Backend membaca posisi kursor (± 30 Hz, hanya saat kursor di sekitar 120 px dari tepi atas) dan membandingkan dengan **hitbox pulau saat ini** (dikirim frontend setiap bentuk berubah). Kursor masuk hitbox → `false`; keluar → `true`.
- **Posisi**: tengah atas monitor tempat kursor berada (`positioner` + `available_monitors`); dihitung ulang saat monitor/skala berubah.
- **macOS notch**: baca `NSScreen.safeAreaInsets` / `auxiliaryTopLeftArea`. Bila ada notch, pulau idle selebar notch dan jendela dinaikkan ke level di atas menu bar (`NSStatusWindowLevel`, via kode `objc2` khusus macOS).
- **Linux**: X11 penuh. Wayland: pakai `gtk-layer-shell` bila compositor mendukung (KDE, Hyprland, Sway); selain itu mode terbatas (jendela biasa tanpa jaminan posisi).
- **Layar penuh**: `system/fullscreen.rs` mendeteksi aplikasi layar penuh di depan → aksi `HideForFullscreen`.

### 6.2 State Machine Pulau (`domain/island`)
```rust
pub enum IslandState {
    Hidden,
    Idle,
    Compact(CompactContent),     // Media, Health, ReviewRunning, Timer
    Expanded(ExpandedContent),   // Media, ReviewResult, HealthAlert, Notice
}
pub enum IslandAction {
    MediaChanged(Option<MediaInfo>), ReviewStarted(ReviewId), ReviewFinished(ReviewReport),
    HealthAlert(HealthAlert), Hover(bool), Clicked, Dismissed, Timeout(Token),
    HideForFullscreen(bool), DoNotDisturb(bool),
}
pub fn reduce(state: &IslandState, action: IslandAction, ctx: &Ctx) -> (IslandState, Vec<Effect>);
```
- **Murni**: tidak ada I/O. Efek samping (mulai timer, kirim event) dikembalikan sebagai `Effect`, lalu dijalankan `IslandService`.
- **Prioritas** (tinggi → rendah): `ReviewResult` (menunggu keputusan) > `ReviewRunning` > `HealthAlert` > `Notice` > `Media` > `Idle`.
- Item berprioritas lebih rendah masuk antrian (maks 5) dan muncul setelah yang lebih tinggi selesai.
- Snapshot dikirim ke frontend lewat event `island://snapshot` setiap state berubah.

### 6.3 Media (`ports::MediaSource`)
```rust
#[async_trait]
pub trait MediaSource: Send + Sync {
    fn subscribe(&self) -> BoxStream<'static, Vec<MediaSession>>;
    async fn control(&self, session: &SessionId, cmd: MediaCommand) -> Result<(), MediaError>;
}
```
- **Windows**: crate `windows`, `GlobalSystemMediaTransportControlsSessionManager` (judul, artis, thumbnail stream, timeline, playback info, kontrol).
- **Linux**: MPRIS lewat D-Bus (`zbus`), sinyal `PropertiesChanged`.
- **macOS**: framework MediaRemote dibatasi untuk aplikasi pihak ketiga di macOS terbaru. Gunakan pendekatan *adapter* (proses pembantu) dan uji di setiap rilis macOS. Bila gagal, fallback ke data dari ekstensi browser.
- Thumbnail diperkecil ≤ 256 px; warna dominan dihitung di backend untuk visualizer.
- Event-driven, **tanpa polling**; progress dihitung dari `position + elapsed × rate`.

### 6.4 Ekstensi Browser "Isla Link" (`ports::BrowserBridge`)
- Aplikasi membuka WebSocket di **`127.0.0.1`** (port acak, disimpan di setelan).
- **Pairing**: pulau menampilkan kode 6 digit → dimasukkan di popup ekstensi → token disimpan kedua sisi. Koneksi tanpa token valid ditolak; `Origin` dicek terhadap ID ekstensi.
- Pesan (JSON bertipe): `SetRate{rate}`, `Search{query}`, `NowPlaying{...}` (fallback macOS).
- `content-youtube.ts`: cari `<video>` aktif → `video.playbackRate = rate`.
- Aplikasi tetap berfungsi penuh tanpa ekstensi (fitur kecepatan disembunyikan).

### 6.5 AI Review saat Push (`application/review`)
**Alur:**
```
git push
  └─ .git/hooks/pre-push  (sh)  →  `isla hook pre-push "$@"` < stdin (daftar ref)
        └─ cli.rs → sambung ke instance berjalan via ipc_socket (named pipe / unix socket)
              └─ ReviewService.run(repo, refs)
                    1. GitRepo.diff(range)               → Diff
                    2. SecretScanner.scan(diff)          → temuan 🔴 lokal
                    3. redact(diff)                      → rahasia disensor
                    4. AiProvider.review(prompt, diff)   → ReviewReport (JSON terstruktur)
                    5. IslandAction::ReviewFinished      → pulau tampil + Lumi
                    6. tunggu keputusan pengguna (timeout dari setelan)
                         ├─ Fix      → AiProvider.fix() → patch → DiffPanel → setuju
                         │             → GitRepo.apply + commit → exit 0 (push lanjut)
                         ├─ Continue → exit 0
                         └─ Cancel   → exit 1 (push dibatalkan)
```
- Bila aplikasi tidak berjalan, hook keluar dengan 0 dan menampilkan pesan di terminal (push tidak pernah terkunci karena Isla).
- **Kontrak AI**: prompt meminta keluaran JSON sesuai skema (`summary`, `findings[{file, line, severity, title, detail, suggestion}]`). Respons divalidasi dengan `serde`; bila tidak valid, coba sekali lagi, lalu tampilkan sebagai teks.
- Batas: ukuran diff maksimal (bisa diatur), file biner & lockfile dilewati, pola pengecualian (`.islaignore`).

```rust
#[async_trait]
pub trait AiProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    async fn review(&self, req: ReviewRequest) -> Result<ReviewReport, AiError>;
    async fn fix(&self, req: FixRequest) -> Result<Patch, AiError>;
    async fn health_check(&self) -> Result<(), AiError>;
}
```
| Adapter | Implementasi |
|---|---|
| `anthropic.rs` | `reqwest` → Messages API, API Key dari `SecretStore`, model bisa dipilih |
| `ollama.rs` | `reqwest` → `http://127.0.0.1:11434`, model lokal pilihan pengguna |
| `claude_cli.rs` | jalankan `claude -p ... --output-format json` milik pengguna (pemakaian pribadi) |

### 6.6 Kesehatan (`application/health_service.rs`)
- `IdleDetector` (input terakhir) → sesi aktif; simpan per menit ke SQLite (`usage_minutes(date, minute, active, brightness)`).
- `BrightnessReader`: Windows (WMI `WmiMonitorBrightness`), macOS (IOKit/DisplayServices), Linux (`/sys/class/backlight`). Bila tidak tersedia (monitor eksternal) → `None`, fitur kecerahan disembunyikan.
- **Aturan** = daftar `HealthRule` (Strategy): `ContinuousUse`, `TwentyTwentyTwenty`, `BrightAtNight`, `LateNight`. Tiap aturan murni: `fn evaluate(&self, snapshot) -> Option<HealthAlert>`.
- Ticker 1 menit (satu-satunya timer berkala di aplikasi).

### 6.7 Lumi (`application/mascot_service.rs` + `features/mascot`)
- Backend memetakan `DomainEvent` → `Mood` (`Idle, Thinking, Happy, Worried, Panic, Sleepy, Sleeping, Listening, Focus`) dan mengirim `mascot://mood`.
- Frontend `useLumiVM` meneruskan mood ke input state machine Rive (`mood` number input + trigger `poke`).
- Desain karakter orisinal; file `lumi.riv` dibuat di editor Rive (paket gratis) dan disimpan lokal.
- Saat aplikasi tersembunyi/opsi "kurangi animasi": Rive dijeda.

### 6.8 Penyimpanan
| Data | Tempat |
|---|---|
| Setelan | `tauri-plugin-store` (JSON di folder data aplikasi) |
| Riwayat pemakaian, riwayat review, clipboard (v1.1) | SQLite `isla.db` + migrasi berversi |
| API Key | `keyring`, **tidak pernah** ditulis ke file/log |
| Token pairing ekstensi | `keyring` |

### 6.9 Frontend MVVM (contoh)
```ts
// features/island/viewmodel/useIslandVM.ts  (VIEWMODEL)
export function useIslandVM() {
  const snapshot = useIslandStore(s => s.snapshot);      // diisi dari event island://snapshot
  return {
    shape: shapeFor(snapshot),                           // lebar, tinggi, radius → untuk Motion
    content: snapshot.content,
    onHover: (h: boolean) => commands.islandHover(h),
    onClick: () => commands.islandClick(),
    onDismiss: () => commands.islandDismiss(),
    reportHitbox: (r: DOMRect) => commands.islandHitbox(toRect(r)),
  };
}

// features/island/view/IslandShell.tsx  (VIEW, tanpa logika)
export function IslandShell() {
  const vm = useIslandVM();
  return (
    <motion.div layout animate={vm.shape} transition={springs.island}
      onHoverStart={() => vm.onHover(true)} onHoverEnd={() => vm.onHover(false)}
      onClick={vm.onClick} onLayoutAnimationComplete={e => vm.reportHitbox(...)}>
      <AnimatePresence mode="popLayout"><ContentFor content={vm.content} /></AnimatePresence>
    </motion.div>
  );
}
```
- Store Zustand per fitur hanya berisi **data dari backend + state UI lokal** (mis. hover).
- Satu file `shared/ipc` yang subscribe semua event dan mengisi store.

## 7. Keamanan & Privasi
- `capabilities/` Tauri: jendela pulau hanya boleh memanggil command pulau/media/mascot; jendela Setelan mendapat sisanya. Tanpa akses `fs`/`shell` umum dari frontend.
- CSP ketat; tanpa memuat skrip remote.
- WebSocket & IPC hanya di loopback, wajib token.
- Rahasia disensor sebelum ke AI; log memakai `tracing` dengan filter yang menyamarkan isi diff.
- Tanpa telemetri.

## 8. Performa
- Tanpa polling kecuali: kursor (hanya di zona atas, 30 Hz) dan ticker kesehatan (1 menit).
- Animasi tak berujung (visualizer, Lumi) hanya saat terlihat; `prefers-reduced-motion` dihormati.
- Event snapshot dikirim hanya saat state berubah (di-debounce 16 ms).
- Thumbnail & gambar dikecilkan di backend.

## 9. Penanganan Error
- Backend: `thiserror` per layer; `anyhow` hanya di `bootstrap`/`main`. **Dilarang `unwrap()`/`expect()`** di luar test.
- Error ke frontend sebagai enum bertipe (`AppError { kind, message_key }`); pesan di-i18n.
- Adapter OS yang gagal **tidak menjatuhkan aplikasi**: fitur terkait dinonaktifkan + status di Setelan.

## 10. Strategi Test

| Layer | Yang dites | Alat |
|---|---|---|
| Domain | Reducer pulau (semua transisi, prioritas, antrian), aturan kesehatan, pemetaan Mood | `cargo test` murni |
| Application | `ReviewService` dengan `AiProvider`/`GitRepo` palsu (sukses, timeout, JSON rusak, fix), `HealthService` dengan `Clock` palsu | `mockall` |
| Infrastructure | `regex_scanner` (set uji rahasia), migrasi SQLite, parser respons Anthropic/Ollama (fixture JSON) | `cargo test` |
| Frontend | ViewModel (store + mock IPC), komponen tiap state pulau | Vitest + Testing Library |
| Manual | Matriks: Win 11, macOS notch & non-notch, Ubuntu X11, KDE Wayland; Chrome/Edge/Firefox | checklist |

## 11. Konvensi Kode
- **Rust**: `rustfmt`, `clippy -D warnings`; modul kecil (< 300 baris); nama trait = kemampuan (`MediaSource`, bukan `IMediaSource`); dokumentasi `///` di setiap item publik `domain`/`application`.
- **TypeScript**: `strict: true`, tanpa `any`; komponen `PascalCase`; hook ViewModel `useXxxVM`; satu komponen per file; tanpa `invoke` di View.
- `bindings.ts` hanya di-generate, jangan diedit manual.
- String UI di `i18n/`, tidak di-hardcode.
- Commit: Conventional Commits (`feat:`, `fix:`, `refactor:`…), kecil per fitur.
- Setiap PR: build 3 OS hijau + test lulus.

## 12. Urutan Kerja untuk Claude Code
1. **Fase 0**: monorepo pnpm, `create-tauri-app` (React + TS), struktur folder §5, tauri-specta, lint/format, CI.
2. **Fase 1**: `domain/island` (reducer + test) → `IslandService` → jendela pulau (transparan, posisi, klik tembus + hitbox, notch macOS) → `IslandShell` + spring.
3. **Fase 2**: port `MediaSource` → adapter Windows → Linux → macOS → `MediaCompact`/`MediaExpanded` + visualizer.
4. **Fase 3**: storage (SQLite, store, keyring) → `SettingsService` → jendela Setelan + Onboarding.
5. **Fase 4**: `GitRepo` + `regex_scanner` + `redact` → `AiProvider` (Anthropic, Ollama, Claude CLI) → `cli.rs` + `ipc_socket` + pemasang hook → `ReviewCard` + `DiffPanel` + alur Perbaiki.
6. **Fase 5**: `HealthService` + aturan → kartu kesehatan & grafik → Lumi (Rive) + `MascotService`.
7. **Fase 6**: ekstensi Isla Link (pairing, kecepatan, cari) → fitur v1.1.
8. **Fase 7**: sambungan Android & fitur P2.

Setiap fase: `cargo test`, `pnpm test`, `clippy`, dan build berhasil sebelum lanjut.
