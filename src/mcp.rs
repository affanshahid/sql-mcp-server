use rmcp::{
    ErrorData, Json, handler::server::wrapper::Parameters, model::ErrorCode, schemars::JsonSchema,
    tool, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::error;

use crate::{cli::Operation, db::DatabasePool};

#[derive(Deserialize, JsonSchema)]
struct Input {
    query: String,
}

#[derive(Serialize, JsonSchema)]
struct Response {
    output: Vec<Value>,
}

pub struct SqlMcpServer {
    pool: DatabasePool,
    operations: Vec<Operation>,
}

impl SqlMcpServer {
    pub fn new(pool: DatabasePool, operations: Vec<Operation>) -> Self {
        Self { pool, operations }
    }
}

#[tool_router(server_handler)]
impl SqlMcpServer {
    #[tool(description = "Run SQL query")]
    async fn query(
        &self,
        Parameters(input): Parameters<Input>,
    ) -> Result<Json<Response>, ErrorData> {
        let output = self
            .pool
            .query_as_json(&input.query)
            .await
            .inspect_err(|err| error!(%err, "Error running query"))
            .map_err(|err| ErrorData::new(ErrorCode::INTERNAL_ERROR, err.to_string(), None))?;

        Ok(Json(Response { output }))
    }
}
