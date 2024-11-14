use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
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
    fn as_notification(&self) -> anyhow::Result<Notification> {
        Ok(Notification {
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
pub struct PlacementRequestV1 {}

impl AsNotification for PlacementRequestV1 {
    #[inline]
    fn get_type(&self) -> NotifType {
        NotifType::PlacementRequestV1
    }
}
