use std::io;

use matrix_sdk::Session;

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
