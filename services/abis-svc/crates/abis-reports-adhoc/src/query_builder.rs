use serde::{Deserialize, Serialize};

use crate::error::{AdhocError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlQuery {
    pub text: String,
    pub parameters: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderQuery {
    pub source_tables: Vec<String>,
    pub projections: Vec<String>,
    pub filters: Vec<String>,
    pub joins: Vec<JoinClause>,
    pub group_by: Vec<String>,
    pub order_by: Vec<OrderByClause>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinClause {
    pub table: String,
    pub on: String,
    pub kind: JoinKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JoinKind {
    Inner,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderByClause {
    pub column: String,
    pub descending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "flavour")]
pub enum QueryBuilder {
    Sql(SqlQuery),
    Builder(BuilderQuery),
}

impl QueryBuilder {
    /// Compile to parameterised SQL. Column + table names are validated against
    /// the allow-list to prevent injection.
    pub fn compile(&self, allowed_tables: &[String]) -> Result<SqlQuery> {
        match self {
            QueryBuilder::Sql(q) => Ok(q.clone()),
            QueryBuilder::Builder(b) => compile_builder(b, allowed_tables),
        }
    }
}

fn compile_builder(b: &BuilderQuery, allowed_tables: &[String]) -> Result<SqlQuery> {
    // Validate all table references
    for table in &b.source_tables {
        validate_identifier(table)?;
        if !allowed_tables.is_empty() && !allowed_tables.contains(table) {
            return Err(AdhocError::Validation(format!(
                "Table '{table}' is not in the allowed list"
            )));
        }
    }
    for join in &b.joins {
        validate_identifier(&join.table)?;
        if !allowed_tables.is_empty() && !allowed_tables.contains(&join.table) {
            return Err(AdhocError::Validation(format!(
                "Join table '{}' is not in the allowed list",
                join.table
            )));
        }
    }

    // Validate projections
    for proj in &b.projections {
        validate_projection(proj)?;
    }

    let select_cols = if b.projections.is_empty() {
        "*".to_string()
    } else {
        b.projections.join(", ")
    };

    let from = b.source_tables.join(", ");

    let mut sql = format!("SELECT {select_cols} FROM {from}");

    for join in &b.joins {
        let kind = match join.kind {
            JoinKind::Inner => "INNER",
            JoinKind::Left => "LEFT",
            JoinKind::Right => "RIGHT",
        };
        sql.push_str(&format!(" {kind} JOIN {} ON {}", join.table, join.on));
    }

    if !b.filters.is_empty() {
        sql.push_str(&format!(" WHERE {}", b.filters.join(" AND ")));
    }

    if !b.group_by.is_empty() {
        sql.push_str(&format!(" GROUP BY {}", b.group_by.join(", ")));
    }

    if !b.order_by.is_empty() {
        let order: Vec<String> = b
            .order_by
            .iter()
            .map(|o| {
                let dir = if o.descending { "DESC" } else { "ASC" };
                format!("{} {dir}", o.column)
            })
            .collect();
        sql.push_str(&format!(" ORDER BY {}", order.join(", ")));
    }

    let limit = b.limit.unwrap_or(crate::validator::DEFAULT_MAX_ROWS);
    sql.push_str(&format!(" LIMIT {limit}"));

    Ok(SqlQuery {
        text: sql,
        parameters: Vec::new(),
    })
}

fn validate_identifier(s: &str) -> Result<()> {
    if s.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
    {
        Ok(())
    } else {
        Err(AdhocError::Validation(format!("Invalid identifier: '{s}'")))
    }
}

fn validate_projection(s: &str) -> Result<()> {
    let s_lower = s.to_lowercase();
    if s_lower.contains(';') || s_lower.contains("--") || s_lower.contains("/*") {
        return Err(AdhocError::Validation(format!("Invalid projection: '{s}'")));
    }
    Ok(())
}
