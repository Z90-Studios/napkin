use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use deadpool_postgres::PoolError;
use derive_more::{Display, From};
use tokio_pg_mapper::Error as PGMError;
use tokio_postgres::error::Error as PGError;
use core::fmt;

pub fn handle_pool_error(x: PoolError) -> NapkinError {
    NapkinError {
        code: "POOL_ERR",
        message: "Unknown Pool Error",
        root: NapkinErrorRoot::PoolError(x),
    }
}

#[derive(Debug, Display, From)]
pub enum NapkinErrorRoot {
    #[display(fmt = "{{ \"error\": \"Not Found\" }}")]
    NotFound,
    PGError(PGError),
    PGMError(PGMError),
    PoolError(PoolError),
}

#[derive(Debug, From)]
// #[display(fmt = "{{ \"code\": \"{}\", \"message\": \"{}\" }}", code, message)]
pub struct NapkinError {
    pub code: &'static str,
    pub message: &'static str,
    pub root: NapkinErrorRoot,
}

impl fmt::Display for NapkinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ \"code\": \"{}\", \"message\": \"{}\", \"root\": \"{}\" }}",
            self.code,
            self.message,
            match &self.root {
                NapkinErrorRoot::NotFound => "NotFound".to_string(),
                NapkinErrorRoot::PGError(ref err) => err.to_string(),
                NapkinErrorRoot::PGMError(ref err) => err.to_string(),
                NapkinErrorRoot::PoolError(ref err) => err.to_string(),
            }
        )
    }
}

impl std::error::Error for NapkinError {}

impl From<PGError> for NapkinError {
    fn from(err: PGError) -> Self {
        NapkinError {
            code: "DB_ERR",
            message: "Database Operation Failed.",
            root: NapkinErrorRoot::PGError(err),
        }
    }
}

impl error::ResponseError for NapkinError {
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();

        let console_output = ::serde_json::json!({
            "code": &self.code,
            "message": &self.message,
            "root": match &self.root {
                NapkinErrorRoot::NotFound => "NotFound".to_string(),
                NapkinErrorRoot::PGError(ref err) => err.to_string(),
                NapkinErrorRoot::PGMError(ref err) => err.to_string(),
                NapkinErrorRoot::PoolError(ref err) => err.to_string(),
            }
        });
        println!(
            "Error Occurred: {}",
            ::serde_json::to_string_pretty(&console_output).unwrap()
        );

        match &self.root {
            NapkinErrorRoot::PoolError(ref err) => HttpResponse::build(status_code)
                .insert_header(ContentType::json())
                .body(err.to_string()),
            _ => HttpResponse::build(status_code)
                .insert_header(ContentType::json())
                .body(self.to_string()),
        }
    }

    fn status_code(&self) -> StatusCode {
        match &self.root {
            NapkinErrorRoot::NotFound => StatusCode::NOT_FOUND,
            NapkinErrorRoot::PoolError(ref _err) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
