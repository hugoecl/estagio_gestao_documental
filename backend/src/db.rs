use std::sync::atomic::AtomicU32;

use ahash::RandomState;
use papaya::HashMap;
use sqlx::mysql::MySqlPool;

use crate::models::contract;
use crate::utils::hashing_utils::hash;

const SCHEMA: &str = include_str!("../sql/schema.sql");

pub struct UserCache {
    pub username: String,
    pub email: String,
    pub password: [u8; 48],
    pub is_admin: bool,
}

pub struct ContractFilesCache {
    pub path: String,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
}

pub struct ContractCache {
    pub contract_number: u32,
    pub date: chrono::NaiveDate,
    pub date_start: chrono::NaiveDate,
    pub date_end: chrono::NaiveDate,
    pub description: String,
    pub location: contract::Location,
    pub service: contract::Service,
    pub status: contract::Status,
    pub supplier: String,
    pub type_of_contract: contract::Type,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub files: HashMap<u32, ContractFilesCache, RandomState>,
}

pub struct Db {
    pub pool: MySqlPool,
}

pub struct Cache {
    pub users: HashMap<u32, UserCache, RandomState>,
    pub last_user_id: AtomicU32,
    pub contracts: HashMap<u32, ContractCache, RandomState>,
    pub last_contract_id: AtomicU32,
    pub last_contract_file_id: AtomicU32,
}

#[inline(always)]
fn i8_to_bool(i: i8) -> bool {
    i != 0
}

async fn get_users_cache(
    pool: &sqlx::Pool<sqlx::MySql>,
) -> Result<(HashMap<u32, UserCache, RandomState>, u32), sqlx::Error> {
    let users = sqlx::query!("SELECT * FROM users").fetch_all(pool).await?;
    let users_length = users.len();

    let users_cache = HashMap::builder()
        .hasher(RandomState::new())
        .capacity(users_length)
        .build();

    let pinned_users_cache = users_cache.pin();
    let last_user_id = if let Some(user) = users.last() {
        user.id
    } else {
        0
    };

    for user in users.into_iter() {
        pinned_users_cache.insert(
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
    }
    drop(pinned_users_cache);
    Ok((users_cache, last_user_id))
}

async fn get_contracts_cache(
    pool: &sqlx::Pool<sqlx::MySql>,
) -> Result<(HashMap<u32, ContractCache, RandomState>, u32, u32), sqlx::Error> {
    let contracts = sqlx::query!("SELECT * FROM contracts")
        .fetch_all(pool)
        .await?;

    let contracts_length = contracts.len();

    let contracts_cache = HashMap::builder()
        .hasher(RandomState::new())
        .capacity(contracts_length)
        .build();
    let pinned_contracts_cache = contracts_cache.pin();

    let last_contract_id = if let Some(contract) = contracts.last() {
        contract.id
    } else {
        0
    };

    let mut last_contract_file_id = 0;

    for contract in contracts.into_iter() {
        let files = sqlx::query!(
            "SELECT * FROM contract_files WHERE contract_id = ?",
            contract.id
        )
        .fetch_all(pool)
        .await?;
        let files_length = files.len();

        let file_cache = HashMap::builder()
            .hasher(RandomState::new())
            .capacity(files_length)
            .build();
        let pinned_file_cache = file_cache.pin();

        for file in files.into_iter() {
            pinned_file_cache.insert(
                file.id,
                ContractFilesCache {
                    path: file.file_path,
                    uploaded_at: file.uploaded_at.unwrap(),
                },
            );
            last_contract_file_id = last_contract_file_id.max(file.id);
        }
        drop(pinned_file_cache);

        pinned_contracts_cache.insert(
            contract.id,
            ContractCache {
                contract_number: contract.contract_number,
                date: contract.date,
                date_start: contract.date_start,
                date_end: contract.date_end,
                description: contract.description,
                location: contract::Location::from(contract.location),
                service: contract::Service::from(contract.service),
                status: contract::Status::from(contract.status),
                supplier: contract.supplier,
                type_of_contract: contract::Type::from(contract.r#type),
                created_at: contract.created_at.unwrap(),
                updated_at: contract.updated_at.unwrap(),
                files: file_cache,
            },
        );
    }

    drop(pinned_contracts_cache);

    Ok((contracts_cache, last_contract_id, last_contract_file_id))
}

impl Db {
    pub async fn new() -> Result<(Db, Cache), sqlx::Error> {
        let pool = MySqlPool::connect("mysql://root:root@localhost:3306/gestao_documental").await?;

        sqlx::raw_sql(SCHEMA).execute(&pool).await?;

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

        let (
            (users_cache, last_user_id),
            (contracts_cache, last_contract_id, last_contract_file_id),
        ) = tokio::try_join!(get_users_cache(&pool), get_contracts_cache(&pool))?;

        println!("Connected to Database");

        Ok((
            Db { pool },
            Cache {
                users: users_cache,
                last_user_id: AtomicU32::new(last_user_id),
                contracts: contracts_cache,
                last_contract_id: AtomicU32::new(last_contract_id),
                last_contract_file_id: AtomicU32::new(last_contract_file_id),
            },
        ))
    }
}
