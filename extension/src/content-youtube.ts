// Content script YouTube / YT Music: terapkan kecepatan dari ekstensi ke elemen video.
chrome.runtime.onMessage.addListener((msg: { type?: string; rate?: number }) => {
  if (msg.type !== "set_rate" || typeof msg.rate !== "number") return;
  const video = document.querySelector("video");
  if (video) video.playbackRate = msg.rate;
});
