import { PORTS, parseCommand, pickTab, searchUrl } from "./protocol.js";

const TOKEN_KEY = "token";
const MEDIA_TABS = ["*://www.youtube.com/*", "*://music.youtube.com/*"];
const ATTEMPT_TIMEOUT_MS = 3000;

let socket: WebSocket | null = null;
let busy = false;

type Reply = { type?: string; token?: string };

/** Buka koneksi ke satu port dan kirim pesan awal; selesai saat aplikasi membalas ready/paired. */
function attempt(port: number, first: object): Promise<{ ws: WebSocket; reply: Reply } | null> {
  return new Promise((resolve) => {
    const ws = new WebSocket(`ws://127.0.0.1:${port}`);
    const fail = () => {
      clearTimeout(timer);
      ws.close();
      resolve(null);
    };
    const timer = setTimeout(fail, ATTEMPT_TIMEOUT_MS);
    ws.onopen = () => ws.send(JSON.stringify(first));
    ws.onerror = fail;
    ws.onmessage = (e) => {
      clearTimeout(timer);
      let reply: Reply = {};
      try {
        reply = JSON.parse(String(e.data));
      } catch {
        // balasan rusak dianggap gagal
      }
      if (reply.type === "ready" || reply.type === "paired") resolve({ ws, reply });
      else fail();
    };
  });
}

async function connectWith(first: object): Promise<Reply | null> {
  for (const port of PORTS) {
    const hit = await attempt(port, first);
    if (hit) {
      attach(hit.ws);
      return hit.reply;
    }
  }
  return null;
}

function attach(ws: WebSocket) {
  socket?.close();
  socket = ws;
  // Ping berkala menjaga service worker MV3 tetap hidup dan memberi tahu kalau koneksi putus.
  const ping = setInterval(() => ws.send(JSON.stringify({ type: "ping" })), 20_000);
  ws.onmessage = (e) => void handle(String(e.data));
  ws.onclose = () => {
    clearInterval(ping);
    if (socket === ws) socket = null;
  };
}

async function handle(raw: string) {
  const cmd = parseCommand(raw);
  if (!cmd) return;
  if (cmd.type === "search") {
    await chrome.tabs.create({ url: searchUrl(cmd.query) });
    return;
  }
  const tab = pickTab(await chrome.tabs.query({ url: MEDIA_TABS }));
  if (tab?.id !== undefined) await chrome.tabs.sendMessage(tab.id, cmd);
}

/** Sambungkan ulang memakai token tersimpan; dipanggil saat start dan tiap alarm. */
async function reconnect() {
  if (busy || socket) return;
  busy = true;
  try {
    const stored = await chrome.storage.local.get(TOKEN_KEY);
    const token = stored[TOKEN_KEY];
    if (typeof token === "string") await connectWith({ type: "hello", token });
  } finally {
    busy = false;
  }
}

chrome.runtime.onMessage.addListener((msg: { type?: string; code?: string }, _sender, reply) => {
  if (msg.type !== "pair") return;
  void (async () => {
    const res = await connectWith({ type: "pair", code: msg.code ?? "" });
    if (res?.token) await chrome.storage.local.set({ [TOKEN_KEY]: res.token });
    reply({ ok: Boolean(res?.token) });
  })();
  return true; // balasan asinkron
});

chrome.alarms.create("reconnect", { periodInMinutes: 0.5 });
chrome.alarms.onAlarm.addListener(() => void reconnect());
chrome.runtime.onStartup.addListener(() => void reconnect());
chrome.runtime.onInstalled.addListener(() => void reconnect());
void reconnect();
