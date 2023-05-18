use std::rc::Rc;
use std::cell::RefCell;
use crate::list::List;


pub type TreeU32 = NTree<u32>;
pub type Tree<T> = NTree<T>;

pub struct NTree<T> {
    pub value: T,
    pub children: Option<Rc<RefCell<Vec<NTree<T>>>>>
}

impl<T> NTree<T> {
    pub fn height(&self) -> usize {
        match &self.children {
            Some(children) => 1 + children.borrow().iter().map(|child| child.height()).max().unwrap_or(0),
            None => 1
        }
    }

    pub fn size(&self) -> usize {
        match &self.children {
            Some(children) => 1 + children.borrow().iter().map(|child| child.size()).sum::<usize>(),
            None => 1
        }
    }
}

impl<T: Clone> NTree<T> {
    pub fn to_list(&self) -> Option<Rc<RefCell<List<T>>>> {
        let list = Rc::new(RefCell::new(List { value: self.value.clone(), next: None }));
        self.to_list_helper(&list);
        Some(list)
    }

    fn to_list_helper(&self, list: &Rc<RefCell<List<T>>>) {
        if let Some(children) = &self.children {
            for child in children.borrow().iter() {
                let new_node = Rc::new(RefCell::new(List {
                    value: child.value.clone(),
                    next: None,
                }));
                list.borrow_mut().append(new_node.clone());
                child.to_list_helper(&new_node);
            }
        }
    }
}

impl<T: std::fmt::Display> NTree<T> {
    pub fn to_string(&self) -> String {
        let mut s = self.value.to_string();
        if let Some(children) = &self.children {
            s += " ( ";
            let len = children.borrow().len();
            for (i, child) in children.borrow().iter().enumerate() {
                s += &child.to_string();
                if i < len - 1 {
                    s += ", ";
                }
            }
            s += " )";
        }
        s
    }
}

