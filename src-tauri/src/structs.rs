use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecallLevel{
    Instantly,
    Difficult,
    Repeat,
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
// pub enum Language{
//     Ru,
//     En,
//     Es,
//     It,
//     De,
//     Fr
// }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Streak{
    pub streak_times: f32,
    pub streak_koef: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Schedule {
    pub on_repeat: bool,
    pub interval: f32,
    pub next_recall: u64,
    pub streak: Streak
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeckStats{
    pub due_today: usize,
    pub total_cards: usize,
    pub repetition_count: usize,
    pub learned_count: usize
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Card{
    pub id: u32,
    pub kanji: String,
    pub kana: String,
    pub translation: String,
    pub example_japan: Option<String>,
    pub example_trans: Option<String>,
    pub recall_level: Option<RecallLevel>,
    pub life_time: Schedule
}

pub struct CardDeck{
    pub cards: Vec<Card>,
    //pub todays_card: u32,
    pub order: VecDeque<u32>,
    pub conn: rusqlite::Connection,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DayActivity {
    pub day_name: String,
    pub date: String,     // "2026-11-13"
    pub done: bool,
    pub is_today: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserProgress {
    pub streak_days: usize,
    pub today_completed: bool,
    pub week_days: Vec<DayActivity>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub daily_quota: usize,
    pub language: String,
}