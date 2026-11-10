use rusqlite::{params, Connection, Result};
use crate::{Card, Language, Schedule, Streak, Japanese, Translation, Example};
const DEFAULT_DECK_JSON: &str = include_str!("../deck.json");

pub fn init_db(conn: &Connection) -> Result<()>{
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS cards (
            id INTEGER PRIMARY KEY,
            japanese_origin TEXT NOT NULL,
            japanese_kana TEXT NOT NULL,
            translation_origin TEXT NOT NULL,
            example_japanese TEXT,
            example_translation TEXT
        );
        CREATE TABLE IF NOT EXISTS schedules (
            card_id INTEGER PRIMARY KEY REFERENCES cards(id) ON DELETE CASCADE,
            on_repeat INTEGER NOT NULL DEFAULT 0,
            interval REAL NOT NULL DEFAULT 0.0,
            next_recall INTEGER NOT NULL DEFAULT 0,
            streak_times REAL NOT NULL DEFAULT 0.0,
            streak_koef REAL NOT NULL DEFAULT 1.0
        );"#
    )?;
    Ok(())
}

pub fn seed_is_empty(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))?;
    if count == 0 {
        let cards: Vec<Card> = serde_json::from_str(DEFAULT_DECK_JSON)
            .expect("Failed to parse default deck JSON");
        for card in cards {
            let translation_text = card.translation
                .get(&Language::Ru)
                .map(|t| t.origin.as_str())
                .unwrap_or("");
            let first_example = card.examples.first();
            let example_jp = first_example.map(|e| e.japanese.as_str());
            let example_tr = first_example
                .and_then(|e| e.other.get(&Language::Ru))
                .map(|s| s.as_str());
            conn.execute(
                "INSERT INTO cards (id, japanese_origin, japanese_kana, translation_origin, example_japanese, example_translation)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
                params![
                    card.id,
                    card.japanese.origin,
                    card.japanese.kana,
                    translation_text,
                    example_jp,
                    example_tr,
                ],
            )?;

            conn.execute(
                "INSERT INTO schedules (card_id, on_repeat, interval, next_recall, streak_times, streak_koef)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
            card.id,
            if card.life_time.on_repeat { 1 } else { 0 },
            card.life_time.interval,
            card.life_time.next_recall as i64,
            card.life_time.streak.streak_times,
            card.life_time.streak.streak_koef,
        ],
            )?;
        }
    }
    Ok(())
}

pub fn load_cards(conn: &Connection) -> Result<Vec<Card>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            c.id,
            c.japanese_origin,
            c.japanese_kana,
            c.translation_origin,
            c.example_japanese,
            c.example_translation,
            s.on_repeat,
            s.interval,
            s.next_recall,
            s.streak_times,
            s.streak_koef
        FROM cards c
        JOIN schedules s ON (c.id = s.card_id)
        ORDER BY c.id;
        "#
    )?;
    let card_iter = stmt.query_map(params![], |row| {
        let id: u32 = row.get(0)?;
        let jp_origin: String = row.get(1)?;
        let jp_kana: String = row.get(2)?;
        let tr_origin: String = row.get(3)?;
        let ex_jp: Option<String> = row.get(4)?;
        let ex_tr: Option<String> = row.get(5)?;
        let on_repeat_int: i64 = row.get(6)?;
        let interval: f32 = row.get(7)?;
        let next_recall: u64 = row.get(8)?;
        let streak_times: f32 = row.get(9)?;
        let streak_koef: f32 = row.get(10)?;

        let mut translation = std::collections::HashMap::new();
        translation.insert(Language::Ru, Translation {
            origin: tr_origin,
            transcription: String::new(),
        });

        let mut examples = Vec::new();
        if let Some(jp) = ex_jp {
            let mut other = std::collections::HashMap::new();
            if let Some(tr) = ex_tr {
                other.insert(Language::Ru, tr);
            }
            examples.push(Example { japanese: jp, other });
        }

        Ok(Card {
            id,
            japanese: Japanese { origin: jp_origin, kana: jp_kana },
            translation,
            recall_level: None,
            examples,
            life_time: Schedule {
                on_repeat: on_repeat_int != 0,
                interval,
                next_recall,
                streak: Streak { streak_times, streak_koef },
            },
        })

    })?;
    card_iter.collect()
}

pub fn update_schedule(conn: &Connection, card_id: u32, schedule: &Schedule) -> Result<()> {
    conn.execute(
        r#"
        UPDATE schedules
        SET on_repeat = ?1,
            interval = ?2,
            next_recall = ?3,
            streak_times = ?4,
            streak_koef = ?5
        WHERE card_id = ?6"#,
        params![
            if schedule.on_repeat{1} else {0},
            schedule.interval,
            schedule.next_recall as i64,
            schedule.streak.streak_times,
            schedule.streak.streak_koef,
            card_id,
        ],
    )?;
    Ok(())
}