mod database;
mod scryfall;
mod set;
mod card;

use crate::set::upsert_sets;
use crate::card::upsert_cards;
use database::pool::create_pool;
use dotenv::dotenv;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let pool = create_pool().await?;

    let set_code_to_id_map = upsert_sets(&pool).await;

    upsert_cards(&set_code_to_id_map, &pool).await;

    Ok(())
}
