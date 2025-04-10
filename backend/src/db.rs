use sqlx::mysql::MySqlPool;

use crate::utils::hashing_utils::hash;

const SCHEMA: &str = include_str!("../sql/schema.sql");

pub struct Db {
    pub pool: MySqlPool,
}

impl Db {
    pub async fn new() -> Result<Db, sqlx::Error> {
        let pool = MySqlPool::connect(
            "mariadb://root:root@localhost:3306/gestao_documental?password=root",
        )
        .await?;

        sqlx::raw_sql(SCHEMA).execute(&pool).await?;

        sqlx::query!(
            "INSERT IGNORE INTO roles (name, description, is_admin) VALUES (?, ?, ?)",
            "Admin",
            "Administrador com acesso completo",
            true
        )
        .execute(&pool)
        .await?;

        let admin_role = sqlx::query!("SELECT id FROM roles WHERE is_admin = true LIMIT 1")
            .fetch_one(&pool)
            .await?;

        let admin_password = hash("admin");

        sqlx::query!(
            "INSERT IGNORE INTO users (username, email, password) VALUES (?, ?, ?)",
            "admin",
            "admin@jcc.pt",
            &admin_password[..]
        )
        .execute(&pool)
        .await?;

        let admin_user_id = sqlx::query!("SELECT id FROM users WHERE email = ?", "admin@jcc.pt")
            .fetch_one(&pool)
            .await?
            .id;

        sqlx::query!(
            "INSERT IGNORE INTO user_roles (user_id, role_id) VALUES (?, ?)",
            admin_user_id,
            admin_role.id
        )
        .execute(&pool)
        .await?;

        println!("Connected to Database");

        Ok(Db { pool })
    }
}
