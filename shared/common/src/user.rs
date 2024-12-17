use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type UserId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: UserId,
}

impl User {
    async fn validate_token(bearer_token: impl AsRef<str>) -> anyhow::Result<UserId> {
        // TODO: bearer token is JWT, platform user id is in the Claims Subject field
        // probably need to encode the user's platform as well

        // TODO: look up user from their platform user id
        // (or create a new user if they don't exist)

        Ok(Uuid::parse_str(bearer_token.as_ref())?)
    }

    pub async fn read_from_user_id(user_id: UserId) -> anyhow::Result<Self> {
        // TODO: read the user from storage

        Ok(Self { user_id })
    }

    pub async fn read_from_token(bearer_token: impl AsRef<str>) -> anyhow::Result<Self> {
        let user_id = Self::validate_token(bearer_token).await?;

        Self::read_from_user_id(user_id).await
    }
}
