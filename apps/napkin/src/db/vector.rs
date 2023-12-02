use deadpool_postgres::Client;
use pgvector::Vector;

use crate::errors::NapkinError;

pub async fn create_vector(_client: &Client) -> Result<Vector, NapkinError> {
    let embedding = Vector::from(vec![1.0, 2.0, 3.0]);

    Ok(embedding)
}
