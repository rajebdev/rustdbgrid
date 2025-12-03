use crate::db::traits::QueryBuilder;
use crate::models::table_request::*;
use anyhow::Result;

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
        let table = &request.table;
        
        // If table is wrapped with RustDBGridQuery(), extract and return the inner query
        let trimmed = table.trim();
        if trimmed.starts_with("RustDBGridQuery(") && trimmed.ends_with(')') {
            let query = &trimmed[16..trimmed.len()-1]; // Extract between RustDBGridQuery( and )
            return format!("({}) AS __query", query);
        }
        
        // Normal table - quote it
        let quoted_table = self.quote_identifier(table);

        if let Some(db) = &request.database {
            format!("{}.{}", self.quote_identifier(db), quoted_table)
        } else {
            quoted_table
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
    pub fn build_filter_condition(&self, filter: &Filter) -> Result<String> {
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

    pub fn format_value(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "NULL".to_string(),
            serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            _ => format!("'{}'", value.to_string().replace("'", "''")),
        }
    }
}
