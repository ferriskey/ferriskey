use std::num::NonZeroU32;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::app_errors::CoreError;

pub const DEFAULT_PAGE_LIMIT: u32 = 20;
pub const MAX_PAGE_LIMIT: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageNumber(NonZeroU32);

impl PageNumber {
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl Default for PageNumber {
    fn default() -> Self {
        Self(NonZeroU32::MIN)
    }
}

impl TryFrom<u32> for PageNumber {
    type Error = CoreError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        NonZeroU32::new(value)
            .map(Self)
            .ok_or_else(|| CoreError::InvalidPagination("page must be at least 1".to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageLimit(u32);

impl PageLimit {
    pub fn get(self) -> u32 {
        self.0
    }
}

impl Default for PageLimit {
    fn default() -> Self {
        Self(DEFAULT_PAGE_LIMIT)
    }
}

impl TryFrom<u32> for PageLimit {
    type Error = CoreError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if (1..=MAX_PAGE_LIMIT).contains(&value) {
            Ok(Self(value))
        } else {
            Err(CoreError::InvalidPagination(format!(
                "limit must be between 1 and {MAX_PAGE_LIMIT}"
            )))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sort<S> {
    pub field: S,
    pub order: SortOrder,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PageRequest<F, S> {
    pub page: PageNumber,
    pub limit: PageLimit,
    pub sort: Sort<S>,
    pub filter: F,
}

impl<F, S> PageRequest<F, S> {
    pub fn offset(&self) -> u64 {
        u64::from(self.page.get() - 1) * u64::from(self.limit.get())
    }

    pub fn map_filter<G>(self, f: impl FnOnce(F) -> G) -> PageRequest<G, S> {
        PageRequest {
            page: self.page,
            limit: self.limit,
            sort: self.sort,
            filter: f(self.filter),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PageMetadata {
    pub page: u64,
    pub limit: u64,
    pub total: u64,
    pub total_pages: u64,
    pub first_page: u64,
    pub last_page: u64,
    pub next_page: Option<u64>,
    pub prev_page: Option<u64>,
}

impl PageMetadata {
    fn compute(total: u64, page: PageNumber, limit: PageLimit) -> Self {
        let page = u64::from(page.get());
        let limit = u64::from(limit.get());
        let total_pages = total.div_ceil(limit);
        let last_page = total_pages.max(1);
        Self {
            page,
            limit,
            total,
            total_pages,
            first_page: 1,
            last_page,
            next_page: (page < last_page).then_some(page + 1),
            prev_page: (page > 1).then_some(page - 1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct Page<T> {
    data: Vec<T>,
    metadata: PageMetadata,
}

impl<T> Page<T> {
    pub fn new(data: Vec<T>, total: u64, page: PageNumber, limit: PageLimit) -> Self {
        Self {
            data,
            metadata: PageMetadata::compute(total, page, limit),
        }
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn metadata(&self) -> PageMetadata {
        self.metadata
    }

    pub fn into_parts(self) -> (Vec<T>, PageMetadata) {
        (self.data, self.metadata)
    }

    pub fn map<U>(self, f: impl FnMut(T) -> U) -> Page<U> {
        Page {
            data: self.data.into_iter().map(f).collect(),
            metadata: self.metadata,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DateRange {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl DateRange {
    pub fn new(from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>) -> Self {
        Self { from, to }
    }

    pub fn is_unbounded(&self) -> bool {
        self.from.is_none() && self.to.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_range_is_unbounded_without_bounds() {
        assert!(DateRange::default().is_unbounded());
        assert!(!DateRange::new(Some(Utc::now()), None).is_unbounded());
    }

    fn page(n: u32) -> PageNumber {
        PageNumber::try_from(n).expect("valid page")
    }

    fn limit(n: u32) -> PageLimit {
        PageLimit::try_from(n).expect("valid limit")
    }

    #[test]
    fn middle_page_metadata() {
        let page = Page::new(vec![1, 2], 134, page(2), limit(20));
        let meta = page.metadata();
        assert_eq!(meta.page, 2);
        assert_eq!(meta.limit, 20);
        assert_eq!(meta.total, 134);
        assert_eq!(meta.total_pages, 7);
        assert_eq!(meta.first_page, 1);
        assert_eq!(meta.last_page, 7);
        assert_eq!(meta.next_page, Some(3));
        assert_eq!(meta.prev_page, Some(1));
    }

    #[test]
    fn empty_listing_metadata() {
        let page: Page<u8> = Page::new(vec![], 0, PageNumber::default(), PageLimit::default());
        let meta = page.metadata();
        assert_eq!(meta.total_pages, 0);
        assert_eq!(meta.last_page, 1);
        assert_eq!(meta.next_page, None);
        assert_eq!(meta.prev_page, None);
    }

    #[test]
    fn page_past_the_end() {
        let page: Page<u8> = Page::new(vec![], 5, page(3), limit(20));
        let meta = page.metadata();
        assert!(page.data().is_empty());
        assert_eq!(meta.last_page, 1);
        assert_eq!(meta.next_page, None);
        assert_eq!(meta.prev_page, Some(2));
    }

    #[test]
    fn exact_multiple_has_no_trailing_page() {
        let meta = Page::new(vec![0u8; 20], 40, page(2), limit(20)).metadata();
        assert_eq!(meta.total_pages, 2);
        assert_eq!(meta.next_page, None);
    }

    #[test]
    fn limit_bounds() {
        assert!(PageLimit::try_from(0).is_err());
        assert!(PageLimit::try_from(101).is_err());
        assert_eq!(limit(1).get(), 1);
        assert_eq!(limit(100).get(), 100);
        assert_eq!(PageLimit::default().get(), DEFAULT_PAGE_LIMIT);
    }

    #[test]
    fn page_number_refuses_zero() {
        assert!(PageNumber::try_from(0).is_err());
        assert_eq!(PageNumber::default().get(), 1);
    }

    #[test]
    fn offset_follows_page_and_limit() {
        let request: PageRequest<(), ()> = PageRequest {
            page: page(3),
            limit: limit(25),
            sort: Sort::default(),
            filter: (),
        };
        assert_eq!(request.offset(), 50);
    }

    #[test]
    fn default_sort_is_descending() {
        assert_eq!(SortOrder::default(), SortOrder::Desc);
    }

    #[test]
    fn map_keeps_metadata() {
        let original = Page::new(vec![1, 2], 134, page(2), limit(20));
        let meta = original.metadata();
        let mapped = original.map(|n| n.to_string());
        assert_eq!(mapped.data(), ["1", "2"]);
        assert_eq!(mapped.metadata(), meta);
    }

    #[test]
    fn map_filter_keeps_paging() {
        let request: PageRequest<u8, ()> = PageRequest {
            page: page(2),
            limit: limit(10),
            sort: Sort::default(),
            filter: 7,
        };
        let mapped = request.map_filter(|f| f.to_string());
        assert_eq!(mapped.filter, "7");
        assert_eq!(mapped.offset(), 10);
    }
}
