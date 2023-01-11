use std::{cell::RefCell, collections::VecDeque, rc::Rc};

pub(crate) struct Receiver<T> {
    buffer: Rc<RefCell<VecDeque<T>>>,
}

impl<T> Receiver<T> {
    pub(crate) fn receive(&self) -> Option<T> {
        self.buffer.borrow_mut().pop_front()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.buffer.borrow().is_empty()
    }
}

#[derive(Clone)]
pub(crate) struct Sender<T> {
    buffer: Rc<RefCell<VecDeque<T>>>,
}

impl<T> Sender<T> {
    pub(crate) fn send(&self, data: T) {
        self.buffer.borrow_mut().push_back(data);
    }
}

pub(crate) fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let buffer: Rc<RefCell<VecDeque<T>>> = Rc::new(RefCell::new(VecDeque::new()));

    let sender = Sender {
        buffer: buffer.clone(),
    };
    let receiver = Receiver { buffer };
    (sender, receiver)
}
