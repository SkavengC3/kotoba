// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::path::PathBuf;
use std::collections::{VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use rusqlite::Connection;
use tauri::{State, Manager};
use crate::structs::{Card, DeckStats, RecallLevel, Schedule, CardDeck, Streak, DayActivity, UserProgress, AppSettings};
//const SESSION_LIMIT: usize = 10;

mod db;
mod parser;
mod structs;



#[tauri::command]
fn get_current_card(state: State<Mutex<CardDeck>>) -> Option<Card> {
    let deck = state.lock().unwrap();
    deck.peek().cloned()
}

#[tauri::command]
fn review(button: RecallLevel, state: State<Mutex<CardDeck>>) -> Option<Card> {
    let mut deck = state.lock().unwrap();
    deck.review_card(button);
    deck.peek().cloned()
}

#[tauri::command]
fn get_user_progress(state: State<Mutex<CardDeck>>) -> UserProgress {
    let deck = state.lock().unwrap();
    deck.get_user_progress()
}

#[tauri::command]
fn get_app_settings(state: State<Mutex<CardDeck>>) -> AppSettings {
    let deck = state.lock().unwrap();
    deck.get_app_settings()
}

#[tauri::command]
fn save_app_settings(daily_quota: usize, language: String, state: State<Mutex<CardDeck>>) {
    let deck = state.lock().unwrap();
    deck.save_app_settings(daily_quota, language);
}

#[tauri::command]
fn get_random_card(state: State<Mutex<CardDeck>>) -> Option<Card> {
    let deck = state.lock().unwrap();
    deck.get_random_card()
}

#[tauri::command]
fn get_deck_stats(state: State<Mutex<CardDeck>>) -> DeckStats {
    let deck = state.lock().unwrap();
    deck.get_stats()
}

#[tauri::command]
fn get_all_cards(state: State<Mutex<CardDeck>>) -> Vec<Card> {
    let deck = state.lock().unwrap();
    deck.get_all_cards()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            if !data_dir.exists() {
                std::fs::create_dir_all(&data_dir)?;
            }
            app.manage(Mutex::new(CardDeck::new(data_dir.join("kotoba.db"))));
            Ok(())
            })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_current_card, get_all_cards, review, get_random_card, get_deck_stats, get_user_progress, get_app_settings, save_app_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


impl Schedule{
    fn new() -> Self{
        Schedule{
            on_repeat: false,
            interval: 0.0,
            next_recall: 0,
            streak: Streak{
                streak_times: 0.0,
                streak_koef: 1.0
            }
        }
    }

    fn recalculate(&mut self, button: RecallLevel){
        const DAY: f32 = 86400.0;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if self.on_repeat && button != RecallLevel::Repeat{
            self.interval = 1.0;
            self.streak.streak_times = 0.0;
            self.next_recall = now + (self.interval * DAY) as u64;
            self.streak.streak_koef = 1.0;
            self.on_repeat = false;
        } else {
            if button == RecallLevel::Instantly{
                self.interval = 3.0*self.streak.streak_koef;
                self.streak.streak_times += 1.0;
                self.streak.streak_koef *= 1.2;
                self.next_recall = now + (self.interval * DAY) as u64;
            } else if button == RecallLevel::Difficult{
                self.interval = 2.0*self.streak.streak_koef;
                self.streak.streak_times += 0.0;
                self.streak.streak_koef *= 1.0;
                self.next_recall = now + (self.interval * DAY) as u64;
            } else {
                self.interval = 0.0;
                self.streak.streak_times = 0.0;
                self.streak.streak_koef = 0.0;
                self.on_repeat = true;
            }
        }
    }
}

impl CardDeck{
    pub fn new(db_path: PathBuf) -> Self {
        let mut conn = Connection::open(&db_path).expect("Failed to open kotoba database");
        db::init_db(&conn).expect("Failed to initialize database");
        db::seed_is_empty(&mut conn).expect("Failed to seed database");
        let cards = db::load_cards(&conn).expect("Failed to load cards from db");
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let today_done = db::is_today_completed(&conn);
        let reviewed_today = db::get_reviewed_today(&conn);
        let quota: usize = db::get_setting(&conn, "daily_quota", "10")
            .parse()
            .unwrap_or(10);
        let mut session_ids: Vec<u32> = cards
            .iter()
            .filter(|c| c.life_time.on_repeat || (c.life_time.streak.streak_times > 0.0 && c.life_time.next_recall <= now))
            .map(|c| c.id)
            .collect();
        // Новые слова (добавляем только если дневная норма ещё не выполнена)
        if !today_done && reviewed_today < quota {
            let needed_new = quota - reviewed_today;
            let new_cards: Vec<u32> = cards
                .iter()
                .filter(|c| c.life_time.streak.streak_times == 0.0 && !c.life_time.on_repeat)
                .take(needed_new)
                .map(|c| c.id)
                .collect();
            session_ids.extend(new_cards);
        }
        let order = VecDeque::from(session_ids);
        CardDeck {
            conn,
            cards,
            order,
            //todays_card: 0,
        }
    }

