use crate::models::{CreateUser, NewUser, User};
use crate::schema::users;
use crate::schema::users::dsl::*;
use diesel::{
    pg::PgConnection,
    r2d2::{self, ConnectionManager, Pool, PooledConnection},
    ExpressionMethods, QueryDsl, RunQueryDsl,
};
use log::{debug, error, info};
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

// Initialize the database connection pool
pub fn init_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}

// User-related database operations
pub struct UserRepository;

impl UserRepository {
    pub fn find_by_username(conn: &mut DbConnection, user_name: &str) -> Option<User> {
        match users.filter(username.eq(user_name)).first::<User>(conn) {
            Ok(user) => Some(user),
            Err(e) => {
                error!("Error finding user by username: {}", e);
                None
            }
        }
    }

    pub fn create_user(
        conn: &mut DbConnection,
        new_user: &NewUser,
    ) -> Result<User, diesel::result::Error> {
        diesel::insert_into(users::table)
            .values(new_user)
            .get_result::<User>(conn)
    }

    pub fn get_all_users(conn: &mut DbConnection) -> Result<Vec<User>, diesel::result::Error> {
        users.load::<User>(conn)
    }

    pub fn count_users(conn: &mut DbConnection) -> i64 {
        match users.count().get_result::<i64>(conn) {
            Ok(count) => count,
            Err(e) => {
                error!("Error counting users: {}", e);
                0
            }
        }
    }
}
