use crate::common::{
    constants::{DEFAULT_PAGE, DEFAULT_PER_PAGE},
    response::extract_request_id,
};
use actix_web::HttpRequest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PaginatedQuery<S, O> {
    #[serde(default = "default_page")]
    pub page: u64,

    #[serde(default = "default_per_page")]
    pub per_page: u64,

    pub search: Option<String>,
    pub sort_by: Option<S>,
    pub sort_order: Option<O>,
}

impl<S, O> Default for PaginatedQuery<S, O> {
    fn default() -> Self {
        Self {
            page: DEFAULT_PAGE,
            per_page: DEFAULT_PER_PAGE,
            search: None,
            sort_by: None,
            sort_order: None,
        }
    }
}

fn default_page() -> u64 {
    DEFAULT_PAGE
}

fn default_per_page() -> u64 {
    DEFAULT_PER_PAGE
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: u64,
    pub per_page: u64,
    pub total_items: u64,
    pub total_pages: u64,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
    pub meta: super::response::Meta,
}

impl<T> PaginatedResponse<T>
where
    T: Serialize,
{
    pub fn new(
        data: Vec<T>,
        page: u64,
        per_page: u64,
        total_items: u64,
        request: &HttpRequest,
    ) -> Self {
        let total_pages = if per_page == 0 {
            0
        } else {
            total_items.div_ceil(per_page)
        };

        Self {
            success: true,
            data,
            pagination: PaginationMeta {
                page,
                per_page,
                total_items,
                total_pages,
                has_next: page < total_pages,
                has_prev: page > 1,
            },
            meta: super::response::Meta::new(extract_request_id(request)),
        }
    }
}
