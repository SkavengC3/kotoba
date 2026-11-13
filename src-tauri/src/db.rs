use rusqlite::{params, Connection, Result};
use crate::{Card, Schedule, Streak};
const DEFAULT_TSV: &str = include_str!("../n5_vocab.tsv");

pub fn init_db(conn: &Connection) -> Result<()>{
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS cards (
            id INTEGER PRIMARY KEY,
            kanji TEXT NOT NULL,
            kana TEXT NOT NULL,
            translation TEXT NOT NULL,
            example_japan TEXT,
            example_trans TEXT
        );
        CREATE TABLE IF NOT EXISTS schedules (
            card_id INTEGER PRIMARY KEY REFERENCES cards(id) ON DELETE CASCADE,
            on_repeat INTEGER NOT NULL DEFAULT 0,
            interval REAL NOT NULL DEFAULT 0.0,
            next_recall INTEGER NOT NULL DEFAULT 0,
            streak_times REAL NOT NULL DEFAULT 0.0,
            streak_koef REAL NOT NULL DEFAULT 1.0
        );
        CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS activity_log (
            date TEXT PRIMARY KEY,
            cards_reviewed INTEGER NOT NULL DEFAULT 0,
            completed INTEGER NOT NULL DEFAULT 0
        );

        INSERT OR IGNORE INTO settings (key, value) VALUES ('daily_quota', '10');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('language', 'Ru');"#
    )?;
    Ok(())
}

pub fn seed_is_empty(conn: &mut Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))?;
    if count == 0 {
        let cards = crate::parser::parse_tsv(DEFAULT_TSV, 1);
        insert_card_batch(conn, &cards)?;
    }
    Ok(())
}

pub fn load_cards(conn: &Connection) -> Result<Vec<Card>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            id,
            kanji,
            kana,
            translation,
            example_japan,
            example_trans,
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
        let kanji: String = row.get(1)?;
        let kana: String = row.get(2)?;
        let translation: String = row.get(3)?;
        let example_japan: Option<String> = row.get(4)?;
        let example_trans: Option<String> = row.get(5)?;
        let on_repeat_int: i64 = row.get(6)?;
        let interval: f32 = row.get(7)?;
        let next_recall: u64 = row.get(8)?;
        let streak_times: f32 = row.get(9)?;
        let streak_koef: f32 = row.get(10)?;

        Ok(Card{
            id,
            kanji,
            kana,
            translation,
            example_japan,
            example_trans,
            recall_level: None,
            life_time: Schedule {
                on_repeat: on_repeat_int!=0,
                interval,
                next_recall,
                streak: Streak {streak_times, streak_koef},
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

fn insert_card_batch(conn: &mut Connection, cards: &[Card]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt_cards = tx.prepare(
            "INSERT OR IGNORE INTO cards (id, kanji, kana, translation, example_japan, example_trans)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6);"
        )?;
        let mut stmt_schedules = tx.prepare(
            "INSERT OR IGNORE INTO schedules (card_id, on_repeat, interval, next_recall, streak_times, streak_koef)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6);"
        )?;

        for card in cards {
            stmt_cards.execute(params![
                card.id,
                card.kanji,
                card.kana,
                card.translation,
                card.example_japan,
                card.example_trans,
            ])?;

            stmt_schedules.execute(params![
                card.id,
                if card.life_time.on_repeat { 1 } else { 0 },
                card.life_time.interval,
                card.life_time.next_recall as i64,
                card.life_time.streak.streak_times,
                card.life_time.streak.streak_koef,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str, default: &str) -> String {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get(0),
    ).unwrap_or_else(|_| default.to_string())
}

pub fn set_setting(conn: &Connection, key: &str, val: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, val],
    )?;
    Ok(())
}

pub fn log_card_reviewed(conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT INTO activity_log (date, cards_reviewed, completed)
         VALUES (date('now', 'localtime'), 1, 0)
         ON CONFLICT(date) DO UPDATE SET cards_reviewed = cards_reviewed + 1",
        [],
    )?;
    Ok(())
}

pub fn mark_today_completed(conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT INTO activity_log (date, cards_reviewed, completed)
         VALUES (date('now', 'localtime'), 0, 1)
         ON CONFLICT(date) DO UPDATE SET completed = 1",
        [],
    )?;
    Ok(())
}

pub fn is_today_completed(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT completed FROM activity_log WHERE date = date('now', 'localtime')",
        [],
        |row| row.get::<_, i64>(0),
    ).map(|c| c == 1).unwrap_or(false)
}

pub fn get_reviewed_today(conn: &Connection) -> usize {
    conn.query_row(
        "SELECT cards_reviewed FROM activity_log WHERE date = date('now', 'localtime')",
        [],
        |row| row.get::<_, usize>(0),
    ).unwrap_or(0)
}