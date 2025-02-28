use std::sync::atomic::AtomicU32;

use ahash::RandomState;
use papaya::HashMap;
use sqlx::mysql::MySqlPool;

use crate::utils::hashing_utils::hash;

const SCHEMA: &str = include_str!("../sql/schema.sql");

pub struct UserCache {
    pub username: String,
    pub email: String,
    pub password: [u8; 48],
    pub is_admin: bool,
}

pub struct Db {
    pub pool: MySqlPool,
}

pub struct Cache {
    pub users: HashMap<u32, UserCache, RandomState>,
    pub last_user_id: AtomicU32,
}

#[inline(always)]
fn i8_to_bool(i: i8) -> bool {
    i != 0
}

impl Db {
    pub async fn new() -> Result<(Db, Cache), sqlx::Error> {
        let pool = MySqlPool::connect("mysql://root:root@localhost:3306/gestao_documental").await?;

        sqlx::query(SCHEMA).execute(&pool).await?;

        let admin_password = hash("admin");

        sqlx::query!(
            "INSERT IGNORE INTO users (username, email, password, is_admin) VALUES (?, ?, ?, ?)",
            "admin",
            "admin@gmail.com",
            &admin_password[..],
            true
        )
        .execute(&pool)
        .await?;

        let users = sqlx::query!("SELECT * FROM users").fetch_all(&pool).await?;
        let users_length = users.len();

        let users_cache = HashMap::builder()
            .hasher(RandomState::new())
            .capacity(users_length)
            .build();

        let mut last_user_id = 0;

        for (i, user) in users.into_iter().enumerate() {
            users_cache.pin().insert(
                user.id,
                UserCache {
                    username: user.username,
                    email: user.email,
                    password: {
                        let mut password = [0u8; 48];
                        password.copy_from_slice(&user.password[..48]);
                        password
                    },
                    is_admin: i8_to_bool(user.is_admin),
                },
            );
            if i == users_length - 1 {
                last_user_id = user.id;
            }
        }

        println!("Connected to Database");

        Ok((
            Db { pool },
            Cache {
                users: users_cache,
                last_user_id: AtomicU32::new(last_user_id),
            },
        ))
    }
}
