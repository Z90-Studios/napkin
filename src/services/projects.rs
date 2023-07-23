use actix_web::{ get, post, web, Responder, Result };

use crate::models::projects::Project;
use crate::errors::{ NapkinError, NapkinErrorRoot };

#[get("")]
pub async fn get_projects() -> Result<impl Responder> {
    let projects: Vec<Project> = Vec::new();
    Ok(web::Json(projects))
}

#[post("")]
pub async fn post_project(body: web::Json<Project>) -> Result<impl Responder> {
    let mut projects: Vec<Project> = Vec::new();
    projects.push(body.into_inner());

    Ok(web::Json(projects))
}

#[get("/{id}")]
pub async fn get_project(id: web::Path<String>) -> Result<impl Responder, NapkinError> {
    let mut projects: Vec<Project> = Vec::new();
    projects.push(Project {
        id: "123".to_string(),
        name: "Napkin".to_string(),
    });

    let project = projects.into_iter().find(|p| p.id == id.to_string());
    match project {
        Some(_) => Ok(web::Json(project)),
        None =>
            Err(NapkinError {
                code: "PROJECT_NO_ID",
                message: "Project with ID {id} Not Found",
                root: &NapkinErrorRoot::NotFound,
            }),
    }
}