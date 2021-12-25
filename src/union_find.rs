use std::cmp::Ordering;

/// This is a standard UnionFind algorithm, see
/// https://en.wikipedia.org/wiki/Disjoint-set_data_structure
/// It uses the path-compression heuristic. Instead of merge-by-size or merge-by-rank we
/// simply assume that items are ordered and always choose the larger ones as roots.
/// Although the theoretical performance of this method is worse, I expect practical performance to be equal.
/// Actually, it should even be faster, because we do not need to compute and store ranks or sizes.
/// It is also used by Mohex: https://github.com/cgao3/benzene-vanilla-cmake/blob/master/src/mohex/MoHexBoard.cpp
///
/// This trait makes no assumptions on how parents are stored. Implementors must provide `get_parent` and `set_parent` methods for this.

pub trait UnionFind<T: Copy + PartialOrd + Eq> {
    fn get_parent(&self, item: T) -> Option<T>;
    fn set_parent(&mut self, item: T, parent: T);

    fn is_in_same_set(&mut self, item1: T, item2: T) -> bool {
        self.find_root(item1) == self.find_root(item2)
    }

    fn find_root(&mut self, item: T) -> T {
        let mut item = item;
        let mut root = item;

        while let Some(next_root) = self.get_parent(root) {
            root = next_root;
        }

        while item != root {
            self.set_parent(item, root);
            item = self.get_parent(item).unwrap();
        }

        root
    }

    fn merge(&mut self, item1: T, item2: T) {
        let root1 = self.find_root(item1);
        let root2 = self.find_root(item2);

        match root1.partial_cmp(&root2) {
            Some(Ordering::Equal) => (),
            Some(Ordering::Greater) => {
                self.set_parent(root2, root1);
            }
            Some(Ordering::Less) | None => {
                self.set_parent(root1, root2);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type Item = u8;

    struct Collection {
        parents: Vec<Option<Item>>,
    }

    impl Collection {
        pub fn new(size: usize) -> Self {
            Self {
                parents: vec![None; size],
            }
        }
    }

    impl UnionFind<Item> for Collection {
        fn get_parent(&self, item: Item) -> Option<Item> {
            self.parents[item as usize]
        }

        fn set_parent(&mut self, item: Item, parent: Item) {
            self.parents[item as usize] = Some(parent);
        }
    }

    #[test]
    fn test_find_root() {
        let mut collection = Collection::new(4);
        assert_eq!(collection.find_root(2), 2);

        collection.merge(2, 3);
        assert_eq!(collection.find_root(2), 3);
    }

    #[test]
    fn test_find_root_compresses_paths() {
        let mut collection = Collection::new(4);
        collection.merge(0, 1);
        collection.merge(1, 2);

        assert_eq!(collection.parents[0], Some(1));
        assert_eq!(collection.find_root(0), 2);
        assert_eq!(collection.parents[0], Some(2));
    }

    #[test]
    fn test_merge_below_root() {
        let mut collection = Collection::new(4);
        collection.merge(1, 2);
        collection.merge(0, 1);
        assert_eq!(collection.parents[0], Some(2));
    }

    #[test]
    fn test_merge_smaller_root_below_larger_root() {
        let mut collection = Collection::new(4);
        collection.merge(0, 3);
        collection.merge(1, 2);
        assert_eq!(collection.parents[0], Some(3));
        assert_eq!(collection.parents[1], Some(2));

        collection.merge(0, 1);
        assert_eq!(collection.parents[1], Some(2));
        assert_eq!(collection.parents[2], Some(3));
        assert_eq!(collection.find_root(1), 3);
    }

    #[test]
    fn test_is_in_same_set() {
        let mut collection = Collection::new(4);
        collection.merge(0, 2);
        collection.merge(1, 2);
        assert!(collection.is_in_same_set(0, 1));
        assert!(collection.is_in_same_set(1, 2));
        assert!(!collection.is_in_same_set(0, 3));
    }
}