    fn peek(&self) -> Option<&Card> {
        let current_id = *self.order.front()?;
        self.cards.iter().find(|c| c.id == current_id)
    }

    fn next_card_id(&mut self) -> Option<u32> {
        self.order.pop_front()
    }

    fn review_card(&mut self, button: RecallLevel){
        if let Some(next_id) = self.next_card_id() {
            if let Some(card) = self.cards.iter_mut().find(|c| c.id == next_id) {
                card.life_time.recalculate(button);
                db::update_schedule(&self.conn, card.id, &card.life_time)
                    .expect("Failed to update schedule");
            }

            if button == RecallLevel::Repeat {
                self.order.push_back(next_id);
            } else {
                let _ = db::log_card_reviewed(&self.conn);
            }

            if self.order.is_empty(){
                let _ = db::mark_today_completed(&self.conn);
            }
        }
    }

    fn get_stats(&self) -> DeckStats {
        let order = self.order.len();
        let cards = self.cards.len();
        let on_repeat = self.cards.iter().filter(|c| c.life_time.on_repeat).count();
        let learned_count = self.cards.iter().filter(|c| c.life_time.streak.streak_times > 0.0).count();
        DeckStats{
            due_today: order,
            total_cards: cards,
            repetition_count: on_repeat,
            learned_count
        }
    }

    fn get_random_card(&self) -> Option<Card> {
        if self.cards.is_empty(){return None}
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let index = nanos as usize % self.cards.len();
        self.cards.get(index).cloned()
    }

    fn get_all_cards(&self) -> Vec<Card> {
        self.cards.clone()
    }

    fn get_app_settings(&self) -> AppSettings {
        let quota: usize = db::get_setting(&self.conn, "daily_quota", "10")
            .parse()
            .unwrap_or(10);
        let language = db::get_setting(&self.conn, "language", "Ru");
        AppSettings { daily_quota: quota, language }
    }

    fn save_app_settings(&self, daily_quota: usize, language: String) {
        let _ = db::set_setting(&self.conn, "daily_quota", &daily_quota.to_string());
        let _ = db::set_setting(&self.conn, "language", &language);
    }

    fn get_user_progress(&self) -> UserProgress {
        let today_completed = db::is_today_completed(&self.conn);

        // Вычисляем дни текущей недели
        let day_labels = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];
        let mut week_days = Vec::new();

        for (i, &label) in day_labels.iter().enumerate() {
            let offset = format!("+{} days", i);
            let date: String = self.conn.query_row(
                "SELECT date('now', 'localtime', 'weekday 0', '-6 days', ?1)",
                rusqlite::params![offset],
                |r| r.get(0),
            ).unwrap_or_default();

            let is_today: bool = self.conn.query_row(
                "SELECT date(?1) = date('now', 'localtime')",
                rusqlite::params![date],
                |r| r.get::<_, i64>(0),
            ).map(|v| v == 1).unwrap_or(false);

            let done: bool = self.conn.query_row(
                "SELECT completed FROM activity_log WHERE date = ?1",
                rusqlite::params![date],
                |r| r.get::<_, i64>(0),
            ).map(|v| v == 1).unwrap_or(false);

            week_days.push(DayActivity {
                day_name: label.to_string(),
                date,
                done,
                is_today,
            });
        }

        // Подсчёт непрерывного стрика
        let mut stmt = self.conn.prepare(
            "SELECT date FROM activity_log WHERE completed = 1 ORDER BY date DESC"
        ).unwrap();
        let completed_dates: Vec<String> = stmt.query_map([], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        let mut streak = 0;
        for (idx, _) in completed_dates.iter().enumerate() {
            // Проверяем непрерывность цепочки
            streak = idx + 1;
        }

        UserProgress {
            streak_days: streak,
            today_completed,
            week_days,
        }
    }
}