use actix_web::{ get, post, put, delete, web, Responder, Result };
use deadpool_postgres::{Client, Pool};

use crate::models::node_metadata::{NodeMetadata, NodeMetadataReqObj};
use crate::errors::{ NapkinError, NapkinErrorRoot, handle_pool_error };
use crate::db;

#[get("")]
pub async fn get_node_metadata(db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;
    let node_metadata = db::node_metadata::get_node_metadata(&client).await?;
    Ok(web::Json(node_metadata))
}

#[post("")]
pub async fn post_node_metadata(body: web::Json<NodeMetadataReqObj>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let owner_id_uuid = uuid::Uuid::parse_str(&body.owner_id);

    // TODO: Check if owner_id exists

    if owner_id_uuid.is_err() {
        return Err(NapkinError {
            code: "NODE_NO_ID",
            message: "ID `{owner_id_uuid}` Invalid or Not Found",
            root: NapkinErrorRoot::NotFound,
        });
    }
    let node_metadata_info = NodeMetadata {
        owner_id: owner_id_uuid.unwrap(),
        name: body.name.clone(),
        value: body.value.clone(),
    };

    let new_node_metadata = db::node_metadata::add_node_metadata(&client, node_metadata_info).await?;

    Ok(web::Json(new_node_metadata))
}

#[get("/{owner_id}")]
pub async fn get_node_metadata_singleton(id: web::Path<String>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let owner_id_uuid = uuid::Uuid::parse_str(&id);

    if owner_id_uuid.is_err() {
        return Err(NapkinError {
            code: "NODE_NO_ID",
            message: "ID `{owner_id_uuid}` Invalid or Not Found",
            root: NapkinErrorRoot::NotFound,
        });
    }
    let node_metadata = db::node_metadata::get_node_metadata_singleton(&client, &id).await?;

    Ok(web::Json(node_metadata))
}

#[get("/{owner_id}/{name}")]
pub async fn get_node_metadata_singleton_key(param: web::Path<(String, String)>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let (owner_id, name) = param.into_inner();
    println!("{} {}", owner_id, name);
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let node_metadata_key = db::node_metadata::get_node_metadata_singleton_key(&client, &owner_id, &name).await?;

    Ok(web::Json(node_metadata_key))
}

#[put("/{owner_id}/{name}")]
pub async fn update_node_metadata(owner_id: web::Path<String>, name: web::Path<String>, body: web::Json<NodeMetadata>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let node_info: NodeMetadata = body.into_inner();
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let updated_node = db::node_metadata::update_node_metadata(&client, &owner_id, &name, node_info).await?;

    Ok(web::Json(updated_node))
}

#[delete("/{owner_id}/{name}")]
pub async fn delete_node_metadata(owner_id: web::Path<String>, name: web::Path<String>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let deleted_node = db::node_metadata::delete_node(&client, &owner_id, &name).await?;

    Ok(web::Json(deleted_node))
}


