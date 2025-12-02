use crate::db::traits::QueryBuilder;
use crate::db::mysql::query_builder::MySQLQueryBuilder;
use crate::models::table_request::*;
use anyhow::Result;

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
