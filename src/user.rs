use uuid::Uuid;

use crate::{MCHAError, minecraft::login_start::LoginStart};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct User {
    pub uuid: Uuid,
    pub username: String,
}

impl TryFrom<LoginStart> for User {
    type Error = MCHAError;

    fn try_from(value: LoginStart) -> Result<Self, MCHAError> {
        Ok(Self {
            uuid: value.uuid.unwrap(),
            username: value.name.0,
        })
    }
}
