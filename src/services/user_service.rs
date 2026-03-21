use sqlx::PgPool;

use crate::{
  dto::CreateUserRequest,
  errors::{RepoError, ServiceError},
  models::User,
  repositories::user_repo,
};

pub async fn create_user(pool: &PgPool, req: CreateUserRequest) -> Result<User, ServiceError> {
  let name = req.display_name.trim();

  if name.is_empty() {
    return Err(ServiceError::Validation(
      "display name must be between 1 and 100 characters".to_string(),
    ));
  }

  user_repo::insert(pool, name)
    .await
    .map_err(|e| ServiceError::Repo(RepoError::Database(e)))
}
