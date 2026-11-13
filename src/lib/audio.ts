// kotoba/src/lib/audio.ts - Минималистичный Singleton плеер для карточек слов
let player: HTMLAudioElement | null = null;

export function playCardAudio(id: number) {
  if (typeof window === "undefined") return;

  if (!player) {
    player = new Audio();
  }

  // Сбрасываем текущее воспроизведение, если пользователь быстро переключает
  player.pause();
  player.currentTime = 0;
  player.src = `/audio/${id}.mp3`;

  player.play().catch((err) => {
    console.warn(`[Kotoba Audio] Студийный файл /audio/${id}.mp3 пока не найден.`, err);
  });
}
