use cuid;
use scryfall::Card;
use sqlx::{FromRow, MySqlPool};
use std::error::Error;

#[derive(Debug, Default, FromRow)]
struct MetaCard {
    scryfallId: String,
}

pub async fn upsert_card(card: &Card, expansion_id: &String, pool: &MySqlPool) -> () {
    let scryfall_id = &card.id.as_hyphenated().to_string();

    let meta_card = fetch_card(scryfall_id, &pool).await.unwrap();

    if meta_card.scryfallId.is_empty() {
        create_card(&card, &expansion_id, &pool).await.unwrap();
    } else {
        update_card(&card, &pool).await.unwrap();
    }
}

async fn fetch_card(scryfall_id: &String, pool: &MySqlPool) -> Result<MetaCard, Box<dyn Error>> {
    let query = "SELECT * FROM `metacard` WHERE `scryfallId` = ?";

    let metacard = sqlx::query_as::<_, MetaCard>(query)
        .bind(scryfall_id)
        .fetch_optional(pool)
        .await?;

    Ok(metacard.unwrap_or_default())
}

async fn create_card(
    card: &Card,
    expansion_id: &String,
    pool: &MySqlPool,
) -> Result<u64, Box<dyn Error>> {
    let query = "
        INSERT INTO metacard
        (id, scryfallId, cardmarketId, name, scryfallUri, imageUri, reserved, expansionId, collectorsNum, price, foilPrice)
        VALUES
        (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
    ";

    let default_price = get_default_price();

    let id = cuid::cuid2();
    let scryfall_id = card.id.as_hyphenated().to_string();
    let cardmarket_id = match card.cardmarket_id {
        Some(cardmarket_id) => Some(cardmarket_id.to_string()),
        None => None,
    };
    let name = &card.name;
    let scryfall_uri = card.scryfall_uri.to_string();
    let image_uri = card.image_uris["small"].to_string();
    let reserved = &card.reserved;
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

async fn update_card(card: &Card, pool: &MySqlPool) -> Result<u64, Box<dyn Error>> {
    let query = "UPDATE metacard SET price  = ?, foilPrice = ? WHERE scryfallId = ?";

    let default_price = get_default_price();

    let scryfall_id = card.id.as_hyphenated().to_string();
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

fn get_default_price() -> String {
    "0.00".to_string()
}
