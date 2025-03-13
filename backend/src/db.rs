use ahash::RandomState;
use papaya::HashMap;
use serde::Serialize;
use sqlx::mysql::MySqlPool;

use crate::models::contract;
use crate::utils::hashing_utils::hash;

const SCHEMA: &str = include_str!("../sql/schema.sql");

fn serialize_date_dmy<S>(date: &chrono::NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&date.format("%d/%m/%Y").to_string())
}

fn serialize_datetime_dmy<S>(
    date: &chrono::DateTime<chrono::Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&date.format("%d/%m/%Y, %H:%M:%S").to_string())
}

pub struct UserCache {
    pub username: String,
    pub email: String,
    pub password: [u8; 48],
    pub is_admin: bool,
}

#[derive(Serialize, Clone)]
pub struct ContractFilesCache {
    pub path: String,
    #[serde(rename = "uploadedAt")]
    #[serde(serialize_with = "serialize_datetime_dmy")]
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Clone)]
pub struct ContractCache {
    #[serde(rename = "contractNumber")]
    pub contract_number: u32,
    #[serde(rename = "dateString")]
    #[serde(serialize_with = "serialize_date_dmy")]
    pub date: chrono::NaiveDate,
    #[serde(rename = "dateStartString")]
    #[serde(serialize_with = "serialize_date_dmy")]
    pub date_start: chrono::NaiveDate,
    #[serde(rename = "dateEndString")]
    #[serde(serialize_with = "serialize_date_dmy")]
    pub date_end: chrono::NaiveDate,
    pub description: String,
    pub location: contract::Location,
    pub service: contract::Service,
    pub status: contract::Status,
    pub supplier: String,
    #[serde(rename = "type")]
    pub type_of_contract: contract::Type,
    #[serde(rename = "createdAt")]
    #[serde(serialize_with = "serialize_datetime_dmy")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "uploadedAt")]
    #[serde(serialize_with = "serialize_datetime_dmy")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub files: HashMap<u32, ContractFilesCache, RandomState>,
}

pub struct PageVisit {
    pub visit_count: u32,
    pub last_visited_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Hash, Eq, PartialEq)]
pub struct AnalyticsKey {
    pub user_id: u32,
    pub page_path: String,
}

pub struct Db {
    pub pool: MySqlPool,
}

pub struct Cache {
    pub users: HashMap<u32, UserCache, RandomState>,
    pub contracts: HashMap<u32, ContractCache, RandomState>,
    pub analytics: HashMap<AnalyticsKey, PageVisit, RandomState>,
}

#[inline(always)]
fn i8_to_bool(i: i8) -> bool {
    i != 0
}

async fn get_analytics_cache(
    pool: &sqlx::Pool<sqlx::MySql>,
) -> Result<HashMap<AnalyticsKey, PageVisit, RandomState>, sqlx::Error> {
    let analytics = sqlx::query!("SELECT * FROM user_page_analytics")
        .fetch_all(pool)
        .await?;

    let analytics_cache = HashMap::builder()
        .hasher(RandomState::new())
        .capacity(analytics.len())
        .build();

    let pinned_analytics_cache = analytics_cache.pin();

    for entry in analytics.into_iter() {
        let key = AnalyticsKey {
            user_id: entry.user_id,
            page_path: entry.page_path,
        };

        pinned_analytics_cache.insert(
            key,
            PageVisit {
                visit_count: entry.visit_count,
                last_visited_at: entry.last_visited_at.unwrap(),
            },
        );
    }

    drop(pinned_analytics_cache);
    Ok(analytics_cache)
}

async fn get_users_cache(
    pool: &sqlx::Pool<sqlx::MySql>,
) -> Result<HashMap<u32, UserCache, RandomState>, sqlx::Error> {
    let users = sqlx::query!("SELECT * FROM users").fetch_all(pool).await?;
    let users_length = users.len();

    let users_cache = HashMap::builder()
        .hasher(RandomState::new())
        .capacity(users_length)
        .build();

    let pinned_users_cache = users_cache.pin();

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
    Ok(users_cache)
}

async fn get_contracts_cache(
    pool: &sqlx::Pool<sqlx::MySql>,
) -> Result<HashMap<u32, ContractCache, RandomState>, sqlx::Error> {
    let contracts = sqlx::query!("SELECT * FROM contracts")
        .fetch_all(pool)
        .await?;

    let contracts_length = contracts.len();

    let contracts_cache = HashMap::builder()
        .hasher(RandomState::new())
        .capacity(contracts_length)
        .build();
    let pinned_contracts_cache = contracts_cache.pin();

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

    Ok(contracts_cache)
}

impl Db {
    pub async fn new() -> Result<(Db, Cache), sqlx::Error> {
        let pool = MySqlPool::connect(
            "mariadb://root:root@localhost:3306/gestao_documental?password=root",
        )
        .await?;

        sqlx::raw_sql(SCHEMA).execute(&pool).await?;

        let admin_password = hash("admin");

        sqlx::query!(
            "INSERT IGNORE INTO users (username, email, password, is_admin) VALUES (?, ?, ?, ?)",
            "admin",
            "admin@jcc.com",
            &admin_password[..],
            true
        )
        .execute(&pool)
        .await?;

        let (users_cache, contracts_cache, analytics_cache) = tokio::try_join!(
            get_users_cache(&pool),
            get_contracts_cache(&pool),
            get_analytics_cache(&pool)
        )?;

        println!("Connected to Database");

        Ok((
            Db { pool },
            Cache {
                users: users_cache,
                contracts: contracts_cache,
                analytics: analytics_cache,
            },
        ))
    }
}
