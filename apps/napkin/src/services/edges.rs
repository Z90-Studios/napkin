use actix_web::{ get, post, put, delete, web, Responder, Result };
use deadpool_postgres::{Client, Pool};

use crate::models::edges::{Edge, EdgeReqObj};
use crate::errors::{ NapkinError, NapkinErrorRoot, handle_pool_error };
use crate::db;

#[get("")]
pub async fn get_edges(db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    // let edges: Vec<Edge> = Vec::new();
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;
    let edges = db::edges::get_edges(&client).await?;
    Ok(web::Json(edges))
}

#[post("")]
pub async fn post_edge(body: web::Json<EdgeReqObj>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    // let embedding: pgvector::Vector = pgvector::Vector::from(body.embedding.clone());
    let project_uuid = uuid::Uuid::parse_str(&body.project);
    let source_uuid = uuid::Uuid::parse_str(&body.source);
    let target_uuid = uuid::Uuid::parse_str(&body.target);

    // TODO: Check if project exists

    if project_uuid.is_err() {
        return Err(NapkinError {
            code: "EDGE_NO_ID",
            message: "Project with ID {id} Not Found",
            root: NapkinErrorRoot::NotFound,
        });
    }
    if source_uuid.is_err() {
        return Err(NapkinError {
            code: "EDGE_NO_ID",
            message: "Source with ID {id} Not Found",
            root: NapkinErrorRoot::NotFound,
        });
    }
    if target_uuid.is_err() {
        return Err(NapkinError {
            code: "EDGE_NO_ID",
            message: "Target with ID {id} Not Found",
            root: NapkinErrorRoot::NotFound,
        });
    }
    let edge_info = Edge {
        id: body.id.clone(),
        project: project_uuid.unwrap(),
        source: source_uuid.unwrap(),
        target: target_uuid.unwrap(),
    };

    let new_edge = db::edges::add_edge(&client, edge_info).await?;

    Ok(web::Json(new_edge))
}

#[get("/{id}")]
pub async fn get_edge(id: web::Path<String>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let edge = db::edges::get_edge(&client, &id).await?;

    Ok(web::Json(edge))
}

#[put("/{id}")]
pub async fn update_edge(id: web::Path<String>, body: web::Json<Edge>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let edge_info: Edge = body.into_inner();
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let updated_edge = db::edges::update_edge(&client, &id, edge_info).await?;

    Ok(web::Json(updated_edge))
}

#[delete("/{id}")]
pub async fn delete_edge(id: web::Path<String>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let deleted_edge = db::edges::delete_edge(&client, &id).await?;

    Ok(web::Json(deleted_edge))
}