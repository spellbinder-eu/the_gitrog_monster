use cuid;
use scryfall::Set;
use sqlx::{FromRow, PgPool};
use std::error::Error;

#[derive(Debug, Default, FromRow)]
pub struct Expansion {
    pub id: String,
    pub code: String,
}

pub async fn upsert_expansion(set: &Set, pool: &PgPool) -> Result<String, Box<dyn Error>> {
    let code = set.id.as_hyphenated().to_string();

    let expansion = fetch_expansion(code, &pool).await.unwrap();

    if expansion.id.is_empty() {
        let result = create_expansion(&set, &pool).await;
        return result;
    }

    Ok(expansion.id)
}

pub async fn preload_expansions(pool: &PgPool) -> Result<Vec<Expansion>, Box<dyn Error>> {
    let query = "SELECT `id`, `code` FROM `expansion`";

    let expansions = sqlx::query_as::<_, Expansion>(query)
        .fetch_all(pool)
        .await?;

    Ok(expansions)
}

async fn fetch_expansion(code: String, pool: &PgPool) -> Result<Expansion, Box<dyn Error>> {
    let query = "SELECT `id`, `code` FROM `expansion` WHERE `code` = ?";

    let expansion = sqlx::query_as::<_, Expansion>(query)
        .bind(code)
        .fetch_optional(pool)
        .await?;

    Ok(expansion.unwrap_or_default())
}

async fn create_expansion(set: &Set, pool: &PgPool) -> Result<String, Box<dyn Error>> {
    let query = "
        INSERT INTO `expansion`
        (`id`, `scryfallId`, `name`, `code`)
        VALUES
        (?, ?, ?, ?);
    ";

    let id = cuid::cuid2();
    let scryfall_id = set.id.to_string();
    let name = &set.name;
    let code = set.code.to_string();

    sqlx::query(query)
        .bind(&id)
        .bind(scryfall_id)
        .bind(name)
        .bind(code)
        .execute(pool)
        .await?;

    let expansion_id = id;

    Ok(expansion_id)
}
