use std::cmp::PartialEq;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use tauri::State;

mod db;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecallLevel{
    Instantly,
    Difficult,
    Repeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Language{
    Ru,
    En,
    Es,
    It,
    De,
    Fr
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Japanese{
    origin: String,
    kana: String
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Translation{
    origin: String,
    transcription: String
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Example{
    japanese: String,
    other: HashMap<Language, String>
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Streak{
    streak_times: f32,
    streak_koef: f32, //Умножение таймера на коэфициент в случае стрика
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Schedule {
    on_repeat: bool, //Если была нажата кнопка Repeat -> true, пока не нажмется другая
    interval: f32,
    next_recall: u64,
    streak: Streak
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Card{
    id: u32,
    japanese: Japanese,
    translation: HashMap<Language, Translation>,
    recall_level: Option<RecallLevel>,
    examples: Vec<Example>,
    life_time: Schedule
}

pub struct CardDeck{
    cards: Vec<Card>,
    todays_card: u32,//Функционал будет позже. Лимит выбирает сам пользователь
    lang: Language,
    order: VecDeque<u32>,
    conn: rusqlite::Connection,
}

#[tauri::command]
fn get_current_card(state: State<Mutex<CardDeck>>) -> Option<Card> {
    let deck = state.lock().unwrap();
    let result = deck.peek().cloned();
    return result;
}

#[tauri::command]
fn review(button: RecallLevel, state: State<Mutex<CardDeck>>) -> Option<Card> {
    let mut deck = state.lock().unwrap();
    deck.review_card(button);
    deck.peek().cloned()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(CardDeck::new()))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_current_card, review])
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
    pub const DEFAULT_DECK_JSON: &str = include_str!("../deck.json");

    fn new() -> Self{
        let conn = rusqlite::Connection::open("kotoba.db")
            .expect("Failed to open kotoba database");
        db::init_db(&conn).expect("Failed to initialize database");
        db::seed_is_empty(&conn).expect("Failed to seed database");
        let cards = db::load_cards(&conn).expect("Failed to load cards from db");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let order: VecDeque<u32> = cards
            .iter()
            .filter(|c| c.life_time.next_recall <= now || c.life_time.on_repeat)
            .map(|c| c.id)
            .collect();

        CardDeck{
            conn,
            cards,
            order,
            todays_card: 0,
            lang: Language::Ru,
        }
    }

    fn peek(&self) -> Option<&Card> {
        let current_id = *self.order.front()?;
        self.cards.iter().find(|c| c.id == current_id)
    }

    fn next_card_id(&mut self) -> Option<u32> {
        let next_id = self.order.pop_front();
        return next_id;
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
            }
        }
    }
}
