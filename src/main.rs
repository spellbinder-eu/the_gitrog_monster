mod expansion;
mod metacard;

use dotenv::dotenv;
use scryfall::bulk::default_cards;
use scryfall::set::Set;
use sqlx::mysql::MySqlPoolOptions;
use std::collections::HashMap;
use std::error::Error;

async fn get_sets() -> Option<Vec<Set>> {
    let set_list = match Set::all().await {
        Ok(set_list) => set_list,
        Err(_) => return None,
    };

    let sets = set_list.into_inner().collect::<Vec<_>>();

    Some(sets)
}

async fn get_cards() -> Option<impl Iterator<Item = Result<scryfall::Card, scryfall::Error>>> {
    let cards = default_cards().await.ok();

    cards
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let db =
        std::env::var("DATABASE_URL").expect("Expected database connection url in environment");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(db.as_str())
        .await
        .expect("Database connection to be established");

    let mut set_code_to_expansion_id = HashMap::<String, String>::new();

    let opt_sets = get_sets().await;
    if opt_sets.is_some() {
        let sets = opt_sets.unwrap();

        for set in sets {
            let id = match crate::expansion::upsert_expansion(&set, &pool).await {
                Some(code) => code,
                None => String::default(),
            };

            let set_code = set.code.to_string();
            set_code_to_expansion_id.insert(set_code, id);
        }
    }

    let cards = get_cards().await;
    if cards.is_some() {
        for unsafe_card in cards.unwrap() {
            let opt_card = unsafe_card.ok();

            if opt_card.is_some() {
                let card = opt_card.unwrap();

                let set_code = card.set.get().to_owned();
                let expansion_id = match set_code_to_expansion_id.get(&set_code) {
                    Some(id) => id.to_owned(),
                    None => String::default(),
                };

                crate::metacard::upsert_card(&card, &expansion_id, &pool).await;
            }
        }
    }

    Ok(())
}
