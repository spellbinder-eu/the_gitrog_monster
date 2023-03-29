use cuid;
use scryfall::Card;
use sqlx::{mysql::MySqlDatabaseError, FromRow, MySqlPool};
use std::error::Error;

#[derive(Debug, Default, FromRow)]
struct MetaCard {
    scryfallId: String,
}

pub async fn upsert_card(
    card: &Card,
    expansion_id: &String,
    pool: &MySqlPool,
) -> Result<(), Box<dyn Error>> {
    let scryfall_id = &card.id.as_hyphenated().to_string();

    let meta_card = match fetch_card(scryfall_id, &pool).await {
        Some(meta_card) => meta_card,
        None => MetaCard::default(),
    };

    if meta_card.scryfallId.is_empty() {
        create_card(&card, &expansion_id, &pool).await?;
    } else {
        update_card(&card, &pool).await?;
    }

    Ok(())
}

async fn fetch_card(scryfall_id: &String, pool: &MySqlPool) -> Option<MetaCard> {
    let query = "SELECT * FROM `metacard` WHERE `scryfallId` = ?";

    let result = sqlx::query_as::<_, MetaCard>(query)
        .bind(scryfall_id)
        .fetch_optional(pool)
        .await;

    match result {
        Ok(option) => option,
        Err(_) => None,
    }
}

async fn create_card(
    card: &Card,
    expansion_id: &String,
    pool: &MySqlPool,
) -> Result<(), Box<dyn Error>> {
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

    let has_small = card.image_uris.contains_key("small");
    let has_normal = card.image_uris.contains_key("normal");
    let has_large = card.image_uris.contains_key("large");

    let image_uri = match true {
        i if has_normal == i => card
            .image_uris
            .get("normal")
            .expect("normal image exists")
            .to_string(),
        i if has_large == i => card
            .image_uris
            .get("large")
            .expect("large image exists")
            .to_string(),
        i if has_small == i => card
            .image_uris
            .get("small")
            .expect("Small image exists")
            .to_string(),
        _ => String::default(),
    };

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
        .await;

    match query_result {
        Ok(result) => result,
        Err(error) => match error {
            MySqlDatabaseError => return Ok(()),
            _ => panic!("Unexpected error while inserting card"),
        },
    };

    Ok(())
}

async fn update_card(card: &Card, pool: &MySqlPool) -> Result<(), Box<dyn Error>> {
    let query = "UPDATE metacard SET price  = ?, foilPrice = ? WHERE scryfallId = ?";

    let default_price = get_default_price();

    let scryfall_id = card.id.as_hyphenated().to_string();
    let price = card.prices.eur.as_ref().unwrap_or(&default_price);
    let foil_price = card.prices.eur_foil.as_ref().unwrap_or(&default_price);

    sqlx::query(query)
        .bind(price)
        .bind(foil_price)
        .bind(scryfall_id)
        .execute(pool)
        .await?;

    Ok(())
}

fn get_default_price() -> String {
    "0.00".to_string()
}
