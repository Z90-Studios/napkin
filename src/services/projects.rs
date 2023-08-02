use actix_web::{ get, post, put, delete, web, Responder, Result };
use deadpool_postgres::{Client, Pool};

use crate::models::projects::Project;
use crate::errors::{ NapkinError, handle_pool_error };
use crate::db;

#[get("")]
pub async fn get_projects(db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    // let projects: Vec<Project> = Vec::new();
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;
    let projects = db::projects::get_projects(&client).await?;
    Ok(web::Json(projects))
}

#[post("")]
pub async fn post_project(body: web::Json<Project>, db_pool: web::Data<Pool>) -> Result<impl Responder> {
    let project_info: Project = body.into_inner();
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let new_project = db::projects::add_project(&client, project_info).await?;

    Ok(web::Json(new_project))
}

#[get("/{id}")]
pub async fn get_project(id: web::Path<String>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let project = db::projects::get_project(&client, &id).await?;

    Ok(web::Json(project))
}

#[put("/{id}")]
pub async fn update_project(id: web::Path<String>, body: web::Json<Project>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let project_info: Project = body.into_inner();
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let updated_project = db::projects::update_project(&client, &id, project_info).await?;

    Ok(web::Json(updated_project))
}

#[delete("/{id}")]
pub async fn delete_project(id: web::Path<String>, db_pool: web::Data<Pool>) -> Result<impl Responder, NapkinError> {
    let client: Client = db_pool.get().await.map_err(handle_pool_error)?;

    let deleted_project = db::projects::delete_project(&client, &id).await?;

    Ok(web::Json(deleted_project))
}