use crate::models::connection::DatabaseType;
use crate::models::table_request::*;
use anyhow::Result;

/// Trait for building SQL queries from JSON request structure
pub trait QueryBuilder {
    /// Build a SELECT query with filters, sorting, and pagination
    fn build_select_query(&self, request: &QueryRequest) -> Result<String>;

    /// Quote identifier (table, column, schema names) appropriately for the database
    fn quote_identifier(&self, identifier: &str) -> String;

    /// Format the table name with database and schema if provided
    fn format_table_name(&self, request: &QueryRequest) -> String;

    /// Build WHERE clause from filters
    fn build_where_clause(&self, filters: &[Filter]) -> Result<String>;

    /// Build ORDER BY clause
    fn build_order_by_clause(&self, order_by: &[OrderBy]) -> String;

    /// Build LIMIT and OFFSET clause
    fn build_pagination_clause(&self, limit: usize, offset: usize) -> String;
}

/// Get appropriate query builder for database type
pub fn get_query_builder(db_type: &DatabaseType) -> Box<dyn QueryBuilder> {
    match db_type {
        DatabaseType::MySQL => Box::new(MySQLQueryBuilder),
        DatabaseType::PostgreSQL => Box::new(PostgreSQLQueryBuilder),
        DatabaseType::MSSQL => Box::new(MSSQLQueryBuilder),
        DatabaseType::MongoDB => Box::new(MongoDBQueryBuilder),
        DatabaseType::Redis => Box::new(RedisQueryBuilder),
        DatabaseType::Ignite => Box::new(IgniteQueryBuilder),
    }
}

/// MySQL Query Builder
pub struct MySQLQueryBuilder;

impl QueryBuilder for MySQLQueryBuilder {
    fn build_select_query(&self, request: &QueryRequest) -> Result<String> {
        let table = self.format_table_name(request);
        let mut query = format!("SELECT * FROM {}", table);

        if !request.filters.is_empty() {
            let where_clause = self.build_where_clause(&request.filters)?;
            query.push_str(&format!(" WHERE {}", where_clause));
        }

        if !request.order_by.is_empty() {
            let order_clause = self.build_order_by_clause(&request.order_by);
            query.push_str(&format!(" ORDER BY {}", order_clause));
        }

        let pagination = self.build_pagination_clause(request.limit, request.offset);
        query.push_str(&format!(" {}", pagination));

        Ok(query)
    }

    fn quote_identifier(&self, identifier: &str) -> String {
        format!("`{}`", identifier.replace("`", "``"))
    }

    fn format_table_name(&self, request: &QueryRequest) -> String {
        let table = self.quote_identifier(&request.table);

        if let Some(db) = &request.database {
            format!("{}.{}", self.quote_identifier(db), table)
        } else {
            table
        }
    }

    fn build_where_clause(&self, filters: &[Filter]) -> Result<String> {
        let conditions: Vec<String> = filters
            .iter()
            .map(|f| self.build_filter_condition(f))
            .collect::<Result<Vec<_>>>()?;

        Ok(conditions.join(" AND "))
    }

