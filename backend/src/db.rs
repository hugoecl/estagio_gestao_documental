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

        let admin_password = hash("admin");

        sqlx::query!(
            "INSERT IGNORE INTO users (username, email, password, is_admin) VALUES (?, ?, ?, ?)",
            "admin",
            "admin@jcc.pt",
            &admin_password[..],
            true
        )
        .execute(&pool)
        .await?;

        // let (users_cache, contracts_cache, analytics_cache) = tokio::try_join!(
        //     get_users_cache(&pool),
        //     get_contracts_cache(&pool),
        //     get_analytics_cache(&pool)
        // )?;

        println!("Connected to Database");

        Ok(
            Db { pool },
            // Cache {
            //     users: users_cache,
            //     contracts: contracts_cache,
            //     analytics: analytics_cache,
            // },
        )
    }
}
