use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    errors::{NapkinError, NapkinErrorRoot},
    models::edges::Edge,
};

pub async fn get_edges(client: &Client) -> Result<Vec<Edge>, NapkinError> {
    let _stmt = "SELECT $edge_fields FROM edges";
    let _stmt = _stmt.replace("$edge_fields", &Edge::sql_table_fields());
    // Uuid type needs to be casted, otherwise death
    let _stmt = _stmt.replace("id", "id::text");
    println!("{}", _stmt);
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Edge::from_row_ref(row).unwrap())
        .collect::<Vec<Edge>>();

    Ok(results)
}

pub async fn add_edge(client: &Client, edge_info: Edge) -> Result<Edge, NapkinError> {
    let _stmt = "INSERT INTO edges(project, source, target) VALUES ($1, $2, $3) RETURNING $edge_fields;";
    let _stmt = _stmt.replace("$edge_fields", &Edge::sql_table_fields());
    let _stmt = _stmt.replace("id", "id::text");
    println!("{}", &_stmt);
    let stmt = client.prepare(&_stmt).await;
    match stmt {
        Ok(stmt) => client
            .query(
                &stmt,
                &[
                    &edge_info.project,
                    &edge_info.source,
                    &edge_info.target,
                ],
            )
            .await?
            .iter()
            .map(|row| Edge::from_row_ref(row).unwrap())
            .collect::<Vec<Edge>>()
            .pop()
            .ok_or(NapkinError {
                code: "EDGE_NO_ID",
                message: "Edge with ID {id} Not Found",
                root: NapkinErrorRoot::NotFound,
            }),
        Err(e) => {
            println!("{}", e);
            return Err(NapkinError {
                code: "EDGE_NO_ID",
                message: "Project with ID {id} Not Found",
                root: NapkinErrorRoot::NotFound,
            });
        }
    }
}

pub async fn get_edge(client: &Client, edge_id: &String) -> Result<Edge, NapkinError> {
    let _stmt = "SELECT $edge_fields FROM edges WHERE id = ANY ('{$id}');";
    let _stmt = _stmt.replace("$edge_fields", &Edge::sql_table_fields());
    let _stmt = _stmt.replace("$id", edge_id);
    let _stmt = _stmt.replace("id", "id::text");
    let stmt = client.prepare(&_stmt).await.unwrap();
    println!("{}", &_stmt);

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Edge::from_row_ref(row).unwrap())
        .collect::<Vec<Edge>>()
        .pop()
        .ok_or(NapkinError {
            code: "EDGE_NO_ID",
            message: "Edge with ID {edge_id} Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}

pub async fn update_edge(
    client: &Client,
    edge_id: &String,
    edge_info: Edge,
) -> Result<Edge, NapkinError> {
    let _stmt = "UPDATE edges SET $updates WHERE id = ANY ('{$id}') RETURNING $edge_fields;";
    let _stmt = _stmt.replace("$edge_fields", &Edge::sql_table_fields());
    let _stmt = _stmt.replace("id", "id::text");
    let _stmt = _stmt.replace("$id", edge_id);
    let _stmt = _stmt.replace("$updates", &Edge::to_update_str(&edge_info));
    let stmt = client.prepare(&_stmt).await.unwrap();
    println!("{}", &_stmt);

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Edge::from_row_ref(row).unwrap())
        .collect::<Vec<Edge>>()
        .pop()
        .ok_or(NapkinError {
            code: "EDGE_NO_ID",
            message: "Edge with ID {edge_id} Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}

pub async fn delete_edge(client: &Client, edge_id: &String) -> Result<Edge, NapkinError> {
    let _stmt = "DELETE FROM edges WHERE id = ANY ('{$id}') RETURNING $edge_fields;";
    let _stmt = _stmt.replace("$id", edge_id);
    let _stmt = _stmt.replace("$edge_fields", &Edge::sql_table_fields());
    let _stmt = _stmt.replace("id", "id::text");
    println!("{}", &_stmt);
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Edge::from_row_ref(row).unwrap())
        .collect::<Vec<Edge>>()
        .pop()
        .ok_or(NapkinError {
            code: "EDGE_NO_ID",
            message: "Edge with ID {edge_id} Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}
