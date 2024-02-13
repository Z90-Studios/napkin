use actix_web::{ get, post, put, delete, web, Responder, Result };
use deadpool_postgres::{Client, Pool};

use crate::models::edge_metadata::{EdgeMetadata, EdgeMetadataReqObj};
use crate::errors::{ NapkinError, NapkinErrorRoot, handle_pool_error };
use crate::db;

#[get("")]
pub async fn get_edge_metadata(db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;
    let edge_metadata = db::edge_metadata::get_edge_metadata(&client).await?;
    Ok(web::Json(edge_metadata))
}

#[post("")]
pub async fn post_edge_metadata(body: web::Json<EdgeMetadataReqObj>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let owner_id_uuid = uuid::Uuid::parse_str(&body.owner_id);

    // TODO: Check if owner_id exists

    if owner_id_uuid.is_err() {
        return Err(NapkinError {
            code: "EDGE_NO_ID",
            message: "ID `{owner_id_uuid}` Invalid or Not Found",
            root: NapkinErrorRoot::NotFound,
        });
    }
    let edge_metadata_info = EdgeMetadata {
        owner_id: owner_id_uuid.unwrap(),
        name: body.name.clone(),
        value: body.value.clone(),
    };

    let new_edge_metadata = db::edge_metadata::add_edge_metadata(&client, edge_metadata_info).await?;

    Ok(web::Json(new_edge_metadata))
}

#[get("/{owner_id}")]
pub async fn get_edge_metadata_singleton(owner_id: web::Path<String>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    println!("{}", owner_id);
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let edge_metadata = db::edge_metadata::get_edge_metadata_singleton(&client, &owner_id).await?;

    Ok(web::Json(edge_metadata))
}

#[get("/{owner_id}/{name}")]
pub async fn get_edge_metadata_singleton_key(param: web::Path<(String, String)>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let (owner_id, name) = param.into_inner();
    println!("{} {}", owner_id, name);
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let edge_metadata_key = db::edge_metadata::get_edge_metadata_singleton_key(&client, &owner_id, &name).await?;

    Ok(web::Json(edge_metadata_key))
}

#[put("/{owner_id}/{name}")]
pub async fn update_edge_metadata(owner_id: web::Path<String>, name: web::Path<String>, body: web::Json<EdgeMetadata>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let edge_info: EdgeMetadata = body.into_inner();
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let updated_edge = db::edge_metadata::update_edge_metadata(&client, &owner_id, &name, edge_info).await?;

    Ok(web::Json(updated_edge))
}

#[delete("/{owner_id}/{name}")]
pub async fn delete_edge_metadata(owner_id: web::Path<String>, name: web::Path<String>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let deleted_edge = db::edge_metadata::delete_edge(&client, &owner_id, &name).await?;

    Ok(web::Json(deleted_edge))
}