    fn build_order_by_clause(&self, order_by: &[OrderBy]) -> String {
        order_by
            .iter()
            .map(|o| {
                let direction = match o.direction {
                    SortDirection::Asc => "ASC",
                    SortDirection::Desc => "DESC",
                };
                format!("{} {}", self.quote_identifier(&o.column), direction)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn build_pagination_clause(&self, limit: usize, offset: usize) -> String {
        format!("LIMIT {} OFFSET {}", limit, offset)
    }
}

impl MySQLQueryBuilder {
    fn build_filter_condition(&self, filter: &Filter) -> Result<String> {
        let column = self.quote_identifier(&filter.column);

        let condition = match &filter.operator {
            FilterOperator::Equals => match &filter.value {
                FilterValue::Single(v) => format!("{} = {}", column, self.format_value(v)),
                _ => anyhow::bail!("Equals operator requires single value"),
            },
            FilterOperator::NotEquals => match &filter.value {
                FilterValue::Single(v) => format!("{} != {}", column, self.format_value(v)),
                _ => anyhow::bail!("NotEquals operator requires single value"),
            },
            FilterOperator::In => match &filter.value {
                FilterValue::Multiple(values) => {
                    let values_str = values
                        .iter()
                        .map(|v| self.format_value(v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{} IN ({})", column, values_str)
                }
                _ => anyhow::bail!("In operator requires multiple values"),
            },
            FilterOperator::NotIn => match &filter.value {
                FilterValue::Multiple(values) => {
                    let values_str = values
                        .iter()
                        .map(|v| self.format_value(v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{} NOT IN ({})", column, values_str)
                }
                _ => anyhow::bail!("NotIn operator requires multiple values"),
            },
            FilterOperator::Like => match &filter.value {
                FilterValue::Single(v) => {
                    format!("{} LIKE {}", column, self.format_value(v))
                }
                _ => anyhow::bail!("Like operator requires single value"),
            },
            FilterOperator::NotLike => match &filter.value {
                FilterValue::Single(v) => {
                    format!("{} NOT LIKE {}", column, self.format_value(v))
                }
                _ => anyhow::bail!("NotLike operator requires single value"),
            },
            FilterOperator::GreaterThan => match &filter.value {
                FilterValue::Single(v) => format!("{} > {}", column, self.format_value(v)),
                _ => anyhow::bail!("GreaterThan operator requires single value"),
            },
            FilterOperator::GreaterThanOrEqual => match &filter.value {
                FilterValue::Single(v) => format!("{} >= {}", column, self.format_value(v)),
                _ => anyhow::bail!("GreaterThanOrEqual operator requires single value"),
            },
            FilterOperator::LessThan => match &filter.value {
                FilterValue::Single(v) => format!("{} < {}", column, self.format_value(v)),
                _ => anyhow::bail!("LessThan operator requires single value"),
            },
            FilterOperator::LessThanOrEqual => match &filter.value {
                FilterValue::Single(v) => format!("{} <= {}", column, self.format_value(v)),
                _ => anyhow::bail!("LessThanOrEqual operator requires single value"),
            },
            FilterOperator::Between => match &filter.value {
                FilterValue::Range { from, to } => {
                    format!(
                        "{} BETWEEN {} AND {}",
                        column,
                        self.format_value(from),
                        self.format_value(to)
                    )
                }
                _ => anyhow::bail!("Between operator requires range value"),
            },
            FilterOperator::IsNull => format!("{} IS NULL", column),
            FilterOperator::IsNotNull => format!("{} IS NOT NULL", column),
        };

        Ok(condition)
    }

    fn format_value(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "NULL".to_string(),
            serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            _ => format!("'{}'", value.to_string().replace("'", "''")),
        }
    }
}

/// PostgreSQL Query Builder
pub struct PostgreSQLQueryBuilder;

impl QueryBuilder for PostgreSQLQueryBuilder {
    fn build_select_query(&self, request: &QueryRequest) -> Result<String> {
        let table = self.format_table_name(request);
        let mut query = format!("SELECT * FROM {}", table);

        if !request.filters.is_empty() {
            let where_clause = self.build_where_clause(&request.filters)?;
            query.push_str(&format!(" WHERE {}", where_clause));
        }

        if !request.order_by.is_empty() {
            let order_clause = self.build_order_by_clause(&request.order_by);
            query.push_str(&format!(" ORDER BY {}", order_clause));
        }

        let pagination = self.build_pagination_clause(request.limit, request.offset);
        query.push_str(&format!(" {}", pagination));

        Ok(query)
    }

    fn quote_identifier(&self, identifier: &str) -> String {
        format!("\"{}\"", identifier.replace("\"", "\"\""))
    }

    fn format_table_name(&self, request: &QueryRequest) -> String {
        let table = self.quote_identifier(&request.table);

        match (&request.database, &request.schema) {
            (Some(db), Some(schema)) => format!(
                "{}.{}.{}",
                self.quote_identifier(db),
                self.quote_identifier(schema),
                table
            ),
            (None, Some(schema)) => format!("{}.{}", self.quote_identifier(schema), table),
            (Some(db), None) => format!("{}.{}", self.quote_identifier(db), table),
            (None, None) => table,
        }
    }

    fn build_where_clause(&self, filters: &[Filter]) -> Result<String> {
        let builder = MySQLQueryBuilder; // Reuse MySQL logic with PostgreSQL quoting
        let conditions: Vec<String> = filters
            .iter()
            .map(|f| {
                let column = self.quote_identifier(&f.column);
                let mut filter_copy = f.clone();
                filter_copy.column = column;
                builder.build_filter_condition(&filter_copy)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(conditions.join(" AND "))
    }

    fn build_order_by_clause(&self, order_by: &[OrderBy]) -> String {
        order_by
            .iter()
            .map(|o| {
                let direction = match o.direction {
                    SortDirection::Asc => "ASC",
                    SortDirection::Desc => "DESC",
                };
                format!("{} {}", self.quote_identifier(&o.column), direction)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn build_pagination_clause(&self, limit: usize, offset: usize) -> String {
        format!("LIMIT {} OFFSET {}", limit, offset)
    }
}

/// MSSQL Query Builder
pub struct MSSQLQueryBuilder;

impl QueryBuilder for MSSQLQueryBuilder {
    fn build_select_query(&self, request: &QueryRequest) -> Result<String> {
        let table = self.format_table_name(request);

        // MSSQL uses OFFSET-FETCH which requires ORDER BY
        let order_clause = if request.order_by.is_empty() {
            // Default ordering by first column if no order specified
            "ORDER BY (SELECT NULL)".to_string()
        } else {
            format!("ORDER BY {}", self.build_order_by_clause(&request.order_by))
        };

        let mut query = format!("SELECT * FROM {}", table);

        if !request.filters.is_empty() {
            let where_clause = self.build_where_clause(&request.filters)?;
            query.push_str(&format!(" WHERE {}", where_clause));
        }

        query.push_str(&format!(" {}", order_clause));

        let pagination = self.build_pagination_clause(request.limit, request.offset);
        query.push_str(&format!(" {}", pagination));

        Ok(query)
    }

    fn quote_identifier(&self, identifier: &str) -> String {
        format!("[{}]", identifier.replace("]", "]]"))
    }

    fn format_table_name(&self, request: &QueryRequest) -> String {
        let table = self.quote_identifier(&request.table);

        match (&request.database, &request.schema) {
            (Some(db), Some(schema)) => format!(
                "{}.{}.{}",
                self.quote_identifier(db),
                self.quote_identifier(schema),
                table
            ),
            (None, Some(schema)) => format!("{}.{}", self.quote_identifier(schema), table),
            (Some(db), None) => format!("{}.dbo.{}", self.quote_identifier(db), table),
            (None, None) => table,
        }
    }

    fn build_where_clause(&self, filters: &[Filter]) -> Result<String> {
        let builder = MySQLQueryBuilder;
        let conditions: Vec<String> = filters
            .iter()
            .map(|f| {
                let column = self.quote_identifier(&f.column);
                let mut filter_copy = f.clone();
                filter_copy.column = column;
                builder.build_filter_condition(&filter_copy)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(conditions.join(" AND "))
    }

    fn build_order_by_clause(&self, order_by: &[OrderBy]) -> String {
        order_by
            .iter()
            .map(|o| {
                let direction = match o.direction {
                    SortDirection::Asc => "ASC",
                    SortDirection::Desc => "DESC",
                };
                format!("{} {}", self.quote_identifier(&o.column), direction)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn build_pagination_clause(&self, limit: usize, offset: usize) -> String {
        format!("OFFSET {} ROWS FETCH NEXT {} ROWS ONLY", offset, limit)
    }
}

/// MongoDB Query Builder (converts to aggregation pipeline JSON)
pub struct MongoDBQueryBuilder;

impl QueryBuilder for MongoDBQueryBuilder {
    fn build_select_query(&self, _request: &QueryRequest) -> Result<String> {
        // MongoDB doesn't use SQL, this returns a JSON aggregation pipeline as string
        // Actual implementation will be in the MongoDB connection handler
        Ok("{}".to_string())
    }

    fn quote_identifier(&self, identifier: &str) -> String {
        identifier.to_string()
    }

    fn format_table_name(&self, request: &QueryRequest) -> String {
        request.table.clone()
    }

    fn build_where_clause(&self, _filters: &[Filter]) -> Result<String> {
        Ok("{}".to_string())
    }

    fn build_order_by_clause(&self, _order_by: &[OrderBy]) -> String {
        "{}".to_string()
    }

    fn build_pagination_clause(&self, _limit: usize, _offset: usize) -> String {
        "".to_string()
    }
}

/// Redis Query Builder (for SCAN operations)
pub struct RedisQueryBuilder;

impl QueryBuilder for RedisQueryBuilder {
    fn build_select_query(&self, request: &QueryRequest) -> Result<String> {
        // Redis uses SCAN command
        Ok(format!("SCAN 0 MATCH {}:*", request.table))
    }

    fn quote_identifier(&self, identifier: &str) -> String {
        identifier.to_string()
    }

    fn format_table_name(&self, request: &QueryRequest) -> String {
        request.table.clone()
    }

    fn build_where_clause(&self, _filters: &[Filter]) -> Result<String> {
        Ok("".to_string())
    }

    fn build_order_by_clause(&self, _order_by: &[OrderBy]) -> String {
        "".to_string()
    }

    fn build_pagination_clause(&self, _limit: usize, _offset: usize) -> String {
        "".to_string()
    }
}

/// Apache Ignite Query Builder
pub struct IgniteQueryBuilder;

impl QueryBuilder for IgniteQueryBuilder {
    fn build_select_query(&self, request: &QueryRequest) -> Result<String> {
        let table = self.format_table_name(request);
        let mut query = format!("SELECT * FROM {}", table);

        if !request.filters.is_empty() {
            let where_clause = self.build_where_clause(&request.filters)?;
            query.push_str(&format!(" WHERE {}", where_clause));
        }

        if !request.order_by.is_empty() {
            let order_clause = self.build_order_by_clause(&request.order_by);
            query.push_str(&format!(" ORDER BY {}", order_clause));
        }

        let pagination = self.build_pagination_clause(request.limit, request.offset);
        query.push_str(&format!(" {}", pagination));

        Ok(query)
    }

    fn quote_identifier(&self, identifier: &str) -> String {
        format!("\"{}\"", identifier.replace("\"", "\"\""))
    }

    fn format_table_name(&self, request: &QueryRequest) -> String {
        let table = self.quote_identifier(&request.table);

        if let Some(schema) = &request.schema {
            format!("{}.{}", self.quote_identifier(schema), table)
        } else {
            table
        }
    }

    fn build_where_clause(&self, filters: &[Filter]) -> Result<String> {
        let builder = MySQLQueryBuilder;
        let conditions: Vec<String> = filters
            .iter()
            .map(|f| {
                let column = self.quote_identifier(&f.column);
                let mut filter_copy = f.clone();
                filter_copy.column = column;
                builder.build_filter_condition(&filter_copy)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(conditions.join(" AND "))
    }

    fn build_order_by_clause(&self, order_by: &[OrderBy]) -> String {
        order_by
            .iter()
            .map(|o| {
                let direction = match o.direction {
                    SortDirection::Asc => "ASC",
                    SortDirection::Desc => "DESC",
                };
                format!("{} {}", self.quote_identifier(&o.column), direction)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn build_pagination_clause(&self, limit: usize, offset: usize) -> String {
        format!("LIMIT {} OFFSET {}", limit, offset)
    }
}
