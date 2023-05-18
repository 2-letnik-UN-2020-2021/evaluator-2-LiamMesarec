use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;
use crate::list::List;

type BinaryTreeU32 = BinaryTree<u32>;

#[derive(Debug)]
pub struct BinaryTree<T> {
    pub value: T,
    pub left: Option<Rc<RefCell<BinaryTree<T>>>>,
    pub right: Option<Rc<RefCell<BinaryTree<T>>>>
}

impl<T> BinaryTree<T> {
    pub fn new(value: T) -> Self {
        Self { value, left: None, right: None }
    }

    pub fn size(&self) -> usize {
        1 + if let Some(right) = &self.right { right.borrow().size() } else { 0 } + if let Some(left) = &self.left { left.borrow().size() } else { 0 }
    }

    pub fn height(&self) -> usize {
        1 + cmp::max(if let Some(right) = &self.right { right.borrow().height() } else { 0 }, if let Some(left) = &self.left { left.borrow().height() } else { 0 })
    }
}

impl<T> BinaryTree<T>
    where
        T: Default,
        T: Clone,
        T: Copy,
        T: std::fmt::Debug
{
    pub fn to_list(&self) -> Rc<RefCell<List<T>>>  {
        self.to_list_helper(
            &Rc::new(RefCell::new(Self { value : self.value.clone(), left: self.left.clone(), right: self.right.clone() })),
            &Rc::new(RefCell::new(List::new_empty()))
        ).borrow().next.clone().unwrap()
    }

    fn to_list_helper(&self, tree: &Rc<RefCell<BinaryTree<T>>>, list: &Rc<RefCell<List<T>>>) -> Rc<RefCell<List<T>>> {
        if let Some(left) = &tree.borrow().left {
            left.borrow().to_list_helper(&left, list);
        };

        list.borrow_mut().append(Rc::new(RefCell::new( List { value: tree.as_ref().borrow().value.clone(), next: None } )));

        if let Some(right) = &tree.borrow().right {
            right.borrow().to_list_helper(&right, list);
        };

        Rc::clone(&list)
    }

    pub fn rotate_right(&mut self) -> Rc<RefCell<BinaryTree<T>>> {
        let new_root = self.left.as_ref().unwrap().clone();
        let mut new_root_ref = new_root.borrow_mut();
        self.left = new_root_ref.right.take();
        new_root_ref.right = Some(Rc::new(RefCell::new(std::mem::replace(self, BinaryTree {
            value: new_root_ref.value.clone(),
            left: None,
            right: None
        }))));

        Rc::clone(&new_root)
    }

    pub fn rotate_left(&mut self) -> Rc<RefCell<BinaryTree<T>>> {
        let new_root = self.right.as_ref().unwrap().clone();
        let mut new_root_ref = new_root.borrow_mut();
        self.right = new_root_ref.left.take();
        new_root_ref.left = Some(Rc::new(RefCell::new(std::mem::replace(self, BinaryTree {
            value: new_root_ref.value.clone(),
            left: None,
            right: None
        }))));

        Rc::clone(&new_root)
    }
}

impl<T> fmt::Display for BinaryTree<T>
where
    T: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{},{})",
            if let Some(right) = &self.right { right.borrow().to_string() } else { String::from("") },
            self.value,
            if let Some(left) = &self.left { left.borrow().to_string() } else { String::from("") })
    }
}

