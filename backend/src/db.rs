use sqlx::mysql::MySqlPool;

use crate::utils::hashing_utils::hash;

const SCHEMA: &str = include_str!("../sql/schema.sql");

pub struct Db {
    pub pool: MySqlPool,
}

impl Db {
    pub async fn new() -> Result<Db, sqlx::Error> {
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

        println!("Connected to Database");

        Ok(Db { pool })
    }
}
