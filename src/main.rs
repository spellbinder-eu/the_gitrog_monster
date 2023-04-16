mod database;
mod scryfall;
mod spellbinder;

use crate::spellbinder::expansion::upsert_expansions;
use crate::spellbinder::metacard::upsert_cards;
use database::pool::create_pool;
use dotenv::dotenv;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let pool = create_pool().await?;

    let set_code_to_expansion_id = upsert_expansions(&pool).await;

    upsert_cards(&set_code_to_expansion_id, &pool).await;

    Ok(())
}
