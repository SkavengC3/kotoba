use crate::{Card, Schedule};

pub fn parse_tsv(content: &str, start_id: u32) -> Vec<Card> {
    let mut current_id = start_id;
    let mut cards = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        } else {
            let parts: Vec<&str> = trimmed.split('\t').collect();
            if parts.len() < 3 {
                continue;
            } else {
                cards.push(Card{
                    id: current_id,
                    kanji: parts[0].to_string(),
                    kana: parts[1].to_string(),
                    translation: parts[2].to_string(),
                    example_japan: parts.get(3).map(|s| s.to_string()),
                    example_trans: parts.get(4).map(|s| s.to_string()),
                    recall_level: None,
                    life_time: Schedule::new()
                });
                current_id += 1;
            }

        }

    }
    cards
}