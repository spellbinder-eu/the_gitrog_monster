use cuid;
use dotenv::dotenv;
use scryfall::{bulk::default_cards, Card};
use sqlx::{mysql::MySqlPoolOptions, FromRow, MySqlPool};
use std::error::Error;

#[derive(Debug, Default, FromRow)]
struct MetaCard {
    scryfallId: String,
}

async fn create_card(card: &Card, pool: &MySqlPool) -> Result<u64, Box<dyn Error>> {
    let query = "
        INSERT INTO metacard
        (id, scryfallId, cardmarketId, name, scryfallUri, imageUri, reserved, expansionId, collectorsNum, price, foilPrice)
        VALUES
        (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
    ";

    let default_price = "0.00".to_string();

    let id = cuid::cuid2();
    let scryfall_id = &card.id.as_hyphenated().to_string();
    let cardmarket_id = match card.cardmarket_id {
        Some(cardmarket_id) => Some(cardmarket_id.to_string()),
        None => None,
    };
    let name = &card.name;
    let scryfall_uri = &card.scryfall_uri.to_string();
    let image_uri = &card.image_uris["small"].to_string();
    let reserved = &card.reserved;
    let expansion_id = "tz4a98xxat96iws9zmbrgj3a";
    let collector_number = &card.collector_number;
    let price = card.prices.eur.as_ref().unwrap_or(&default_price);
    let foil_price = card.prices.eur_foil.as_ref().unwrap_or(&default_price);

    let query_result = sqlx::query(query)
        .bind(id)
        .bind(scryfall_id)
        .bind(cardmarket_id)
        .bind(name)
        .bind(scryfall_uri)
        .bind(image_uri)
        .bind(reserved)
        .bind(expansion_id)
        .bind(collector_number)
        .bind(price)
        .bind(foil_price)
        .execute(pool)
        .await?;

    Ok(query_result.rows_affected())
}

async fn fetch_card(scryfall_id: &String, pool: &MySqlPool) -> Result<MetaCard, Box<dyn Error>> {
    let query = "SELECT * FROM `metacard` WHERE `scryfallId` = ?";

    let metacard = sqlx::query_as::<_, MetaCard>(query)
        .bind(scryfall_id)
        .fetch_optional(pool)
        .await?;

    Ok(metacard.unwrap_or_default())
}

async fn update_card(card: &Card, pool: &MySqlPool) -> Result<u64, Box<dyn Error>> {
    let query = "UPDATE metacard SET price  = ?, foilPrice = ? WHERE scryfallId = ?";

    let default_price = "0.00".to_string();

    let scryfall_id = &card.id.as_hyphenated().to_string();
    let price = card.prices.eur.as_ref().unwrap_or(&default_price);
    let foil_price = card.prices.eur_foil.as_ref().unwrap_or(&default_price);

    let query_result = sqlx::query(query)
        .bind(price)
        .bind(foil_price)
        .bind(scryfall_id)
        .execute(pool)
        .await?;

    Ok(query_result.rows_affected())
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
        .unwrap();

    let cards = default_cards().await.unwrap();
    for card in cards {
        let safe_card = card.unwrap();
        let scryfall_id = &safe_card.id.as_hyphenated().to_string();

        let meta_card = fetch_card(scryfall_id, &pool).await.unwrap();

        if meta_card.scryfallId.is_empty() {
            create_card(&safe_card, &pool).await.unwrap();
        } else {
            update_card(&safe_card, &pool).await.unwrap();
        }
    }

    Ok(())
}
