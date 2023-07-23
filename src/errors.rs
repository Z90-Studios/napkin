use actix_web::{ error, http::{ header::ContentType, StatusCode }, HttpResponse };
use derive_more::{ Display, Error };

#[derive(Debug, Display, Error)]
pub enum NapkinErrorRoot {
    #[display(fmt = "{{ \"error\": \"Not Found\" }}")]
    NotFound,
}

#[derive(Debug, Display, Error)]
#[display(fmt = "{{ \"code\": \"{}\", \"message\": \"{}\" }}", code, message)]
pub struct NapkinError {
    pub code: &'static str,
    pub message: &'static str,
    pub root: &'static NapkinErrorRoot,
}

impl error::ResponseError for NapkinError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self.root {
            NapkinErrorRoot::NotFound => StatusCode::NOT_FOUND,
        }
    }
}
