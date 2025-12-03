use crate::db::mysql::query_builder::MySQLQueryBuilder;
use crate::db::traits::QueryBuilder;
use crate::models::table_request::*;
use anyhow::Result;

pub struct MSSQLQueryBuilder;

impl QueryBuilder for MSSQLQueryBuilder {
    fn build_select_query(&self, request: &QueryRequest) -> Result<String> {
        let table = self.format_table_name(request);

        let order_clause = if request.order_by.is_empty() {
            "ORDER BY (SELECT NULL)".to_string()
        } else {
            format!("ORDER BY {}", self.build_order_by_clause(&request.order_by))
        };

        // Handle SELECT clause with TOP for SQL Server 2008 compatibility
        let select_clause = if request.offset == 0 && request.limit > 0 {
            // SQL Server 2008: use TOP
            format!("SELECT TOP {} *", request.limit)
        } else {
            // SQL Server 2012+ or no limit
            "SELECT *".to_string()
        };

        let mut query = format!("{} FROM {}", select_clause, table);

        if !request.filters.is_empty() {
            let where_clause = self.build_where_clause(&request.filters)?;
            query.push_str(&format!(" WHERE {}", where_clause));
        }

        query.push_str(&format!(" {}", order_clause));

        // Add OFFSET...FETCH only if offset > 0 (SQL Server 2012+)
        if request.offset > 0 {
            query.push_str(&format!(
                " OFFSET {} ROWS FETCH NEXT {} ROWS ONLY",
                request.offset, request.limit
            ));
        }

        Ok(query)
    }

    fn quote_identifier(&self, identifier: &str) -> String {
        format!("[{}]", identifier.replace("]", "]]"))
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

        match (&request.database, &request.schema) {
            (Some(db), Some(schema)) => format!(
                "{}.{}.{}",
                self.quote_identifier(db),
                self.quote_identifier(schema),
                quoted_table
            ),
            (None, Some(schema)) => format!("{}.{}", self.quote_identifier(schema), quoted_table),
            (Some(db), None) => format!("{}.dbo.{}", self.quote_identifier(db), quoted_table),
            (None, None) => quoted_table,
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
        if offset > 0 {
            // SQL Server 2012+: use OFFSET...FETCH
            format!("OFFSET {} ROWS FETCH NEXT {} ROWS ONLY", offset, limit)
        } else {
            // Handled in build_select_query for SQL Server 2008 compatibility
            "".to_string()
        }
    }
}
