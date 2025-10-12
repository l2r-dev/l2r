use crate::assets::html::TeraContext;

pub struct Pagination {
    current_page: usize,
    total_pages: usize,
    start_idx: usize,
    end_idx: usize,
}

impl Pagination {
    pub fn new(len: usize, page: usize, per_page: usize) -> Self {
        let per_page = per_page.max(1);
        let total_pages = len.div_ceil(per_page);
        let total_pages = total_pages.max(1);
        let current_page = page.max(1).min(total_pages);

        let start_idx = page.saturating_sub(1) * per_page;
        let end_idx = (start_idx + per_page).min(len);

        Pagination {
            current_page,
            total_pages,
            start_idx,
            end_idx,
        }
    }

    pub fn current_page(&self) -> usize {
        self.current_page
    }

    pub fn prev_page(&self) -> usize {
        self.current_page.saturating_sub(1)
    }

    pub fn next_page(&self) -> usize {
        self.current_page + 1
    }

    pub fn total_pages(&self) -> usize {
        self.total_pages
    }

    pub fn has_prev(&self) -> bool {
        self.current_page > 1
    }

    pub fn has_next(&self) -> bool {
        self.current_page < self.total_pages
    }

    pub fn range(&self) -> (usize, usize) {
        (self.start_idx, self.end_idx)
    }
}

impl TeraContext for Pagination {
    fn tera_context(&self) -> tera::Context {
        let mut context = tera::Context::new();
        context.insert("current_page", &self.current_page());
        context.insert("total_pages", &self.total_pages());
        context.insert("has_prev", &self.has_prev());
        context.insert("has_next", &self.has_next());
        context.insert("prev_page", &self.prev_page());
        context.insert("next_page", &self.next_page());
        context
    }
}

impl Pagination {
    /// Create an iterator over paginated items from a slice
    pub fn iter_slice<'a, T>(&self, items: &'a [T]) -> impl Iterator<Item = &'a T> {
        let (start, end) = self.range();
        items.get(start..end).unwrap_or(&[]).iter()
    }

    /// Apply pagination to any iterator
    pub fn iter_paginate<I: Iterator>(
        iter: I,
        page: usize,
        per_page: usize,
    ) -> impl Iterator<Item = I::Item> {
        let skip = page.saturating_sub(1) * per_page;
        iter.skip(skip).take(per_page)
    }
}

pub trait PaginateSlice<T> {
    /// Paginate a slice and return a Vec of cloned items with pagination info
    fn paginate(&self, page: usize, per_page: usize) -> (Vec<T>, Pagination)
    where
        T: Clone;

    /// Get a paginated slice without cloning items
    fn paginate_slice(&self, page: usize, per_page: usize) -> (&[T], Pagination);

    /// Paginate with a function that determines how many items fit based on their content
    fn paginate_func<F>(
        &self,
        page: usize,
        max_capacity: usize,
        weight_fn: F,
    ) -> (Vec<T>, Pagination)
    where
        T: Clone,
        F: Fn(&T) -> usize;
}

impl<T> PaginateSlice<T> for [T] {
    fn paginate(&self, page: usize, per_page: usize) -> (Vec<T>, Pagination)
    where
        T: Clone,
    {
        let (slice, pagination) = self.paginate_slice(page, per_page);
        (slice.to_vec(), pagination)
    }

    fn paginate_slice(&self, page: usize, per_page: usize) -> (&[T], Pagination) {
        let len = self.len();
        let pagination = Pagination::new(len, page, per_page);
        let start = page.saturating_sub(1) * per_page;
        let slice = &self[start.min(len)..start.saturating_add(per_page).min(len)];
        (slice, pagination)
    }

    fn paginate_func<F>(
        &self,
        page: usize,
        max_capacity: usize,
        weight_fn: F,
    ) -> (Vec<T>, Pagination)
    where
        T: Clone,
        F: Fn(&T) -> usize,
    {
        if self.is_empty() {
            return (Vec::new(), Pagination::new(0, 1, 1));
        }

        // Create pages by grouping items that fit within capacity
        let pages: Vec<Vec<&T>> = self
            .iter()
            .scan(0, |current_weight, item| {
                let item_weight = weight_fn(item);
                if *current_weight + item_weight > max_capacity && *current_weight > 0 {
                    *current_weight = item_weight;
                    Some((item, true)) // Start new page
                } else {
                    *current_weight += item_weight;
                    Some((item, false)) // Continue current page
                }
            })
            .fold(Vec::new(), |mut pages: Vec<Vec<&T>>, (item, new_page)| {
                if new_page || pages.is_empty() {
                    pages.push(vec![item]);
                } else if let Some(last_page) = pages.last_mut() {
                    last_page.push(item);
                }
                pages
            });

        let total_pages = pages.len().max(1);
        let current_page_items = pages
            .get(page)
            .map(|page_items| page_items.iter().cloned().cloned().collect())
            .unwrap_or_default();

        let pagination = Pagination::new(total_pages, page + 1, 1);
        (current_page_items, pagination)
    }
}

pub trait PaginateIterator<T> {
    fn paginate_collect(self, page: usize, per_page: usize) -> (Vec<T>, Pagination);
}

impl<I: Iterator> PaginateIterator<I::Item> for I
where
    I: ExactSizeIterator,
{
    fn paginate_collect(self, page: usize, per_page: usize) -> (Vec<I::Item>, Pagination) {
        let len = self.len();
        let pagination = Pagination::new(len, page, per_page);
        let skip = page.saturating_sub(1) * per_page;
        let items: Vec<I::Item> = self.skip(skip).take(per_page).collect();
        (items, pagination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_improvements() {
        let items: Vec<i32> = (1..=20).collect();

        let (slice, pagination) = items.paginate_slice(2, 5);
        assert_eq!(slice, &[6, 7, 8, 9, 10]);
        assert_eq!(pagination.current_page, 2);
        assert_eq!(pagination.total_pages, 4);
        assert!(pagination.has_prev());
        assert!(pagination.has_next());

        let iter = items.iter().copied();
        let (page_items, _) = iter.paginate_collect(3, 5);
        assert_eq!(page_items, vec![11, 12, 13, 14, 15]);

        let (empty_slice, pagination) = items.paginate_slice(10, 5);
        assert!(empty_slice.is_empty());
        assert_eq!(pagination.current_page, 4); // Clamped to max page

        let iter_items: Vec<_> = Pagination::iter_paginate(items.iter(), 2, 3).collect();
        assert_eq!(iter_items, vec![&4, &5, &6]);

        let p = Pagination::new(items.len(), 1, 7);
        let slice_iter: Vec<_> = p.iter_slice(&items).collect();
        assert_eq!(slice_iter, vec![&1, &2, &3, &4, &5, &6, &7]);
    }
}
