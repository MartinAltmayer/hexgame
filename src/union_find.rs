use std::cmp::Ordering;

pub trait UnionFind<T: Copy + PartialOrd + Eq> {
    fn get_parent(&self, item: T) -> T;
    fn set_parent(&mut self, item: T, parent: T);

    fn is_in_same_set(&mut self, item1: T, item2: T) -> bool {
        self.find_root(item1) == self.find_root(item2)
    }

    fn find_root(&mut self, item: T) -> T {
        let mut root = item;
        loop {
            let parent = self.get_parent(root);
            if parent != root {
                root = parent;
            } else {
                break;
            }
        }

        loop {
            let parent = self.get_parent(item);
            if parent != root {
                self.set_parent(item, root)
            } else {
                break;
            }
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd)]
struct Item {
    value: u8,
    parent: usize,
}

#[cfg(test)]
mod test {
    use super::*;

    type Item = u8;

    struct Collection {
        parents: Vec<Item>,
    }

    impl UnionFind<Item> for Collection {
        fn get_parent(&self, item: Item) -> Item {
            self.parents[item as usize]
        }

        fn set_parent(&mut self, item: Item, parent: Item) {
            self.parents[item as usize] = parent;
        }
    }

    fn build_collection() -> Collection {
        // Each item has itself as parent, i.e. all sets have a single element.
        return Collection {
            parents: vec![0, 1, 2, 3],
        };
    }

    #[test]
    fn test_find_root() {
        let mut collection = build_collection();
        assert_eq!(collection.find_root(2), 2);

        collection.merge(2, 3);
        assert_eq!(collection.find_root(2), 3);
    }

    #[test]
    fn test_find_root_compresses_paths() {
        let mut collection = build_collection();
        collection.merge(0, 1);
        collection.merge(1, 2);

        assert_eq!(collection.parents[0], 1);
        assert_eq!(collection.find_root(0), 2);
        assert_eq!(collection.parents[0], 2);
    }

    #[test]
    fn test_merge_below_root() {
        let mut collection = build_collection();
        collection.merge(1, 2);
        collection.merge(0, 1);
        assert_eq!(collection.parents[0], 2);
    }

    #[test]
    fn test_merge_smaller_root_below_larger_root() {
        let mut collection = build_collection();
        collection.merge(0, 3);
        collection.merge(1, 2);
        assert_eq!(collection.parents[0], 3);
        assert_eq!(collection.parents[1], 2);

        collection.merge(0, 1);
        assert_eq!(collection.parents[1], 2);
        assert_eq!(collection.parents[2], 3);
        assert_eq!(collection.find_root(1), 3);
    }

    #[test]
    fn test_is_in_same_set() {
        let mut collection = build_collection();
        collection.merge(0, 2);
        collection.merge(1, 2);
        assert!(collection.is_in_same_set(0, 1));
        assert!(collection.is_in_same_set(1, 2));
        assert!(!collection.is_in_same_set(0, 3));
    }
}
