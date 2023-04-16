mod config;
mod model;
mod query;

use crate::scryfall::fetch_cards;
use crate::spellbinder::metacard::query::upsert_card;
use sqlx::PgPool;
use std::collections::HashMap;

pub async fn upsert_cards(set_code_to_expansion_id: &HashMap<String, String>, pool: &PgPool) -> () {
    let opt_cards = fetch_cards().await;
    if opt_cards.is_none() {
        return ();
    }
    let cards = opt_cards.unwrap();

    for res_card in cards {
        let opt_card = res_card.ok();
        if opt_card.is_none() {
            continue;
        }

        let card = opt_card.unwrap();

        if card.digital || card.cardmarket_id.is_none() {
            continue;
        }

        let set_code = card.set.get().to_owned();
        let expansion_id = match set_code_to_expansion_id.get(&set_code) {
            Some(id) => id.to_owned(),
            None => String::default(),
        };

        if expansion_id.is_empty() {
            continue;
        }

        // @todo batch inserts
        upsert_card(&card, &expansion_id, &pool)
            .await
            .unwrap_or_default();
    }
}
