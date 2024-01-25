use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    errors::{NapkinError, NapkinErrorRoot},
    models::nodes::Node,
};

pub async fn get_nodes(client: &Client) -> Result<Vec<Node>, NapkinError> {
    let _stmt = "SELECT $node_fields FROM nodes";
    let _stmt = _stmt.replace("$node_fields", &Node::sql_table_fields());
    // Uuid type needs to be casted, otherwise death
    let _stmt = _stmt.replace("id", "id::text");
    println!("{}", _stmt);
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Node::from_row_ref(row).unwrap())
        .collect::<Vec<Node>>();

    Ok(results)
}

pub async fn add_node(client: &Client, node_info: Node) -> Result<Node, NapkinError> {
    let _stmt = "INSERT INTO nodes(project, title, data) VALUES ($1, $2, $3) RETURNING $node_fields;";
    let _stmt = _stmt.replace("$node_fields", &Node::sql_table_fields());
    let _stmt = _stmt.replace("id", "id::text");
    println!("{}", &_stmt);
    let stmt = client.prepare(&_stmt).await;
    match stmt {
        Ok(stmt) => client
            .query(
                &stmt,
                &[
                    &node_info.project,
                    &node_info.title,
                    &node_info.data,
                    // &node_info.embedding,
                ],
            )
            .await?
            .iter()
            .map(|row| Node::from_row_ref(row).unwrap())
            .collect::<Vec<Node>>()
            .pop()
            .ok_or(NapkinError {
                code: "NODE_NO_ID",
                message: "Node with ID {id} Not Found",
                root: NapkinErrorRoot::NotFound,
            }),
        Err(e) => {
            println!("{}", e);
            return Err(NapkinError {
                code: "NODE_NO_ID",
                message: "Project with ID {id} Not Found",
                root: NapkinErrorRoot::NotFound,
            });
        }
    }
}

pub async fn get_node(client: &Client, node_id: &String) -> Result<Node, NapkinError> {
    let _stmt = "SELECT $node_fields FROM nodes WHERE id = ANY ('{$id}');";
    let _stmt = _stmt.replace("$node_fields", &Node::sql_table_fields());
    let _stmt = _stmt.replace("$id", node_id);
    let _stmt = _stmt.replace("id", "id::text");
    let stmt = client.prepare(&_stmt).await.unwrap();
    println!("{}", &_stmt);

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Node::from_row_ref(row).unwrap())
        .collect::<Vec<Node>>()
        .pop()
        .ok_or(NapkinError {
            code: "NODE_NO_ID",
            message: "Node with ID {node_id} Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}

pub async fn update_node(
    client: &Client,
    node_id: &String,
    node_info: Node,
) -> Result<Node, NapkinError> {
    let _stmt = "UPDATE nodes SET $updates WHERE id = ANY ('{$id}') RETURNING $node_fields;";
    let _stmt = _stmt.replace("$node_fields", &Node::sql_table_fields());
    let _stmt = _stmt.replace("id", "id::text");
    let _stmt = _stmt.replace("$id", node_id);
    let _stmt = _stmt.replace("$updates", &Node::to_update_str(&node_info));
    let stmt = client.prepare(&_stmt).await.unwrap();
    println!("{}", &_stmt);

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Node::from_row_ref(row).unwrap())
        .collect::<Vec<Node>>()
        .pop()
        .ok_or(NapkinError {
            code: "NODE_NO_ID",
            message: "Node with ID {node_id} Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}

pub async fn delete_node(client: &Client, node_id: &String) -> Result<Node, NapkinError> {
    let _stmt = "DELETE FROM nodes WHERE id = ANY ('{$id}') RETURNING $node_fields;";
    let _stmt = _stmt.replace("$id", node_id);
    let _stmt = _stmt.replace("$node_fields", &Node::sql_table_fields());
    let _stmt = _stmt.replace("id", "id::text");
    println!("{}", &_stmt);
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| Node::from_row_ref(row).unwrap())
        .collect::<Vec<Node>>()
        .pop()
        .ok_or(NapkinError {
            code: "NODE_NO_ID",
            message: "Node with ID {node_id} Not Found",
            root: NapkinErrorRoot::NotFound,
        })
}
