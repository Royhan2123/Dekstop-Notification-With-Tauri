# PRD — Isla Desk (Dynamic Island untuk Desktop)

> Product Requirements Document: **apa** yang dibuat dan **kenapa**.
> Detail teknis ada di `architecture-desktop.md`. Daftar ide lengkap ada di `ide-fitur-desktop.md`.

## 1. Ringkasan

**Isla Desk** adalah aplikasi desktop (Windows, macOS, Linux) yang menampilkan pulau animasi bergaya Dynamic Island di **tengah atas layar**. Pulau ini menampilkan musik/video yang sedang diputar, hasil review kode AI saat `git push`, pengingat kesehatan pemakaian laptop, dan karakter AI animasi bernama **Lumi**.

- **Teknologi**: Tauri 2 (Rust + React/TypeScript).
- **Biaya pembuat: Rp0.** Tanpa server, tanpa database online, tanpa login Google, tanpa API berbayar milik pembuat.
- **Biaya pengguna**: opsional. Mereka bisa memakai API Key Claude sendiri, atau AI lokal Ollama yang gratis.

## 2. Tujuan & Non-Tujuan

### Tujuan
1. Pulau yang halus, cantik, dan "hidup" di semua OS desktop.
2. Musik/video otomatis tampil tanpa setup, seperti di HP.
3. Asisten developer: review kode otomatis sebelum push, dengan pilihan perbaiki / lanjutkan.
4. Menjaga kesehatan pengguna: jam pakai & kecerahan layar.
5. Kode bersih: Clean Architecture + MVVM, mudah dirawat dan diuji.

### Non-Tujuan
- Server/backend, akun pengguna, sinkronisasi cloud, database online.
- Login Google / YouTube Data API.
- Mengunduh audio/video YouTube (melanggar aturan YouTube).
- Iklan, telemetri, pelacakan.
- Menyediakan kuota AI dari akun pembuat (termasuk berbagi login langganan Claude, yang tidak diizinkan Anthropic).

## 3. Pengguna

| Persona | Kebutuhan utama |
|---|---|
| **Developer** | Review kode sebelum push, notifikasi build/terminal, fokus |
| **Pelajar / pekerja** | Musik sambil kerja, Pomodoro, pengingat istirahat |
| **Pecinta tampilan keren** | Animasi halus, Lumi, personalisasi |

## 4. Fitur

Prioritas: **P0** = MVP, **P1** = v1.1, **P2** = v1.2+.

### 4.1 Pulau Inti — P0
- Jendela transparan, selalu di atas, tanpa bingkai, di **tengah atas** monitor aktif.
- macOS dengan notch: pulau menempel di notch.
- State: **Idle** (kapsul kecil) → **Ringkas** (info satu baris) → **Lebar** (kartu lengkap).
- Animasi spring halus (60 fps); konten berganti dengan fade/scale.
- **Klik tembus**: area di luar pulau tidak menghalangi klik ke aplikasi di bawahnya.
- Hover: sedikit melebar. Klik: buka penuh. Klik kanan: menu cepat.
- Antrian & prioritas bila beberapa hal terjadi bersamaan.
- Sembunyi otomatis saat aplikasi layar penuh (game, presentasi, video).

**Kriteria diterima**: pulau tidak pernah menghalangi klik di luar bentuknya; animasi tidak patah-patah di laptop kelas menengah.

### 4.2 Media Otomatis — P0
- Membaca sesi media dari OS: YouTube di browser, YouTube Music, Spotify, VLC, dll.
- Ringkas: thumbnail kecil + gelombang suara beranimasi (warna mengikuti thumbnail).
- Lebar: thumbnail, judul, channel/artis, progress, sebelumnya / putar-jeda / berikutnya.
- Bila ada beberapa sesi: pilih yang sedang diputar; scroll di pulau untuk berpindah.

**Kriteria diterima**: video YouTube yang diputar di Chrome/Edge/Firefox tampil < 1 detik setelah mulai; putar/jeda bekerja.

