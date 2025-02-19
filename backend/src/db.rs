use sqlx::mysql::MySqlPool;

pub struct Db {
    pub pool: MySqlPool,
}

impl Db {
    pub async fn new() -> Result<Db, sqlx::Error> {
        let pool = MySqlPool::connect("mysql://root:root@localhost:3306/gestao_documental").await?;
        Ok(Db { pool })
    }
}
