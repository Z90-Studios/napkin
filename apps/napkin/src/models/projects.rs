use serde::{Serialize, Deserialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "projects")]
pub struct Project {
    pub id: Option<String>,
    pub scope: String,
    pub name: String,
}

impl Project {
    pub fn to_update_str(&self) -> String {
        let update = "SET scope = $scope, name = $name";
        let update = update.replace("$scope", &self.scope);
        let update = update.replace("$name", &self.name);

        update
    }
}