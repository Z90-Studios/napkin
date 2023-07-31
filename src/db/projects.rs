use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{errors::NapkinError, models::projects::Project};

pub async fn get_projects(client: &Client) -> Result<Vec<Project>, NapkinError> {
    let _stmt = "SELECT $project_fields FROM projects";
    let _stmt = _stmt.replace("$project_fields", &Project::sql_table_fields());
    // Uuid type needs to be casted, otherwise death
    let _stmt = _stmt.replace("id", "id::text");
    println!("{}", _stmt);
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Project::from_row_ref(row).unwrap())
        .collect::<Vec<Project>>();

    Ok(results)
}