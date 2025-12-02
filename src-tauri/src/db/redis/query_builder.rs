use crate::db::traits::QueryBuilder;
use crate::models::table_request::*;
use anyhow::Result;

pub struct RedisQueryBuilder;

impl QueryBuilder for RedisQueryBuilder {
    fn build_select_query(&self, request: &QueryRequest) -> Result<String> {
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
