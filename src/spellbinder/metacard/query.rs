use crate::spellbinder::metacard::config::get_default_price;
use cuid;
use scryfall::Card;
use sqlx::{query, query_as, PgPool};
use std::error::Error;

pub(in crate::spellbinder::metacard) async fn upsert_card(
    card: &Card,
    expansion_id: &String,
    pool: &PgPool,
) -> Result<(), Box<dyn Error>> {
    let scryfall_id = &card.id.as_hyphenated().to_string();

    let card_exists = check_card_exists(scryfall_id, &pool).await?;

    if card_exists {
        update_card(&card, &pool).await?;
    } else {
        create_card(&card, &expansion_id, &pool).await?;
    }

    Ok(())
}

pub(in crate::spellbinder::metacard) async fn check_card_exists(
    scryfall_id: &String,
    pool: &PgPool,
) -> Result<bool, Box<dyn Error>> {
    let result = query!(
        r#"SELECT EXISTS(SELECT 1 FROM "MetaCard" WHERE "scryfallId" = $1) as "exists""#,
        scryfall_id
    )
    .fetch_one(pool)
    .await?;

    let exists = result.exists.unwrap();

    Ok(exists)
}

pub(in crate::spellbinder::metacard) async fn create_card(
    card: &Card,
    expansion_id: &String,
    pool: &PgPool,
) -> Result<(), Box<dyn Error>> {
    let id = cuid::cuid2();

    let scryfall_id = card.id.as_hyphenated().to_string();

    let cardmarket_id: Option<i32> = match card.cardmarket_id {
        Some(cardmarket_id) => Some(cardmarket_id.try_into().unwrap()),
        None => None,
    };

    let name = &card.name;

    let scryfall_uri = card.scryfall_uri.to_string();

    let image_uris = &card.image_uris;
    let image_uri = match true {
        i if image_uris.contains_key("normal") == i => card
            .image_uris
            .get("normal")
            .expect("normal image exists")
            .to_string(),
        i if image_uris.contains_key("large") == i => card
            .image_uris
            .get("large")
            .expect("large image exists")
            .to_string(),
        i if image_uris.contains_key("small") == i => card
            .image_uris
            .get("small")
            .expect("Small image exists")
            .to_string(),
        _ => String::default(),
    };

    let reserved = &card.reserved;

    let collector_number = &card.collector_number;

    let default_price = get_default_price();

    let price = match &card.prices.eur {
        Some(price_string) => price_string.parse::<f64>().unwrap(),
        None => default_price.parse::<f64>().unwrap(),
    };

    let foil_price = match &card.prices.eur_foil {
        Some(price_string) => price_string.parse::<f64>().unwrap(),
        None => default_price.parse::<f64>().unwrap(),
    };

    query_as!(
        MetaCard,
        r#"
        INSERT INTO "MetaCard"
        ("id", "scryfallId", "cardmarketId", "name", "scryfallUri", "imageUri", "reserved", "expansionId", "collectorsNum", "price", "foilPrice")
        VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);
    "#,
        id,
        scryfall_id,
        cardmarket_id,
        name,
        scryfall_uri,
        image_uri,
        reserved,
        expansion_id,
        collector_number,
        price,
        foil_price,
    )
    .fetch_one(pool)
    .await?;

    Ok(())
}

pub(in crate::spellbinder::metacard) async fn update_card(
    card: &Card,
    pool: &PgPool,
) -> Result<(), Box<dyn Error>> {
    let query = r#"UPDATE "MetaCard" SET "price" = $1, "foilPrice" = $2 WHERE "scryfallId" = $3"#;

    let scryfall_id = card.id.as_hyphenated().to_string();

    let default_price = get_default_price();

    let price = match &card.prices.eur {
        Some(price_string) => price_string.parse::<f64>().unwrap(),
        None => default_price.parse::<f64>().unwrap(),
    };

    let foil_price = match &card.prices.eur_foil {
        Some(price_string) => price_string.parse::<f64>().unwrap(),
        None => default_price.parse::<f64>().unwrap(),
    };

    sqlx::query(query)
        .bind(price)
        .bind(foil_price)
        .bind(scryfall_id)
        .execute(pool)
        .await?;

    Ok(())
}
