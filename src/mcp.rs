use rmcp::{
    ErrorData, Json, handler::server::wrapper::Parameters, model::ErrorCode, schemars::JsonSchema,
    tool, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::error;

use crate::{
    cli::Permissions,
    db::{DatabasePool, OperationInfo},
};

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
    permissions: Permissions,
}

impl SqlMcpServer {
    pub fn new(pool: DatabasePool, permissions: Permissions) -> Self {
        Self { pool, permissions }
    }
}

#[tool_router(server_handler)]
impl SqlMcpServer {
    #[tool(description = "Run SQL query")]
    async fn query(
        &self,
        Parameters(input): Parameters<Input>,
    ) -> Result<Json<Response>, ErrorData> {
        let parsed_operations = self
            .pool
            .parse_operations(&input.query)
            .inspect_err(|err| error!(%err, "Error parsing query"))
            .map_err(|err| ErrorData::new(ErrorCode::PARSE_ERROR, err.to_string(), None))?;

        let is_allowed = parsed_operations
            .iter()
            .all(|o| self.permissions.operations.contains(&o.operation()));

        if !is_allowed {
            return Err(ErrorData::new(
                ErrorCode::INVALID_PARAMS,
                "Query contains operations disallowed by configured permissions",
                None,
            ));
        }

        parsed_operations.into_iter().try_for_each(|o| match o {
            OperationInfo::Select { has_limit }
                if self.permissions.deny_limitless_select && !has_limit =>
            {
                Err(ErrorData::new(
                    ErrorCode::INVALID_PARAMS,
                    "Configured permission do not allow SELECT queries without a LIMIT clause",
                    None,
                ))
            }
            OperationInfo::Update { has_where }
                if self.permissions.deny_boundless_update && !has_where =>
            {
                Err(ErrorData::new(
                    ErrorCode::INVALID_PARAMS,
                    "Configured permission do not allow UPDATE queries without a WHERE clause",
                    None,
                ))
            }
            OperationInfo::Delete { has_where }
                if self.permissions.deny_boundless_delete && !has_where =>
            {
                Err(ErrorData::new(
                    ErrorCode::INVALID_PARAMS,
                    "Configured permission do not allow DELETE queries without a WHERE clause",
                    None,
                ))
            }
            _ => Ok(()),
        })?;

        let output = self
            .pool
            .query_as_json(&input.query)
            .await
            .inspect_err(|err| error!(%err, "Error running query"))
            .map_err(|err| ErrorData::new(ErrorCode::INTERNAL_ERROR, err.to_string(), None))?;

        Ok(Json(Response { output }))
    }
}
