use serde::{Deserialize, Serialize};
use sqlparser::ast::{Expr, ObjectName, SelectItem, SetExpr, Statement, TableFactor};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

use crate::error::{AdhocError, Result};

/// Default LIMIT injected when the query has none.
pub const DEFAULT_MAX_ROWS: u64 = 10_000;

/// Functions that must never appear in ad-hoc queries.
pub const BLOCKED_FUNCTIONS: &[&str] = &[
    "pg_read_file",
    "pg_read_binary_file",
    "pg_ls_dir",
    "pg_stat_file",
    "pg_sleep",
    "dblink",
    "dblink_exec",
    "lo_import",
    "lo_export",
    "lo_create",
    "lo_unlink",
    "pg_stat_reset",
    "pg_reload_conf",
    "xpath",
    "xmlparse",
    "xmlserialize",
    "copy",
    "pg_execute_server_program",
    "pg_write_file",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintReport {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    pub safety_score: f32,
    pub modified_sql: Option<String>,
}

impl LintReport {
    pub fn is_safe(&self) -> bool {
        self.errors.is_empty()
    }
}

struct SqlVisitor {
    blocked_fns: Vec<String>,
    referenced_tables: Vec<String>,
    has_limit: bool,
    join_depth: usize,
    has_aggregates: bool,
    errors: Vec<Diagnostic>,
    warnings: Vec<Diagnostic>,
}

impl SqlVisitor {
    fn new() -> Self {
        Self {
            blocked_fns: Vec::new(),
            referenced_tables: Vec::new(),
            has_limit: false,
            join_depth: 0,
            has_aggregates: false,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Validate a SQL string against the allow-list and safety rules.
pub fn validate_sql(
    sql: &str,
    allowed_tables: &[String],
    max_rows: Option<u64>,
) -> Result<LintReport> {
    let dialect = PostgreSqlDialect {};
    let parse_result = Parser::parse_sql(&dialect, sql);

    let mut report = LintReport {
        errors: Vec::new(),
        warnings: Vec::new(),
        safety_score: 1.0,
        modified_sql: None,
    };

    let statements = match parse_result {
        Ok(stmts) => stmts,
        Err(e) => {
            report.errors.push(Diagnostic {
                code: "E003".to_string(),
                message: format!("Statement not allowed or could not be parsed: {e}"),
                severity: DiagnosticSeverity::Error,
            });
            report.safety_score = 0.0;
            return Ok(report);
        }
    };

    if statements.is_empty() {
        report.errors.push(Diagnostic {
            code: "E001".to_string(),
            message: "No SQL statement found".to_string(),
            severity: DiagnosticSeverity::Error,
        });
        report.safety_score = 0.0;
        return Ok(report);
    }

    if statements.len() > 1 {
        report.errors.push(Diagnostic {
            code: "E002".to_string(),
            message: "Only one statement is allowed".to_string(),
            severity: DiagnosticSeverity::Error,
        });
    }

    let stmt = &statements[0];

    // Check statement type
    let is_select = matches!(stmt, Statement::Query(_));
    if !is_select {
        let stmt_str = stmt.to_string().to_lowercase();
        let first_word = stmt_str.split_whitespace().next().unwrap_or("");
        report.errors.push(Diagnostic {
            code: "E003".to_string(),
            message: format!(
                "Only SELECT (or WITH ... SELECT) statements are allowed; got '{first_word}'"
            ),
            severity: DiagnosticSeverity::Error,
        });
        report.safety_score = 0.0;
        return Ok(report);
    }

    // Check blocked functions in raw SQL (case-insensitive)
    let sql_lower = sql.to_lowercase();
    for fn_name in BLOCKED_FUNCTIONS {
        if sql_lower.contains(&format!("{fn_name}(")) || sql_lower.contains(&format!("{fn_name} ("))
        {
            report.errors.push(Diagnostic {
                code: "E004".to_string(),
                message: format!("Blocked function '{fn_name}' is not allowed"),
                severity: DiagnosticSeverity::Error,
            });
        }
    }

    // Extract table references and other info from the query
    if let Statement::Query(q) = stmt {
        let mut visitor = SqlVisitor::new();
        extract_query_info(q, &mut visitor, 0);

        // Check table references against allow-list
        if !allowed_tables.is_empty() {
            for table in &visitor.referenced_tables {
                let table_lower = table.to_lowercase();
                if !allowed_tables
                    .iter()
                    .any(|t| t.to_lowercase() == table_lower)
                {
                    report.errors.push(Diagnostic {
                        code: "E005".to_string(),
                        message: format!("Table '{table}' is not in the allowed tables list"),
                        severity: DiagnosticSeverity::Error,
                    });
                }
            }
        }

        // System catalog rejection
        for table in &visitor.referenced_tables {
            let t = table.to_lowercase();
            if t.starts_with("pg_") || t.starts_with("information_schema") {
                report.errors.push(Diagnostic {
                    code: "E006".to_string(),
                    message: format!("Access to system catalog '{table}' is not allowed"),
                    severity: DiagnosticSeverity::Error,
                });
            }
        }

        // Blocked functions from visitor
        for fn_name in &visitor.blocked_fns {
            report.errors.push(Diagnostic {
                code: "E004".to_string(),
                message: format!("Blocked function '{fn_name}' is not allowed"),
                severity: DiagnosticSeverity::Error,
            });
        }

        // LIMIT injection
        let limit = max_rows.unwrap_or(DEFAULT_MAX_ROWS);
        let has_limit = visitor.has_limit;
        if !has_limit {
            report.warnings.push(Diagnostic {
                code: "W001".to_string(),
                message: format!("No LIMIT clause found; {limit} rows will be enforced"),
                severity: DiagnosticSeverity::Warning,
            });
            let modified = format!("SELECT * FROM ({sql}) __eel_adhoc_wrap LIMIT {limit}");
            report.modified_sql = Some(modified);
        }

        // Safety score computation
        let mut score = 1.0f32;

        if visitor.join_depth > 3 {
            score -= 0.2;
            report.warnings.push(Diagnostic {
                code: "W002".to_string(),
                message: format!(
                    "High join depth ({}); consider simplifying",
                    visitor.join_depth
                ),
                severity: DiagnosticSeverity::Warning,
            });
        } else if visitor.join_depth > 1 {
            score -= 0.05 * visitor.join_depth as f32;
        }

        if visitor.referenced_tables.len() > 5 {
            score -= 0.1;
        }

        if visitor.has_aggregates {
            score -= 0.02;
        }

        let blocked_count = report.errors.iter().filter(|e| e.code == "E004").count();
        score -= blocked_count as f32 * 0.3;

        if !has_limit {
            score -= 0.1;
        }

        score -= report.errors.len() as f32 * 0.2;

        report.safety_score = score.clamp(0.0, 1.0);
    }

    Ok(report)
}

fn extract_query_info(query: &sqlparser::ast::Query, visitor: &mut SqlVisitor, depth: usize) {
    if query.limit.is_some() {
        visitor.has_limit = true;
    }

    extract_set_expr_info(&query.body, visitor, depth);
}

fn extract_set_expr_info(body: &SetExpr, visitor: &mut SqlVisitor, depth: usize) {
    match body {
        SetExpr::Select(select) => {
            for table_with_joins in &select.from {
                extract_table_factor_info(&table_with_joins.relation, visitor, depth);
                for join in &table_with_joins.joins {
                    visitor.join_depth += 1;
                    extract_table_factor_info(&join.relation, visitor, depth);
                }
            }

            for item in &select.projection {
                if let SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } = item
                {
                    if is_aggregate_expr(expr) {
                        visitor.has_aggregates = true;
                    }
                    check_expr_for_blocked_fns(expr, visitor);
                }
            }

            if let Some(selection) = &select.selection {
                check_expr_for_blocked_fns(selection, visitor);
            }
        }
        SetExpr::SetOperation { left, right, .. } => {
            extract_set_expr_info(left, visitor, depth);
            extract_set_expr_info(right, visitor, depth);
        }
        SetExpr::Query(subq) => {
            extract_query_info(subq, visitor, depth + 1);
        }
        _ => {}
    }
}

fn extract_table_factor_info(factor: &TableFactor, visitor: &mut SqlVisitor, depth: usize) {
    match factor {
        TableFactor::Table { name, .. } => {
            visitor.referenced_tables.push(object_name_to_str(name));
        }
        TableFactor::Derived { subquery, .. } => {
            extract_query_info(subquery, visitor, depth + 1);
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => {
            extract_table_factor_info(&table_with_joins.relation, visitor, depth);
            for join in &table_with_joins.joins {
                visitor.join_depth += 1;
                extract_table_factor_info(&join.relation, visitor, depth);
            }
        }
        _ => {}
    }
}

fn object_name_to_str(name: &ObjectName) -> String {
    name.0
        .iter()
        .map(|p| p.value.clone())
        .collect::<Vec<_>>()
        .join(".")
}

fn is_aggregate_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Function(f) => {
            let name = f
                .name
                .0
                .last()
                .map(|p| p.value.to_lowercase())
                .unwrap_or_default();
            matches!(
                name.as_str(),
                "count" | "sum" | "avg" | "max" | "min" | "stddev" | "variance"
            )
        }
        _ => false,
    }
}

fn check_expr_for_blocked_fns(expr: &Expr, visitor: &mut SqlVisitor) {
    if let Expr::Function(f) = expr {
        let name = f
            .name
            .0
            .last()
            .map(|p| p.value.to_lowercase())
            .unwrap_or_default();
        if BLOCKED_FUNCTIONS.contains(&name.as_str()) && !visitor.blocked_fns.contains(&name) {
            visitor.blocked_fns.push(name);
        }
    }
}
