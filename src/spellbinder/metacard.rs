use crate::scryfall::fetch_cards;
use sqlx::PgPool;
use std::collections::HashMap;
use cuid;
use scryfall::Card;
use std::error::Error;
use itertools::Itertools;

pub async fn upsert_cards(set_code_to_expansion_id: &HashMap<String, String>, pool: &PgPool) {
    let opt_cards = fetch_cards().await;
    if opt_cards.is_none() {
        return;
    }

    let cards = opt_cards.unwrap();
    
    for batch in &cards.into_iter().chunks(200) {
        let mut card_tuples: Vec<(String, Card)> = Vec::new();
        for res_card in batch {
            let opt_card = res_card.ok();
            if opt_card.is_none() {
                continue;
            }

            let card = opt_card.unwrap();

            if card.cardmarket_id.is_none() {
                continue;
            }

            let set_code = card.set.get().to_owned();
            let expansion_id = match set_code_to_expansion_id.get(&set_code) {
                Some(id) => id.to_owned(),
                None => continue,
            };

            card_tuples.push((expansion_id, card));
        }

        batch_upsert_cards(card_tuples, &pool)
            .await
            .unwrap_or_default();
    }
}

struct FormattedCard {
    pub id: String,
    pub scryfall_id: String,
    pub cardmarket_id: i32,
    pub name: String,
    pub scryfall_uri: String,
    pub reserved: bool,
    pub expansion_id: String,
    pub collectors_num: String,
    pub price: f64,
    pub foil_price: f64,
    pub front_image_uri: String,
    pub back_image_uri: String,
}

impl FormattedCard {
    pub fn new(expansion_id: String, card: &Card) -> Self {
        let id: String = cuid::cuid2();

        let scryfall_id: String = card.id.as_hyphenated().to_string();

        let cardmarket_id: i32 = match card.cardmarket_id {
            Some(cardmarket_id) => cardmarket_id.try_into().unwrap(),
            None => 0,
        };

        let name: String = card.name.to_owned();

        let scryfall_uri: String = card.scryfall_uri.to_string();

        let reserved: bool = card.reserved;

        let collectors_num: String = card.collector_number.to_owned();

        let price: f64 = match &card.prices.eur {
            Some(price_string) => price_string.parse::<f64>().unwrap(),
            None => 0.0,
        };

        let foil_price: f64 = match &card.prices.eur_foil {
            Some(price_string) => price_string.parse::<f64>().unwrap(),
            None => 0.0,
        };

        let images = FormattedCard::extract_images(card);
        let (front_image_uri, back_image_uri) = images;

        Self {
            id,
            scryfall_id,
            cardmarket_id,
            name,
            scryfall_uri,
            reserved,
            expansion_id,
            collectors_num,
            price,
            foil_price,
            front_image_uri,
            back_image_uri
        }
    }

    fn extract_images(card: &Card) -> (String, String) {
        if let Some(card_faces) = &card.card_faces {
            let front_image_uris = match card_faces.first() {
                Some(card_face) => card_face.image_uris.clone().unwrap_or_default(),
                None => HashMap::new(), 
            };

            let front_image_uri = match front_image_uris.get("normal") {
                Some(front_image_uri) => front_image_uri.to_string(),
                None => String::new(),
            };

            let back_image_uris = match card_faces.last() {
                Some(card_face) => card_face.image_uris.clone().unwrap_or_default(),
                None => HashMap::new(),
            };

            let back_image_uri = match back_image_uris.get("normal") {
                Some(back_image_uri) => back_image_uri.to_string(),
                None => String::new(),
            };

            return (front_image_uri, back_image_uri);
        }

        let normal_image_uri = match card.image_uris.get("normal") {
            Some(image_uri) => image_uri.to_string(),
            None => String::new(),
        };

        return (normal_image_uri, String::new());
    }
}

async fn batch_upsert_cards(
    cards: Vec<(String, Card)>,
    pool: &PgPool
) -> Result<(), Box<dyn Error>> {
    // @see https://github.com/launchbadge/sqlx/blob/main/FAQ.md#how-can-i-bind-an-array-to-a-values-clause-how-can-i-do-bulk-inserts
    let mut ids: Vec<String> = Vec::new();
    let mut scryfall_ids: Vec<String> = Vec::new();
    let mut cardmarket_ids: Vec<i32> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    let mut scryfall_uris: Vec<String> = Vec::new();
    let mut reserveds: Vec<bool> = Vec::new();
    let mut expansion_ids: Vec<String> = Vec::new();
    let mut collectors_nums: Vec<String> = Vec::new();
    let mut prices: Vec<f64> = Vec::new();
    let mut foil_prices: Vec<f64> = Vec::new();
    let mut front_image_uris: Vec<String> = Vec::new();
    let mut back_image_uris: Vec<String> = Vec::new();

    for card_tuple in cards {
        let (expansion_id, card) = card_tuple;
        let formatted_card = FormattedCard::new(expansion_id, &card);

        ids.push(formatted_card.id);
        scryfall_ids.push(formatted_card.scryfall_id);
        cardmarket_ids.push(formatted_card.cardmarket_id);
        names.push(formatted_card.name);
        scryfall_uris.push(formatted_card.scryfall_uri);
        reserveds.push(formatted_card.reserved);
        expansion_ids.push(formatted_card.expansion_id);
        collectors_nums.push(formatted_card.collectors_num);
        prices.push(formatted_card.price);
        foil_prices.push(formatted_card.foil_price);
        front_image_uris.push(formatted_card.front_image_uri);
        back_image_uris.push(formatted_card.back_image_uri);
    }

    sqlx::query!(r#"INSERT INTO  "MetaCard"
        ("id", "scryfallId", "cardmarketId", "name", "scryfallUri", "reserved", "expansionId", "collectorsNum", "price", "foilPrice", "frontImageUri", "backImageUri")
        (SELECT * FROM UNNEST($1::text[], $2::text[], $3::int4[], $4::text[], $5::text[], $6::bool[], $7::text[], $8::text[], $9::float8[], $10::float8[], $11::text[], $12::text[]))
        ON CONFLICT ("scryfallId") DO UPDATE SET "price" = EXCLUDED."price", "foilPrice" = EXCLUDED."foilPrice"
        "#, 
        &ids[..], 
        &scryfall_ids[..], 
        &cardmarket_ids[..], 
        &names[..], 
        &scryfall_uris[..], 
        &reserveds[..], 
        &expansion_ids[..], 
        &collectors_nums[..], 
        &prices[..], 
        &foil_prices[..], 
        &front_image_uris[..], 
        &back_image_uris[..]
    )
    .execute(pool)
    .await?;

    Ok(())
}
