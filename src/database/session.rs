use std::convert::TryFrom;

use crate::schema::matrix_session;
use diesel::prelude::*;
use matrix_sdk::identifiers::UserId;

#[derive(Queryable)]
pub struct Session {
    pub user_id: String,
    pub access_token: String,
    pub device_id: String,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "matrix_session"]
pub struct NewSession {
    pub user_id: String,
    pub access_token: String,
    pub device_id: String,
}

impl Into<matrix_sdk::Session> for Session {
    fn into(self) -> matrix_sdk::Session {
        matrix_sdk::Session {
            access_token: self.access_token,
            user_id: UserId::try_from(self.user_id).unwrap(),
            device_id: self.device_id,
        }
    }
}

pub fn add_session(
    conn: &SqliteConnection,
    session: matrix_sdk::Session,
) -> Result<usize, diesel::result::Error> {
    use crate::schema::matrix_session;
    use crate::schema::matrix_session::dsl::*;

    let new_session = NewSession {
        user_id: session.user_id.to_string(),
        access_token: session.access_token,
        device_id: session.device_id.into(),
    };

    diesel::insert_into(matrix_session::table)
        .values(&new_session)
        .on_conflict(user_id)
        .do_update()
        .set(&new_session)
        .execute(conn)
}

pub fn get_session(conn: &SqliteConnection) -> Result<Session, diesel::result::Error> {
    use crate::schema::matrix_session::dsl::*;

    matrix_session.first::<Session>(conn)
}
