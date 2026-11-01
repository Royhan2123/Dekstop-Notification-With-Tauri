# Ide Fitur — Dynamic Island untuk Desktop

> Kumpulan ide sebelum dijadikan `prd.md` + `architecture.md` versi desktop.
> Teknologi: **Tauri** (Windows, macOS, Linux). Target biaya pembuat: **Rp0**.
> Tanpa server, tanpa database online, tanpa login Google. Biaya AI ditanggung pengguna (API Key sendiri) atau gratis lewat Ollama.
> Nama kerja: **Isla Desk** · Karakter AI: **Lumi**

---

## 1. Ide dari kamu (sudah dirapikan)

| # | Fitur | Catatan |
|---|---|---|
| 1 | **Pulau animasi di tengah atas layar** | Selalu di atas, transparan; di MacBook menempel di notch |
| 2 | **Deteksi otomatis musik/video yang sedang diputar** | Seperti di HP: membaca "kartu media" sistem (Windows SMTC, macOS, Linux MPRIS). YouTube di browser, YouTube Music, Spotify, dll. otomatis muncul. **Tanpa API, tanpa login, Rp0** |
| 3 | **Kontrol dari pulau** | Judul, channel, thumbnail, progress, putar/jeda, berikutnya/sebelumnya |
| 4 | **Percepat / perlambat (0,25×–2×)** | Lewat **ekstensi browser kecil buatan kita** yang menerima perintah dari pulau |
| 4b | **Cari lagu dari pulau** | Ketik di pulau, lalu halaman pencarian YouTube terbuka di browser (tanpa API) |
| 5 | **AI agent cek kode saat `git push`** | Tombol **Perbaiki / Lihat detail / Tetap push** |
| 6 | **Pilihan otak AI** | API Key Claude milik pengguna, Ollama (gratis, offline), atau Claude Code pribadi |
| 7 | **Jam pemakaian laptop + kecerahan layar** | Peringatan kalau kelamaan atau terlalu terang |
| 8 | **Karakter AI yang bergerak halus** | Lumi, lihat bagian 3 |

---

## 2. Ide tambahan dari saya

### 🧑‍💻 Untuk Developer
- **Terminal selesai, pulau kabari**: `npm run build` selesai dalam 2m 14s ✅ / gagal ❌ + tombol "Tanya AI kenapa".
- **Pesan commit ajaib**: AI membaca perubahan kode lalu menyarankan pesan commit yang rapi. Tinggal klik.
- **Penjaga rahasia**: sebelum push, deteksi password/API key yang tidak sengaja ikut ter-commit. Lumi langsung panik 😱.
- **Port watcher**: "localhost:3000 sudah jalan" + tombol buka di browser.
- **GitHub radar**: status GitHub Actions, PR baru di-review, issue di-assign ke kamu.
- **Mode Bebek Karet (Rubber Duck)**: ceritakan bug ke Lumi; dia mendengarkan, mengangguk, lalu memberi petunjuk tanpa langsung memberi jawaban.
- **Rangkuman harian**: tiap sore, AI merangkum commit hari ini jadi catatan standup siap tempel.

### 🎵 Musik & Hiburan
- **Visualizer warna sampul**: gelombang suara di pulau mengikuti warna dominan thumbnail lagu.
- **Loop A-B**: ulangi bagian tertentu lagu, cocok untuk belajar alat musik atau lagu.
- **Sleep timer**: musik pelan-pelan mengecil lalu berhenti setelah 30 menit.
- **Mode fokus musik**: satu klik untuk memutar playlist lo-fi sekaligus memulai Pomodoro.
- **Kontrol musik apa saja**: Spotify atau YouTube di browser ikut terkontrol dari pulau lewat media key sistem.

### 💚 Kesehatan
- **Laporan layar mingguan**: grafik jam pakai per hari + rata-rata kecerahan.
- **Aturan 20-20-20**: tiap 20 menit, Lumi menunjuk ke kejauhan, "lihat jauh 20 detik ya" 👀.
- **Napas 1 menit**: pulau mengembang-mengempis pelan sebagai panduan tarik dan buang napas.
- **Alarm lembur**: lewat jam 22.00 masih ngoding? Lumi menguap dan menyarankan berhenti.
- **Pengingat minum & postur**: interval bisa diatur, bisa dimatikan.
- **Hangat otomatis**: saran menurunkan kecerahan/mengaktifkan mode malam saat sudah gelap.