### 4.3 Ekstensi Browser "Isla Link" — P1
- Ekstensi kecil (Chrome, Edge, Firefox) yang dipasangkan dengan aplikasi lewat kode pairing.
- **Kecepatan putar 0,25×–2×** dari pulau (slider + preset 0,75× / 1× / 1,25× / 1,5× / 2×).
- **Cari lagu**: ketik di pulau → buka pencarian YouTube di tab baru.
- Tanpa ekstensi, fitur 4.2 tetap jalan; tombol kecepatan disembunyikan.

### 4.4 AI Review Kode saat Push — P0
- Pengguna mengaktifkan per repo ("Aktifkan Isla di repo ini") → aplikasi memasang git hook `pre-push`.
- Saat `git push`:
  1. Pulau melebar: "🔍 Mengecek N file…", Lumi berpikir.
  2. **Pemindai rahasia lokal** dijalankan dulu (tanpa AI): password, token, API key.
  3. Diff dikirim ke **otak AI pilihan pengguna** (rahasia yang terdeteksi disensor dulu).
  4. Hasil: ringkasan + daftar temuan (tingkat: 🔴 kritis / 🟠 penting / 🔵 saran).
  5. Tombol **Perbaiki** · **Lihat detail** · **Tetap push** · **Batal**.
- **Perbaiki**: AI membuat patch → tampil di panel diff → pengguna setujui → dibuat commit baru `fix: perbaikan dari review Isla` → push dilanjutkan.
- Batas waktu: bila pengguna tidak merespons 2 menit, perilaku default bisa diatur (lanjut / batal).
- Bila AI tidak tersedia (offline/kuota habis): beri tahu, push tetap bisa dilanjutkan.

**Otak AI** (dipilih di Setelan):

| Pilihan | Biaya | Catatan |
|---|---|---|
| API Key Claude (milik pengguna) | Dibayar pengguna | Kualitas terbaik |
| Ollama (lokal) | Gratis | Offline, butuh RAM cukup |
| Claude Code CLI | Langganan pengguna sendiri | Hanya untuk pemakaian pribadi di laptop sendiri; tidak dipromosikan sebagai cara login untuk pengguna lain |

**Kriteria diterima**: push dengan password di kode ditahan dan ditandai 🔴; push bersih lolos < 30 detik dengan model API.

### 4.5 Kesehatan Layar — P0
- Hitung **jam aktif** (abaikan saat idle > 3 menit).
- Baca **kecerahan layar** (layar internal laptop).
- Pengingat (bisa diatur/dimatikan):
  - Pemakaian nonstop > 2 jam → saran istirahat.
  - Aturan 20-20-20 (opsional).
  - Kecerahan tinggi di malam hari → saran turunkan.
  - Lewat jam 22.00 → "alarm lembur".
- Ringkasan hari ini di pulau; grafik mingguan di jendela Setelan.

### 4.6 Lumi, Karakter AI — P0
- Animasi Rive dengan state machine. MVP: Idle, Berpikir, Senang, Cemas, Tidur.
- Reaktif terhadap event: review berjalan/lolos/gagal, musik, istirahat, malam.
- P2: ekspresi tambahan (panik, fokus, menguap, joget, skin musiman).

### 4.7 Onboarding & Setelan — P0
- Onboarding 4 langkah: kenalan dengan Lumi → pilih otak AI → izin OS (bila perlu) → mulai otomatis saat login.
- Setelan: posisi & ukuran pulau, monitor, tema, suara, durasi tampil, shortcut, otak AI & API Key, repo yang diawasi, aturan kesehatan, bahasa (ID/EN).

### 4.8 Fitur v1.1 (P1)
Quick Ask AI (`Ctrl+Space`), notifikasi terminal selesai, Pomodoro + mode fokus, HUD volume/kecerahan, animasi charger, rak file, riwayat clipboard.

### 4.9 Fitur v1.2+ (P2)
Sambungan ke aplikasi Android (notifikasi HP, file, cari HP) lewat WiFi lokal; kalender via tautan iCal; GitHub radar; pesan commit AI; indikator kamera/mikrofon; cuaca mini; skin musiman; paket suara; Mode Bebek Karet.

