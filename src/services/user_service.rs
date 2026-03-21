use sqlx::PgPool;

use crate::{
  dto::CreateUserRequest,
  errors::{RepoError, ServiceError},
  models::User,
  repositories::user_repo,
};

#[tracing::instrument(
  skip(pool, req),
  fields(display_name_len = req.display_name.len(), color_len = req.color.len()),
  err(Debug)
)]
pub async fn create_user(pool: &PgPool, req: CreateUserRequest) -> Result<User, ServiceError> {
  let name = req.display_name.trim();
  let color = req.color.trim();

  if name.is_empty() {
    return Err(ServiceError::Validation(
      "display name must be between 1 and 100 characters".to_string(),
    ));
  }

  if color.is_empty() {
    return Err(ServiceError::Validation(
      "color must be between 1 and 64 characters".to_string(),
    ));
  }

  let user = user_repo::insert(pool, name, color)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))?;

  tracing::info!("user created");

  Ok(user)
}