### ⚡ Produktivitas
- **Quick Ask** (`Ctrl + Space`): pulau berubah jadi kolom chat AI.
- **Screenshot → tanya AI**: pilih area layar, tanya "error ini kenapa?"
- **Pomodoro + Lumi**: timer di pulau; selama fokus, notifikasi lain ditahan.
- **Hitung mundur meeting**: dari tautan kalender rahasia (iCal) Google/Outlook, tanpa login atau API. "Meeting 5 menit lagi" + tombol **Join**.
- **Rak file**: seret file ke pulau, simpan sementara, seret keluar ke tempat lain.
- **Riwayat clipboard**: 20 salinan terakhir, cari cepat.
- **Terjemah kilat**: blok teks + shortcut, terjemahan muncul di pulau.
- **Catatan cepat & to-do**: ketik langsung di pulau, centang dari pulau.

### 🖥️ Sistem
- **HUD volume & kecerahan** gaya pulau, menggantikan tampilan bawaan yang membosankan.
- **Animasi charger**: colok charger, petir ⚡ mengalir di pulau + persentase baterai.
- **Baterai earphone Bluetooth** saat tersambung.
- **Indikator kamera/mikrofon aktif** 🟢🟠: privasi ala iPhone, ketahuan aplikasi mana yang sedang memakai.
- **Unduhan selesai**, flashdisk dicolok, WiFi putus/tersambung.
- **Multi-monitor**: pulau ikut pindah ke layar tempat kursor berada.

### 📱 Terhubung dengan Aplikasi Android Kita
- **Notifikasi HP muncul di laptop**, lewat WiFi yang sama, tanpa server.
- **Balas cepat** pesan dari laptop.
- **Baterai HP** tampil di pulau.
- **Cari HP**: klik, HP berbunyi walau mode senyap.
- **Kirim file HP ↔ laptop**: seret ke pulau.

### 🎨 Personalisasi & Seru-seruan
- **Pulau mengikuti waktu**: kilau oranye pagi, biru malam.
- **Cuaca mini**: hujan? Ada tetesan air kecil jatuh di dalam pulau 🌧️.
- **Skin musiman**: Ramadan 🌙, 17 Agustus 🇮🇩, Tahun Baru 🎆.
- **Paket suara**: pilih bunyi notifikasi (lembut, retro 8-bit, kristal).
- **Easter egg**: ketik kode rahasia, Lumi joget.

---

## 3. Karakter AI: **Lumi**

Bola cahaya mungil berwarna ungu-biru yang tinggal di dalam pulau. Bentuknya lembut seperti jeli, dengan dua mata yang ekspresif. Desainnya orisinal, dianimasikan dengan **Rive** supaya gerakannya halus dan ringan.

| Situasi | Ekspresi Lumi |
|---|---|
| Idle | Berkedip pelan, "bernapas" naik-turun |
| AI sedang mengecek kode | Mata berputar 🌀, ada titik-titik "berpikir" |
| Push lolos / tes hijau | Melompat senang, percikan bintang ✨ |
| Ada masalah di kode | Keringat menetes 💧, wajah cemas |
| Ada password bocor | Panik, berkedip merah |
| Musik diputar | Mengangguk mengikuti ketukan 🎧 |
| Mode fokus | Pakai ikat kepala, serius |
| Kelamaan di laptop | Menguap 🥱, menunjuk jam |
| Laptop ditinggal / malam | Tidur, "Zzz" melayang |
| Diklik | Tertawa geli, berguling |

Gratis: Rive punya paket gratis, dan file `.riv` bisa dijalankan offline di Tauri.

---

## 4. Interaksi

| Gerakan | Hasil |
|---|---|
| Arahkan kursor ke pulau | Melebar sedikit, memperlihatkan info ringkas |
| Klik | Buka tampilan penuh |
| Scroll di atas pulau | Atur volume / pindah antar kartu |
| Seret file ke pulau | Masuk rak file |
| Klik kanan | Menu cepat (Fokus, Jeda notifikasi, Setelan) |
| `Ctrl + Space` | Quick Ask AI |

---

## 5. Usulan Prioritas

| Tahap | Isi |
|---|---|
| **MVP** | Pulau + animasi, Lumi (5 ekspresi dasar), deteksi & kontrol media otomatis, AI cek kode saat push, jam pakai + kecerahan |
| **v1.1** | Ekstensi browser (kecepatan 0,25×–2×), cari lagu, Quick Ask, terminal selesai, Pomodoro, HUD volume/kecerahan, animasi charger, rak file |
| **v1.2** | Sambungan ke Android (notifikasi, file, cari HP), kalender iCal, GitHub radar |
| **v2.0** | Skin musiman, cuaca mini, paket suara, laporan mingguan, Mode Bebek Karet |
