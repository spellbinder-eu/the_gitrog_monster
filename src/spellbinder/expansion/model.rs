use sqlx::FromRow;

#[derive(Debug, Default, FromRow)]
pub struct Expansion {
    pub id: String,
    pub code: String,
}
