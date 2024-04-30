use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    errors::{NapkinError, NapkinErrorRoot},
    models::node_metadata::NodeMetadata,
};

pub async fn get_node_metadata(client: &Client) -> Result<Vec<NodeMetadata>, NapkinError> {
    let _stmt = "SELECT $node_metadata_fields FROM node_metadata";
    let _stmt = _stmt.replace("$node_metadata_fields", &NodeMetadata::sql_table_fields());
    // Uuid type needs to be casted, otherwise death
    // let _stmt = _stmt.replace("id", "id::text");
    println!("{}", _stmt);
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| NodeMetadata::from_row_ref(row).unwrap())
        .collect::<Vec<NodeMetadata>>();

    Ok(results)
}

pub async fn add_node_metadata(client: &Client, node_metadata_info: NodeMetadata) -> Result<NodeMetadata, NapkinError> {
    let _stmt = "INSERT INTO node_metadata(owner_id, name, value) VALUES ($1, $2, $3) RETURNING $node_metadata_fields;";
    let _stmt = _stmt.replace("$node_metadata_fields", &NodeMetadata::sql_table_fields());
    // let _stmt = _stmt.replace("id", "id::text");
    println!("{}", &_stmt);
    let stmt = client.prepare(&_stmt).await;
    match stmt {
        Ok(stmt) => client
            .query(
                &stmt,
                &[
                    &node_metadata_info.owner_id,
                    &node_metadata_info.name,
                    &node_metadata_info.value,
                ],
            )
            .await?
            .iter()
            .map(|row| NodeMetadata::from_row_ref(row).unwrap())
            .collect::<Vec<NodeMetadata>>()
            .pop()
            .ok_or(NapkinError {
                code: "NODE_METADATA_NO_ID",
                message: "Node Metadata with ID {id} Not Found",
                root: NapkinErrorRoot::NotFound,
            }),
        Err(e) => {
            println!("{}", e);
            return Err(NapkinError {
                code: "NODE_METADATA_NO_ID",
                message: "Project with ID {id} Not Found",
                root: NapkinErrorRoot::NotFound,
            });
        }
    }
}

pub async fn get_node_metadata_singleton(client: &Client, owner_id: &String) -> Result<Vec<NodeMetadata>, NapkinError> {
    let _stmt = "SELECT $node_metadata_fields FROM node_metadata WHERE (owner_id = '$owner_id');";
    let _stmt = _stmt.replace("$node_metadata_fields", &NodeMetadata::sql_table_fields());
    let _stmt = _stmt.replace("$owner_id", owner_id);
    println!("{}", &_stmt);
    let _stmt = client.prepare(&_stmt).await;
    if _stmt.is_err() {
        println!("{}", _stmt.err().unwrap());
        return Err(NapkinError {
            code: "NODE_METADATA_NO_ID",
            message: "Node Metadata with ID ({owner_id}, {name}) Not Found",
            root: NapkinErrorRoot::NotFound,
        });
    } else {
        let stmt = _stmt.unwrap();

        let results = client
            .query(&stmt, &[])
            .await?
            .iter()
            .map(|row| NodeMetadata::from_row_ref(row).unwrap())
            .collect::<Vec<NodeMetadata>>();
    
        if results.is_empty() {
            Err(NapkinError {
                code: "NODE_METADATA_NO_ID",
                message: "Node Metadata with ID ({owner_id}, {name}) Not Found",
                root: NapkinErrorRoot::NotFound,
            })
        } else {
            Ok(results)
        }
    }
}

pub async fn get_node_metadata_singleton_key(client: &Client, owner_id: &String, name: &String) -> Result<NodeMetadata, NapkinError> {
    let _stmt = "SELECT $node_metadata_fields FROM node_metadata WHERE (owner_id = '$owner_id' AND name = '$name');";
    let _stmt = _stmt.replace("$node_metadata_fields", &NodeMetadata::sql_table_fields());
    let _stmt = _stmt.replace("$owner_id", owner_id);
    let _stmt = _stmt.replace("$name", name);
    println!("{}", &_stmt);
    let _stmt = client.prepare(&_stmt).await;
    if _stmt.is_err() {
        println!("{}", _stmt.err().unwrap());
        return Err(NapkinError {
            code: "NODE_METADATA_NO_ID",
            message: "Node Metadata with ID ({owner_id}, {name}) Not Found",
            root: NapkinErrorRoot::NotFound,
        });
    } else {
        let stmt = _stmt.unwrap();

        client
            .query(&stmt, &[])
            .await?
            .iter()
            .map(|row| NodeMetadata::from_row_ref(row).unwrap())
            .collect::<Vec<NodeMetadata>>()
            .pop()
            .ok_or(NapkinError {
                code: "NODE_METADATA_NO_ID",
                message: "Node Metadata with ID ({owner_id}, {name}) Not Found",
                root: NapkinErrorRoot::NotFound,
            })
    }
}

pub async fn update_node_metadata(
    client: &Client,
    owner_id: &String,
    name: &String,
    node_metadata_info: NodeMetadata,
) -> Result<NodeMetadata, NapkinError> {
    let _stmt = "UPDATE node_metadata $updates WHERE id = ANY ('{$owner_id}', '{$name}') RETURNING $node_metadata_fields;";
    let _stmt = _stmt.replace("$node_metadata_fields", &NodeMetadata::sql_table_fields());
    let _stmt = _stmt.replace("owner_id", "owner_id::text");
    let _stmt = _stmt.replace("$owner_id", owner_id);
    let _stmt = _stmt.replace("$name", name);
    let _stmt = _stmt.replace("$updates", &NodeMetadata::to_update_str(&node_metadata_info));
    let stmt = client.prepare(&_stmt).await.unwrap();
    println!("{}", &_stmt);

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| NodeMetadata::from_row_ref(row).unwrap())
        .collect::<Vec<NodeMetadata>>()
        .pop()
        .ok_or(NapkinError {
            code: "NODE_METADATA_NO_ID",
            message: "Node Metadata with ID ({owner_id}, {name}) Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}

pub async fn delete_node(client: &Client, owner_id: &String, name: &String) -> Result<NodeMetadata, NapkinError> {
    let _stmt = "DELETE FROM node_metadata WHERE id = ANY ('{$owner_id}', '{$name}') RETURNING $node_metadata_fields;";
    let _stmt = _stmt.replace("$owner_id", owner_id);
    let _stmt = _stmt.replace("$node_metadata_fields", &NodeMetadata::sql_table_fields());
    let _stmt = _stmt.replace("owner_id", "owner_id::text");
    let _stmt = _stmt.replace("$name", name);
    println!("{}", &_stmt);
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| NodeMetadata::from_row_ref(row).unwrap())
        .collect::<Vec<NodeMetadata>>()
        .pop()
        .ok_or(NapkinError {
            code: "NODE_METADATA_NO_ID",
            message: "Node Metadata with ID {node_metadata_id} Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}
