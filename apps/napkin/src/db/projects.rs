use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    errors::{NapkinError, NapkinErrorRoot},
    models::projects::Project,
};

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

pub async fn add_project(client: &Client, project_info: Project) -> Result<Project, NapkinError> {
    let _stmt = "INSERT INTO projects(name) VALUES ($1) RETURNING $project_fields;";
    let _stmt = _stmt.replace("$project_fields", &Project::sql_table_fields());
    let _stmt = _stmt.replace("id", "id::text");
    let stmt = client.prepare(&_stmt).await.unwrap();
    println!("{}", &_stmt);

    client
        .query(&stmt, &[&project_info.name])
        .await?
        .iter()
        .map(|row| Project::from_row_ref(row).unwrap())
        .collect::<Vec<Project>>()
        .pop()
        .ok_or(NapkinError {
            code: "PROJECT_NO_ID",
            message: "Project with ID {id} Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}

pub async fn get_project(client: &Client, project_id: &String) -> Result<Project, NapkinError> {
    let _stmt = "SELECT $project_fields FROM projects WHERE id = ANY ('{$id}');";
    let _stmt = _stmt.replace("$project_fields", &Project::sql_table_fields());
    let _stmt = _stmt.replace("$id", project_id);
    let _stmt = _stmt.replace("id", "id::text");
    let stmt = client.prepare(&_stmt).await.unwrap();
    println!("{}", &_stmt);

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Project::from_row_ref(row).unwrap())
        .collect::<Vec<Project>>()
        .pop()
        .ok_or(NapkinError {
            code: "PROJECT_NO_ID",
            message: "Project with ID {project_id} Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}

pub async fn update_project(
    client: &Client,
    project_id: &String,
    project_info: Project,
) -> Result<Project, NapkinError> {
    let _stmt = "UPDATE projects SET $updates WHERE id = ANY ('{$id}') RETURNING $project_fields;";
    let _stmt = _stmt.replace("$project_fields", &Project::sql_table_fields());
    let _stmt = _stmt.replace("id", "id::text");
    let _stmt = _stmt.replace("$id", project_id);
    let _stmt = _stmt.replace("$updates", &Project::to_update_str(&project_info));
    let stmt = client.prepare(&_stmt).await.unwrap();
    println!("{}", &_stmt);

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Project::from_row_ref(row).unwrap())
        .collect::<Vec<Project>>()
        .pop()
        .ok_or(NapkinError {
            code: "PROJECT_NO_ID",
            message: "Project with ID {project_id} Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}

pub async fn delete_project(client: &Client, project_id: &String) -> Result<Project, NapkinError> {
    let _stmt = "DELETE FROM projects WHERE id = ANY ('{$id}') RETURNING $project_fields;";
    let _stmt = _stmt.replace("$id", project_id);
    let _stmt = _stmt.replace(
        "$project_fields",
        &Project::sql_table_fields(),
    );
    let _stmt = _stmt.replace("id", "id::text");
    println!("{}", &_stmt);
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Project::from_row_ref(row).unwrap())
        .collect::<Vec<Project>>()
        .pop()
        .ok_or(NapkinError {
            code: "PROJECT_NO_ID",
            message: "Project with ID {project_id} Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}
