#!/usr/bin/env python3
"""
Скрипт загрузки эталонного японского аудио для слов JLPT N5.
Читает n5_vocab.tsv и генерирует чистые студийные MP3 файлы в static/audio/{id}.mp3
Использует нативный токийский голос ja-JP-NanamiNeural через edge-tts.
"""

import sys
import os
import asyncio
from pathlib import Path

# Пути
SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_DIR = SCRIPT_DIR.parent
TSV_PATH = PROJECT_DIR / "src-tauri" / "n5_vocab.tsv"
OUTPUT_DIR = PROJECT_DIR / "static" / "audio"

try:
    import edge_tts
except ImportError:
    print("[!] Библиотека edge-tts не установлена.")
    print("    Установите её командой: pip install edge-tts")
    sys.exit(1)

async def generate_audio(word_id: int, kanji: str, kana: str):
    target_file = OUTPUT_DIR / f"{word_id}.mp3"
    if target_file.exists():
        print(f"[{word_id:02d}/60] Уже существует: {kanji} ({kana}) -> {target_file.name}")
        return

    # Для японского озвучивания подаем кану для идеального чтения кандзи
    text_to_speak = kana if kana else kanji
    voice = "ja-JP-NanamiNeural"  # Эталонный токийский женский голос (Tokyo Standard)
    
    communicate = edge_tts.Communicate(text=text_to_speak, voice=voice, rate="-5%")
    await communicate.save(str(target_file))
    print(f"[{word_id:02d}/60] Сохранено: {kanji} ({kana}) -> {target_file.name}")

async def main():
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    
    if not TSV_PATH.exists():
        print(f"[-] Файл словаря {TSV_PATH} не найден!")
        sys.exit(1)

    print(f"[*] Чтение словаря из {TSV_PATH}...")
    words = []
    with open(TSV_PATH, "r", encoding="utf-8") as f:
        current_id = 1
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            parts = line.split("\t")
            if len(parts) >= 2:
                kanji = parts[0].strip()
                kana = parts[1].strip()
                words.append((current_id, kanji, kana))
                current_id += 1

    print(f"[*] Найдено слов: {len(words)}. Запуск генерации аудиопака...")
    
    for word_id, kanji, kana in words:
        await generate_audio(word_id, kanji, kana)
        await asyncio.sleep(0.05)  # Небольшая пауза между запросами

    print("\n[+] Успешно! Все аудиофайлы готовы в папке static/audio/")

if __name__ == "__main__":
    asyncio.run(main())
