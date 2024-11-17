use serde::{de::DeserializeOwned, Deserialize, Serialize};

use common::user::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub recipient: String,
    pub r#type: NotifType,
    pub message: String,
}

impl Notification {
    pub fn to_message<T: DeserializeOwned>(self) -> anyhow::Result<T> {
        Ok(serde_json::from_str(&self.message)?)
    }
}

pub trait AsNotification: Serialize {
    fn get_type(&self) -> NotifType;

    #[inline]
    fn as_notification(&self, recipient: impl Into<String>) -> anyhow::Result<Notification> {
        Ok(Notification {
            recipient: recipient.into(),
            r#type: self.get_type(),
            message: serde_json::to_string(self)?,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifType {
    PlacementRequestV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementRequestV1 {
    pub player_ids: Vec<UserId>,
}

impl AsNotification for PlacementRequestV1 {
    #[inline]
    fn get_type(&self) -> NotifType {
        NotifType::PlacementRequestV1
    }
}

impl PlacementRequestV1 {
    pub fn new(player_ids: Vec<UserId>) -> Self {
        Self { player_ids }
    }
}
