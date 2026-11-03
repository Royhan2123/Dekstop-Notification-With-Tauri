// Content script YouTube / YT Music: terapkan kecepatan dari ekstensi ke elemen video.
let wanted: number | null = null;

function apply() {
  const video = document.querySelector("video");
  if (video && wanted !== null && video.playbackRate !== wanted) video.playbackRate = wanted;
}

chrome.runtime.onMessage.addListener((msg: { type?: string; rate?: number }) => {
  if (msg.type !== "set_rate" || typeof msg.rate !== "number") return;
  wanted = msg.rate;
  apply();
});

// YouTube mengembalikan kecepatan ke bawaan saat video baru dimuat (mis. video berikutnya);
// event media tidak menggelembung, jadi didengar di fase capture.
document.addEventListener("loadedmetadata", apply, true);
