//! Protocol types shared between concourse and inboard-relay.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An action for the relay to execute against a database connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Execute a SQL query and return results.
    Query { sql: String },
    /// List all tables in the database.
    ListTables,
    /// Describe a single table's columns.
    DescribeTable { table: String },
    /// Return the full schema for a table.
    TableSchema { table: String },
}

/// A job dispatched to a relay for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique request identifier for correlating results.
    pub request_id: Uuid,
    /// The connection to execute against.
    pub connection_id: String,
    /// The action to perform.
    pub action: Action,
}

/// Status of a completed job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Success,
    Error,
}

/// A column definition returned from introspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub primary_key: bool,
}

/// Result returned by a relay after executing a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Result {
    pub request_id: Uuid,
    pub status: Status,
    /// Column names for query results.
    #[serde(default)]
    pub columns: Vec<String>,
    /// Rows of values for query results.
    #[serde(default)]
    pub rows: Vec<Vec<serde_json::Value>>,
    /// Column definitions for introspection results.
    #[serde(default)]
    pub column_defs: Vec<ColumnDef>,
    /// Table names for list_tables results.
    #[serde(default)]
    pub tables: Vec<String>,
    /// Error message if status is Error.
    #[serde(default)]
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn action_query_round_trip() {
        let action = Action::Query {
            sql: "SELECT 1".into(),
        };
        let json = serde_json::to_string(&action).unwrap();
        let parsed: Action = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, Action::Query { sql } if sql == "SELECT 1"));
    }

    #[test]
    fn action_query_serializes_with_type_tag() {
        let action = Action::Query {
            sql: "SELECT 1".into(),
        };
        let value: serde_json::Value = serde_json::to_value(&action).unwrap();
        assert_eq!(value["type"], "query");
        assert_eq!(value["sql"], "SELECT 1");
    }

    #[test]
    fn job_round_trip() {
        let job = Job {
            request_id: Uuid::nil(),
            connection_id: "conn-1".into(),
            action: Action::ListTables,
        };
        let json = serde_json::to_string(&job).unwrap();
        let parsed: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.request_id, Uuid::nil());
        assert_eq!(parsed.connection_id, "conn-1");
        assert!(matches!(parsed.action, Action::ListTables));
    }

    #[test]
    fn result_round_trip() {
        let result = Result {
            request_id: Uuid::nil(),
            status: Status::Success,
            columns: vec!["id".into()],
            rows: vec![vec![serde_json::json!(1)]],
            column_defs: vec![],
            tables: vec![],
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: Result = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.columns, vec!["id"]);
        assert_eq!(parsed.rows.len(), 1);
    }

    #[test]
    fn result_defaults_missing_fields() {
        let json = r#"{"request_id":"00000000-0000-0000-0000-000000000000","status":"success"}"#;
        let result: Result = serde_json::from_str(json).unwrap();
        assert!(result.columns.is_empty());
        assert!(result.rows.is_empty());
        assert!(result.column_defs.is_empty());
        assert!(result.tables.is_empty());
        assert!(result.error.is_none());
    }

    #[test]
    fn status_serializes_as_snake_case() {
        let json = serde_json::to_string(&Status::Success).unwrap();
        assert_eq!(json, r#""success""#);
        let json = serde_json::to_string(&Status::Error).unwrap();
        assert_eq!(json, r#""error""#);
    }
}
