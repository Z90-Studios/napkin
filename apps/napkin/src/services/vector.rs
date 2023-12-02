use actix_web::{post, web, Responder, Result};
use deadpool_postgres::{Client, Pool};

use crate::db;
use crate::errors::handle_pool_error;

#[post("")]
pub async fn create_vector(
    db_pool: web::Data<Pool>,
) -> Result<impl Responder> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let new_vector = db::vector::create_vector(&client).await?;

    Ok(web::Json(new_vector))
}
