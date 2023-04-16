use crate::spellbinder::expansion::model::Expansion;
use cuid;
use scryfall::Set;
use sqlx::query;
use sqlx::query_as;
use sqlx::PgPool;
use std::error::Error;

pub(crate) async fn upsert_expansion(set: &Set, pool: &PgPool) -> Result<String, Box<dyn Error>> {
    let code = set.id.as_hyphenated().to_string();

    let expansion = fetch_expansion(code.to_owned(), &pool).await?;

    if !expansion.id.is_empty() {
        return Ok(expansion.id);
    }

    let id = match create_expansion(&set, &pool).await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Error creating expansion ({}): {}", code, e);
            return Err(e);
        }
    };

    Ok(id)
}

pub(crate) async fn preload_expansions(pool: &PgPool) -> Result<Vec<Expansion>, Box<dyn Error>> {
    let expansions = query_as!(Expansion, r#"SELECT "id", "code" FROM "Expansion""#)
        .fetch_all(pool)
        .await?;

    Ok(expansions)
}

pub(in crate::spellbinder::expansion) async fn fetch_expansion(
    code: String,
    pool: &PgPool,
) -> Result<Expansion, Box<dyn Error>> {
    let expansion = query_as!(
        Expansion,
        r#"SELECT "id", "code" FROM "Expansion" WHERE "code" = $1"#,
        code
    )
    .fetch_optional(pool)
    .await?;

    Ok(expansion.unwrap_or_default())
}

pub(in crate::spellbinder::expansion) async fn create_expansion(
    set: &Set,
    pool: &PgPool,
) -> Result<String, Box<dyn Error>> {
    let id = cuid::cuid2();
    let code = set.code.to_string();

    let query_result = query!(
        r#"
        INSERT INTO "Expansion"
        ("id", "scryfallId", "name", "code")
        VALUES
        ($1, $2, $3, $4);
    "#,
        id,
        set.id.to_string(),
        set.name,
        code
    )
    .execute(pool)
    .await?;

    if query_result.rows_affected() == 0 {
        return Err(format!("Failed to create expansion ({})", code).into());
    }

    Ok(id)
}
