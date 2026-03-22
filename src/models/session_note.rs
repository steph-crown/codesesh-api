use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionNote {
  pub id: Uuid,
  pub session_id: Uuid,
  pub user_id: Uuid,
  pub content: String,
  pub created_at: OffsetDateTime,
  pub updated_at: OffsetDateTime,
}
