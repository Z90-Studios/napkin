use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    errors::{NapkinError, NapkinErrorRoot},
    models::edge_metadata::EdgeMetadata,
};

pub async fn get_edge_metadata(client: &Client) -> Result<Vec<EdgeMetadata>, NapkinError> {
    let _stmt = "SELECT $edge_metadata_fields FROM edge_metadata";
    let _stmt = _stmt.replace("$edge_metadata_fields", &EdgeMetadata::sql_table_fields());
    // Uuid type needs to be casted, otherwise death
    // let _stmt = _stmt.replace("id", "id::text");
    println!("{}", _stmt);
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| EdgeMetadata::from_row_ref(row).unwrap())
        .collect::<Vec<EdgeMetadata>>();

    Ok(results)
}

pub async fn add_edge_metadata(client: &Client, edge_metadata_info: EdgeMetadata) -> Result<EdgeMetadata, NapkinError> {
    let _stmt = "INSERT INTO edge_metadata(owner_id, name, value) VALUES ($1, $2, $3) RETURNING $edge_metadata_fields;";
    let _stmt = _stmt.replace("$edge_metadata_fields", &EdgeMetadata::sql_table_fields());
    // let _stmt = _stmt.replace("id", "id::text");
    println!("{}", &_stmt);
    let stmt = client.prepare(&_stmt).await;
    match stmt {
        Ok(stmt) => client
            .query(
                &stmt,
                &[
                    &edge_metadata_info.owner_id,
                    &edge_metadata_info.name,
                    &edge_metadata_info.value,
                ],
            )
            .await?
            .iter()
            .map(|row| EdgeMetadata::from_row_ref(row).unwrap())
            .collect::<Vec<EdgeMetadata>>()
            .pop()
            .ok_or(NapkinError {
                code: "EDGE_METADATA_NO_ID",
                message: "Edge Metadata with ID {id} Not Found",
                root: NapkinErrorRoot::NotFound,
            }),
        Err(e) => {
            println!("{}", e);
            return Err(NapkinError {
                code: "EDGE_METADATA_NO_ID",
                message: "Project with ID {id} Not Found",
                root: NapkinErrorRoot::NotFound,
            });
        }
    }
}

pub async fn get_edge_metadata_singleton(client: &Client, owner_id: &String, name: &String) -> Result<EdgeMetadata, NapkinError> {
    let _stmt = "SELECT $edge_metadata_fields FROM edge_metadata WHERE (owner_id = '$owner_id' AND name = '$name');";
    let _stmt = _stmt.replace("$edge_metadata_fields", &EdgeMetadata::sql_table_fields());
    let _stmt = _stmt.replace("$owner_id", owner_id);
    let _stmt = _stmt.replace("$name", name);
    println!("{}", &_stmt);
    let _stmt = client.prepare(&_stmt).await;
    if _stmt.is_err() {
        println!("{}", _stmt.err().unwrap());
        return Err(NapkinError {
            code: "EDGE_METADATA_NO_ID",
            message: "Edge Metadata with ID ({owner_id}, {name}) Not Found",
            root: NapkinErrorRoot::NotFound,
        });
    } else {
        let stmt = _stmt.unwrap();

        client
            .query(&stmt, &[])
            .await?
            .iter()
            .map(|row| EdgeMetadata::from_row_ref(row).unwrap())
            .collect::<Vec<EdgeMetadata>>()
            .pop()
            .ok_or(NapkinError {
                code: "EDGE_METADATA_NO_ID",
                message: "Edge Metadata with ID ({owner_id}, {name}) Not Found",
                root: NapkinErrorRoot::NotFound,
            })
    }
}

pub async fn update_edge_metadata(
    client: &Client,
    owner_id: &String,
    name: &String,
    edge_metadata_info: EdgeMetadata,
) -> Result<EdgeMetadata, NapkinError> {
    let _stmt = "UPDATE edge_metadata SET $updates WHERE id = ANY ('{$owner_id}', '{$name}') RETURNING $edge_metadata_fields;";
    let _stmt = _stmt.replace("$edge_metadata_fields", &EdgeMetadata::sql_table_fields());
    let _stmt = _stmt.replace("owner_id", "owner_id::text");
    let _stmt = _stmt.replace("$owner_id", owner_id);
    let _stmt = _stmt.replace("$name", name);
    let _stmt = _stmt.replace("$updates", &EdgeMetadata::to_update_str(&edge_metadata_info));
    let stmt = client.prepare(&_stmt).await.unwrap();
    println!("{}", &_stmt);

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| EdgeMetadata::from_row_ref(row).unwrap())
        .collect::<Vec<EdgeMetadata>>()
        .pop()
        .ok_or(NapkinError {
            code: "EDGE_METADATA_NO_ID",
            message: "Edge Metadata with ID ({owner_id}, {name}) Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}

pub async fn delete_edge(client: &Client, owner_id: &String, name: &String) -> Result<EdgeMetadata, NapkinError> {
    let _stmt = "DELETE FROM edge_metadata WHERE id = ANY ('{$owner_id}', '{$name}') RETURNING $edge_metadata_fields;";
    let _stmt = _stmt.replace("$owner_id", owner_id);
    let _stmt = _stmt.replace("$edge_metadata_fields", &EdgeMetadata::sql_table_fields());
    let _stmt = _stmt.replace("owner_id", "owner_id::text");
    let _stmt = _stmt.replace("$name", name);
    println!("{}", &_stmt);
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| EdgeMetadata::from_row_ref(row).unwrap())
        .collect::<Vec<EdgeMetadata>>()
        .pop()
        .ok_or(NapkinError {
            code: "EDGE_METADATA_NO_ID",
            message: "Edge Metadata with ID {edge_metadata_id} Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}
