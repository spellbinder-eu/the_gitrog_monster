pub mod model;
pub mod query;

use crate::scryfall::fetch_sets;
use crate::spellbinder::expansion::query::{preload_expansions, upsert_expansion};
use sqlx::PgPool;
use std::collections::HashMap;

pub async fn upsert_expansions(pool: &PgPool) -> HashMap<String, String> {
    let mut set_code_to_expansion_id = HashMap::<String, String>::new();

    let opt_expansions = preload_expansions(&pool).await;
    if let Ok(expansions) = opt_expansions {
        for expansion in expansions {
            set_code_to_expansion_id.insert(expansion.code, expansion.id);
        }
    }

    let opt_sets = fetch_sets().await;
    if opt_sets.is_some() {
        let sets = opt_sets.unwrap();

        for set in sets.iter() {
            let set_code = set.code.to_string();

            if set_code_to_expansion_id.contains_key(&set_code) {
                continue;
            }

            // @todo figure out why id isn't being set
            // @todo batch inserts
            let id = match upsert_expansion(&set, &pool).await {
                Ok(code) => code,
                Err(_) => panic!("Expansion id not returned"),
            };

            set_code_to_expansion_id.insert(set_code, id);
        }
    }

    set_code_to_expansion_id
}
