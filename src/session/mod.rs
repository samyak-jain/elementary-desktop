use std::io;

use matrix_sdk::identifiers::{DeviceId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Session {
    pub access_token: String,
    pub user_id: UserId,
    pub device_id: Box<DeviceId>,
    pub homeserver: String,
}

impl From<Session> for matrix_sdk::Session {
    fn from(s: Session) -> Self {
        Self {
            access_token: s.access_token,
            user_id: s.user_id,
            device_id: s.device_id,
        }
    }
}

fn session_path() -> std::path::PathBuf {
    std::path::PathBuf::from("./data/config/session.toml")
}

pub fn write_session(session: &Session) -> Result<(), io::Error> {
    let serialized = toml::to_string(&session)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
    std::fs::write(session_path(), serialized)?;
    Ok(())
}

pub fn get_session() -> Result<Option<Session>, io::Error> {
    let path = session_path();
    if !path.is_file() {
        return Ok(None);
    }
    let session: Session = toml::from_slice(&std::fs::read(path)?)?;
    Ok(Some(session))
}
