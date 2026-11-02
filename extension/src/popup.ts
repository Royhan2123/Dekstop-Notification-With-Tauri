const code = document.querySelector<HTMLInputElement>("#code")!;
const out = document.querySelector<HTMLDivElement>("#status")!;

document.querySelector("#pair")!.addEventListener("click", async () => {
  out.textContent = "Menghubungkan...";
  const res = await chrome.runtime.sendMessage({ type: "pair", code: code.value.trim() });
  out.textContent = res?.ok ? "Terhubung." : "Gagal. Cek kode dan pastikan Isla Desk berjalan.";
});
