// kotoba/src/lib/audio.ts - Singleton плеер для локальных MP3 через Blob URL
// Обходит ограничение WebKitGTK (GStreamer), где <audio src="tauri://..."> не стримит кастомную схему в release-сборках.

let player: HTMLAudioElement | null = null;
let latestRequestId = 0;
const blobUrlCache = new Map<number, string>();

async function getAudioBlobUrl(id: number): Promise<string> {
  const cached = blobUrlCache.get(id);
  if (cached) return cached;

  const response = await fetch(`/audio/${id}.mp3`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status} while loading /audio/${id}.mp3`);
  }

  const buffer = await response.arrayBuffer();
  const blob = new Blob([buffer], { type: "audio/mpeg" });
  const blobUrl = URL.createObjectURL(blob);
  blobUrlCache.set(id, blobUrl);
  return blobUrl;
}

export async function playCardAudio(id: number) {
  if (typeof window === "undefined") return;

  const requestId = ++latestRequestId;

  if (!player) {
    player = new Audio();
  }

  player.pause();
  player.currentTime = 0;

  try {
    const blobUrl = await getAudioBlobUrl(id);
    if (requestId !== latestRequestId) return;

    player.src = blobUrl;
    await player.play();
  } catch (err) {
    console.warn(`[Kotoba Audio] Не удалось воспроизвести /audio/${id}.mp3:`, err);
  }
}
