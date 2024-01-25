use actix_web::{ get, post, put, delete, web, Responder, Result };
use deadpool_postgres::{Client, Pool};

use crate::models::nodes::{Node, NodeReqObj};
use crate::errors::{ NapkinError, NapkinErrorRoot, handle_pool_error };
use crate::db;

#[get("")]
pub async fn get_nodes(db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    // let nodes: Vec<Node> = Vec::new();
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;
    let nodes = db::nodes::get_nodes(&client).await?;
    Ok(web::Json(nodes))
}

#[post("")]
pub async fn post_node(body: web::Json<NodeReqObj>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    // let embedding: pgvector::Vector = pgvector::Vector::from(body.embedding.clone());
    let project_uuid = uuid::Uuid::parse_str(&body.project);
    if project_uuid.is_err() {
        return Err(NapkinError {
            code: "NODE_NO_ID",
            message: "Project with ID {id} Not Found",
            root: NapkinErrorRoot::NotFound,
        });
    }
    let node_info = Node {
        id: body.id.clone(),
        project: project_uuid.unwrap(),
        title: body.title.clone(),
        data: serde_json::Value::String(body.data.clone()),
        // embedding: embedding,
    };

    let new_node = db::nodes::add_node(&client, node_info).await?;

    Ok(web::Json(new_node))
}

#[get("/{id}")]
pub async fn get_node(id: web::Path<String>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let node = db::nodes::get_node(&client, &id).await?;

    Ok(web::Json(node))
}

#[put("/{id}")]
pub async fn update_node(id: web::Path<String>, body: web::Json<Node>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let node_info: Node = body.into_inner();
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let updated_node = db::nodes::update_node(&client, &id, node_info).await?;

    Ok(web::Json(updated_node))
}

#[delete("/{id}")]
pub async fn delete_node(id: web::Path<String>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let deleted_node = db::nodes::delete_node(&client, &id).await?;

    Ok(web::Json(deleted_node))
}