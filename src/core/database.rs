pub mod user_operations {
    use diesel::{prelude::*, insert_into};
    use diesel::result::Error;
    use crate::models::user::{User, NewUser};
    use crate::models::schema::users::dsl::*;

    use super::sqlite_operations::establish_connection;

    pub fn get_user_by_name(user_name: String) -> Result<Option<User>, Error> {
        let conn = establish_connection();
        let mut resp = users
            .filter(name.eq(user_name))
            .load::<User>(&conn)?;
        Ok(resp.pop())
    }

    pub fn insert_user(user: User) ->  Result<User, Error> {
        let conn = establish_connection();
        let _ = insert_into(users)
            .values(&user)
            .execute(&conn)?;
        Ok(user)
    }
}

pub mod chat_operations {

}

pub mod round_operations {

}

pub mod sqlite_operations {
    use diesel::{SqliteConnection, Connection};
    use std::{env};
    pub(crate) fn establish_connection() -> SqliteConnection {
        SqliteConnection::establish(
            &env::var("DATABASE_URL").expect("No DATABASE_URL in .env")
        ).expect("Error connecting to database!")
    }
}