use sqlx::mysql::MySqlPool;

use crate::utils::hashing_utils::hash;

const SCHEMA: &str = include_str!("../sql/schema.sql");

pub struct Db {
    pub pool: MySqlPool,
}

impl Db {
    pub async fn new() -> Result<Db, sqlx::Error> {
        // 🔑 LIGAÇÃO À DB
        let pool = MySqlPool::connect(
            "mysql://root:admin123@localhost:3306/gestao_documental?charset=utf8mb4",
        )
        .await?;

        // ⚠️ IMPORTANTE:
        // O schema já foi corrido manualmente.
        // Se corrermos outra vez dá erro "table already exists"
        // Por isso deixamos comentado.
        //
        // Se um dia precisares recriar tudo, descomenta.
        //
        // sqlx::raw_sql(SCHEMA).execute(&pool).await?;

        // ROLES DEFAULT
        sqlx::query!(
            "INSERT IGNORE INTO roles (name, description, is_admin) VALUES (?, ?, ?), (?, ?, ?)",
            "Admin",
            "Administrador com acesso completo",
            true,
            "Colaborador",
            "Função padrão para novos utilizadores",
            false
        )
        .execute(&pool)
        .await?;

        let admin_role = sqlx::query!("SELECT id FROM roles WHERE is_admin = true LIMIT 1")
            .fetch_one(&pool)
            .await?;

        // USER ADMIN DEFAULT
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
