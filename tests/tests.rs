#[macro_use] extern crate indoc;
#[macro_use] extern crate pretty_assertions;
extern crate rctree;

use rctree::Node;

use std::fmt;

#[test]
fn it_works() {
    use std::cell;

    struct DropTracker<'a>(&'a cell::Cell<u32>);
    impl<'a> Drop for DropTracker<'a> {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let mut new_counter = 0;
    let drop_counter = cell::Cell::new(0);
    let mut new = || {
        new_counter += 1;
        Node::new((new_counter, DropTracker(&drop_counter)))
    };

    {
        let mut a = new();  // 1
        a.append(new());  // 2
        a.append(new());  // 3
        a.prepend(new());  // 4
        let mut b = new();  // 5
        b.append(a.clone());
        a.insert_before(new());  // 6
        a.insert_before(new());  // 7
        a.insert_after(new());  // 8
        a.insert_after(new());  // 9
        let c = new();  // 10
        b.append(c.clone());

        assert_eq!(drop_counter.get(), 0);
        c.previous_sibling().unwrap().detach();
        assert_eq!(drop_counter.get(), 1);

        assert_eq!(b.descendants().map(|node| {
            let borrow = node.borrow();
            borrow.0
        }).collect::<Vec<_>>(), [
            5, 6, 7, 1, 4, 2, 3, 9, 10
        ]);
    }

    assert_eq!(drop_counter.get(), 10);
}


struct TreePrinter<T>(Node<T>);

impl<T: fmt::Debug> fmt::Debug for TreePrinter<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{:?}", self.0.borrow()).unwrap();
        iter_children(&self.0, 1, f);

        Ok(())
    }
}

fn iter_children<T: fmt::Debug>(parent: &Node<T>, depth: usize, f: &mut fmt::Formatter) {
    for child in parent.children() {
        for _ in 0..depth {
            write!(f, "    ").unwrap();
        }
        writeln!(f, "{:?}", child.borrow()).unwrap();
        iter_children(&child, depth + 1, f);
    }
}

#[test]
fn make_copy_1() {
    let mut node1 = Node::new(1);
    let node2 = Node::new(2);
    node1.append(node2);
    let node1_copy = node1.make_copy();
    node1.append(node1_copy);

    assert_eq!(format!("{:?}", TreePrinter(node1)), indoc!("
        1
            2
            1
    "));
}

#[test]
fn make_deep_copy_1() {
    let mut node1 = Node::new(1);
    let mut node2 = Node::new(2);
    node1.append(node2.clone());
    node2.append(node1.make_deep_copy());

    assert_eq!(format!("{:?}", TreePrinter(node1)), indoc!("
        1
            2
                1
                    2
    "));
}

#[test]
#[should_panic]
fn append_1() {
    let mut node1 = Node::new(1);
    let node1_2 = node1.clone();
    node1.append(node1_2);
}

#[test]
#[should_panic]
fn prepend_1() {
    let mut node1 = Node::new(1);
    let node1_2 = node1.clone();
    node1.prepend(node1_2);
}

#[test]
#[should_panic]
fn insert_before_1() {
    let mut node1 = Node::new(1);
    let node1_2 = node1.clone();
    node1.insert_before(node1_2);
}

#[test]
#[should_panic]
fn insert_after_1() {
    let mut node1 = Node::new(1);
    let node1_2 = node1.clone();
    node1.insert_after(node1_2);
}

#[test]
#[should_panic]
fn iter_1() {
    let mut node1 = Node::new(1);
    let mut node2 = Node::new(2);
    node1.append(node2.clone());
    node2.append(node1.make_deep_copy());

    let _n = node2.borrow_mut();
    for _ in node1.descendants() {}
}

//#[test]
//fn stack_overflow() {
//    let mut parent = Node::new(1);
//    for _ in 0..1000000 {
//        let node = Node::new(1);
//        parent.append(node.clone());
//        parent = node;
//    }
//}

#[test]
fn root_1() {
    let node1 = Node::new(1);
    assert_eq!(node1, node1.root());
}

#[test]
fn root_2() {
    let mut node1 = Node::new("node1");
    let node2 = Node::new("node2");
    node1.append(node2.clone());
    assert_eq!(node1.root(), node1);
    assert_eq!(node2.root(), node1);
}

#[test]
fn root_3() {
    let mut node1 = Node::new("node1");
    let mut node2 = Node::new("node2");
    let node3 = Node::new("node3");
    node1.append(node2.clone());
    node2.append(node3.clone());
    assert_eq!(node1.root(), node1);
    assert_eq!(node2.root(), node1);
    assert_eq!(node3.root(), node1);
}

#[test]
fn root_4() {
    let mut node1 = Node::new("node1");
    let node2 = Node::new("node2");
    let node3 = Node::new("node3");
    node1.append(node2.clone());
    node1.prepend(node3.clone());
    assert_eq!(node1.root(), node1);
    assert_eq!(node2.root(), node1);
    assert_eq!(node3.root(), node1);
}
