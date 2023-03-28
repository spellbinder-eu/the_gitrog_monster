mod metacard;

use dotenv::dotenv;
use scryfall::bulk::default_cards;
use sqlx::mysql::MySqlPoolOptions;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let db =
        std::env::var("DATABASE_URL").expect("Expected database connection url in environment");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(db.as_str())
        .await
        .unwrap();

    let cards = default_cards().await.unwrap();
    for card in cards {
        crate::metacard::upsert_card(&card.unwrap(), &pool).await;
    }

    Ok(())
}