## 5. Kebutuhan Non-Fungsional

| Aspek | Target |
|---|---|
| OS | Windows 10/11, macOS 12+, Linux (X11 penuh; Wayland terbatas, lihat Risiko) |
| CPU saat idle | < 1% |
| RAM | < 150 MB |
| Ukuran installer | < 15 MB (tanpa model Ollama) |
| Waktu mulai | < 2 detik sampai pulau tampil |
| Privasi | Tanpa telemetri. Koneksi internet hanya ke penyedia AI pilihan pengguna & cek update. Semua data di laptop |
| Penyimpanan | SQLite lokal + file setelan; API Key di brankas OS (Keychain / Credential Manager / Secret Service) |
| Aksesibilitas | Bisa dipakai dengan keyboard; opsi kurangi animasi |
| Bahasa | Indonesia (default), Inggris |

## 6. Biaya

| Item | Pembuat | Pengguna |
|---|---|---|
| Tauri, Rust, React, Rive runtime, SQLite | Gratis | — |
| Server / database online | Tidak ada | — |
| Distribusi & auto-update (GitHub Releases) | Gratis | — |
| Ekstensi: Firefox Add-ons & Edge Add-ons | Gratis | — |
| Ekstensi: Chrome Web Store | Opsional, ±USD 5 sekali | — |
| Tanda tangan kode macOS (Apple Developer) | Opsional, USD 99/tahun | — |
| Tanda tangan kode Windows | Opsional, berbayar | — |
| AI | — | API Key sendiri **atau** Ollama gratis |

## 7. Risiko

| Risiko | Mitigasi |
|---|---|
| macOS membatasi akses data "Now Playing" di versi baru | Adapter khusus macOS + uji di setiap versi macOS baru; fallback via ekstensi browser |
| Linux Wayland tidak mengizinkan aplikasi mengatur posisi jendela sendiri | Pakai layer-shell (KDE, Hyprland, Sway); GNOME Wayland: mode terbatas; X11 penuh |
| Windows SmartScreen / Gatekeeper memperingatkan aplikasi tanpa tanda tangan | Panduan instalasi; tanda tangan resmi setelah aplikasi ramai |
| Kode sensitif terkirim ke AI | Pemindai rahasia + sensor sebelum dikirim; opsi Ollama 100% lokal; daftar file/folder yang dikecualikan |
| Git hook tidak jalan di shell tertentu | Hook berupa skrip POSIX `sh` (Git for Windows sudah menyertakan sh) yang memanggil binary Isla |
| Biaya API pengguna membengkak | Batas ukuran diff, estimasi token sebelum kirim, pengaturan model |
| Pulau mengganggu kerja | Klik tembus, sembunyi saat layar penuh, mode Jangan Ganggu |

## 8. Milestone

| Fase | Isi | Hasil |
|---|---|---|
| 0 | Setup monorepo, Tauri 2 + React + TS, lint/format, struktur layer, CI | Build kosong jalan di 3 OS |
| 1 | Jendela pulau (transparan, posisi, klik tembus, notch) + state machine + animasi | Pulau idle ↔ lebar |
| 2 | Media otomatis (Windows dulu, lalu Linux, lalu macOS) | **Demo pertama** |
| 3 | Penyimpanan (SQLite, setelan, brankas API Key) + Setelan + Onboarding | |
| 4 | AI review: git hook, pemindai rahasia, provider Claude API & Ollama, panel diff, perbaiki | **MVP fitur developer** |
| 5 | Kesehatan layar + Lumi (Rive) | **MVP v1.0** |
| 6 | Ekstensi browser (kecepatan, cari) + fitur v1.1 | v1.1 |
| 7 | Sambungan Android & fitur P2 | v1.2+ |

## 9. Ukuran Keberhasilan
- Pulau tampil benar di Windows 11, macOS (notch & tanpa notch), dan Ubuntu X11 tanpa setelan manual.
- 0 crash dalam pemakaian 8 jam.
- Review kode menangkap 100% rahasia di set uji; push bersih tidak tertahan lebih dari 30 detik.
- CPU idle < 1% diukur 10 menit.
