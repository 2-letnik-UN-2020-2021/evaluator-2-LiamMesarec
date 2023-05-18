use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

pub type ListU32 = List<u32>;

pub struct List<T> {
    pub value: T,
    pub next: Option<Rc<RefCell<List<T>>>>
}

impl<T> List<T>
    where T: Default 
{
    pub fn new_empty() -> Self {
        Self { value: T::default(), next: None }
    }
}

impl<T> List<T> {
    pub fn new(value: T) -> Self {
        Self { value, next: None }
    }

    pub fn append(&mut self, list: Rc<RefCell<List<T>>>) -> Rc<RefCell<List<T>>> {
        match &self.next {
            Some(next) => next.borrow_mut().append(list),
            None => {
                self.next = Some(Rc::clone(&list));
                Rc::clone(&list)
            }
        }
    }

    pub fn len(&self) -> usize {
        match &self.next {
            Some(next) => 1 + next.borrow().len(),
            None => 1
        }
    }
}

impl<T> List<T>
where
    T: PartialOrd,
    T: Clone
{
    pub fn is_ordered(&self) -> bool {
        self.is_ordered_helper(&Rc::new(RefCell::new(
            Self { value : self.value.clone(), next: self.next.clone() }
        )))
    }

    fn is_ordered_helper(&self, list: &Rc<RefCell<List<T>>>) -> bool {
        match &self.next {
            Some(next) => {
                if next.borrow().value < list.borrow().value {
                    return false;
                }

                next.borrow().is_ordered_helper(&list.borrow().next.as_ref().unwrap())
            }
            None => {
                true
            }
        }
    }
}

impl<T> fmt::Display for List<T>
where
    T: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match &self.next {
            Some(next) => {
                write!(f, "{};{}", self.value, next.borrow().to_string())
            },
            None => write!(f, "{}", self.value)
        }
    }
}

