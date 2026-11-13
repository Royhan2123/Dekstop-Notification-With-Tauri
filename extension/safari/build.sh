#!/usr/bin/env bash
# Membangun ekstensi Isla Link untuk Safari (macOS). Butuh Xcode dan pnpm.
#
# Safari hanya menerima ekstensi yang dibungkus aplikasi macOS. Skrip ini mengemas berkas ekstensi
# yang dibutuhkan saja, membuat proyek Xcode lewat safari-web-extension-converter, lalu membangunnya
# dengan tanda tangan lokal ("Sign to Run Locally"). Hasil: safari-build/build/.../Isla Link.app
#
# Memakai hasilnya:
#   1. Safari > Pengaturan > Lanjutan > centang "Tampilkan fitur untuk pengembang web".
#   2. Menu Pengembang > "Izinkan Ekstensi Tidak Bertanda Tangan" (diulang tiap Safari dibuka ulang).
#   3. Buka "Isla Link.app" sekali, lalu Safari > Pengaturan > Ekstensi > aktifkan "Isla Link".
#   4. Izinkan ekstensi di situs media (YouTube, Spotify, ...), lalu pairing seperti di browser lain.
set -euo pipefail
cd "$(dirname "$0")/.."

APP_NAME="Isla Link"
BUNDLE_ID="com.royhan.isladesk.link"
OUT="safari-build"

pnpm build

rm -rf "$OUT"
mkdir -p "$OUT/pack"
cp manifest.json popup.html "$OUT/pack/"
cp -R dist "$OUT/pack/dist"

xcrun safari-web-extension-converter "$OUT/pack" \
  --project-location "$OUT" --app-name "$APP_NAME" --bundle-identifier "$BUNDLE_ID" \
  --macos-only --swift --copy-resources --no-open --no-prompt

# Konverter memberi ID aplikasi yang tak berawalan sama dengan ID ekstensi; Xcode menolaknya.
sed -i '' "s/com\\.royhan\\.isladesk\\.Isla-Link/${BUNDLE_ID}/g" "$OUT/$APP_NAME/$APP_NAME.xcodeproj/project.pbxproj"

xcodebuild -project "$OUT/$APP_NAME/$APP_NAME.xcodeproj" -scheme "$APP_NAME" -configuration Debug \
  -derivedDataPath "$OUT/build" CODE_SIGN_IDENTITY="-" CODE_SIGN_STYLE=Manual DEVELOPMENT_TEAM="" build \
  | grep -E "^error|BUILD (SUCCEEDED|FAILED)"

echo "Aplikasi: $PWD/$OUT/build/Build/Products/Debug/$APP_NAME.app"
