use cursive_table_view::{TableView, TableViewItem};
use std::hash::Hash;

pub trait TableViewExtensions<T, H>
where
    T: TableViewItem<H>,
    H: Eq + Hash + Copy + Clone + 'static,
{
    fn index_of<F>(&mut self, lambda: F) -> Option<usize>
    where
        F: Fn(&T) -> bool;
}

impl<T, H> TableViewExtensions<T, H> for TableView<T, H>
where
    T: TableViewItem<H>,
    H: Eq + Hash + Copy + Clone + 'static,
{
    fn index_of<F>(&mut self, lambda: F) -> Option<usize>
    where
        F: Fn(&T) -> bool,
    {
        self.borrow_items()
            .iter()
            .enumerate()
            .find(|(_, item)| lambda(item))
            .map(|(index, _)| index)
    }
}
