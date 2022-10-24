use std::io::{Error, ErrorKind};
use uuid::Uuid;
use super::schema::users;

#[derive(Debug, Queryable, Insertable)]
#[table_name = "users"]
pub struct User {
    id: String,
    pub name: String,
    chat_id: String
}

#[derive(Debug)]
pub struct NewUser {
    id: String,
    pub name: String,
    chat_id: String,
    checked: NameCheckedState,
}

#[derive(Debug, PartialEq)]
pub enum NameCheckedState {
    Valid,
    Invalid,
    Unchecked,
}

impl User {
    pub fn from(new_user: NewUser) -> Result<Self, Error> {
        match new_user.checked {
            NameCheckedState::Valid => Ok(User {
                id: new_user.id,
                name: new_user.name,
                chat_id: new_user.chat_id
            }),
            NameCheckedState::Invalid => Err(Error::new(ErrorKind::Other, "Invalid user name")),
            NameCheckedState::Unchecked => Err(Error::new(ErrorKind::Other, "User name not yet checked")),
        }
    }
}

impl NewUser {
    pub fn from(name: String, chat_id: String) -> Self {
        Self { 
            id: Uuid::new_v4().to_string(), 
            name, 
            chat_id, 
            checked: NameCheckedState::Unchecked 
        }
    }

    pub fn validate(&mut self) {
        self.checked = NameCheckedState::Valid;
    }

    pub fn invalidate(&mut self) {
        self.checked = NameCheckedState::Invalid;
    }

    pub fn is_valid(&self) -> bool {
        self.checked == NameCheckedState::Valid
    }
}