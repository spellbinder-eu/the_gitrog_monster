use cuid;
use scryfall::Set;
use sqlx::{FromRow, MySqlPool};
use std::error::Error;

#[derive(Debug, Default, FromRow)]
struct Expansion {
    id: String,
}

pub async fn upsert_expansion(set: &Set, pool: &MySqlPool) -> Option<String> {
    let code = &set.id.as_hyphenated().to_string();

    let expansion = fetch_expansion(code, &pool).await.unwrap();

    if expansion.id.is_empty() {
        let result = create_expansion(&set, &pool).await;
        return result.ok();
    }

    let id = expansion.id;

    Some(id)
}

async fn fetch_expansion(code: &String, pool: &MySqlPool) -> Result<Expansion, Box<dyn Error>> {
    let query = "SELECT * FROM `expansion` WHERE `code` = ?";

    let metacard = sqlx::query_as::<_, Expansion>(query)
        .bind(code)
        .fetch_optional(pool)
        .await?;

    Ok(metacard.unwrap_or_default())
}

async fn create_expansion(set: &Set, pool: &MySqlPool) -> Result<String, Box<dyn Error>> {
    let query = "
        INSERT INTO `expansion`
        (`id`, `scryfallId`, `name`, `code`)
        VALUES
        (?, ?, ?, ?);
        SELECT LAST_INSERT_ID();
    ";

    let id = cuid::cuid2();
    let scryfall_id = set.id.to_string();
    let name = &set.name;
    let code = set.code.to_string();

    let query_result = sqlx::query_as::<_, Expansion>(query)
        .bind(id)
        .bind(scryfall_id)
        .bind(name)
        .bind(code)
        .fetch_optional(pool)
        .await?;

    let expansion = match query_result {
        Some(expansion) => expansion,
        None => Expansion::default(),
    };

    let expansion_id = expansion.id;

    Ok(expansion_id)
}
