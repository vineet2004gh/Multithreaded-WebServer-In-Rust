use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Queryable, Identifiable, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub role: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}
