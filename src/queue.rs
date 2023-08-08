use std::collections::VecDeque;

#[derive(Debug)]
pub struct Queue<T>
where T: std::fmt::Debug + Clone
{
    elements: VecDeque<T>
}

impl<T> Queue<T>
where T: std::fmt::Debug + Clone
{
    pub fn new(capacity: usize) -> Self {
        Self { elements: VecDeque::with_capacity(capacity) }
    }

    pub fn push(&mut self, e: T) {
        if self.elements.len() >= self.elements.capacity() {
            self.elements.pop_front();
        }
        self.elements.push_back(e);
    }

    pub fn clone_elements(&self) -> Vec<T> {
        let mut res = Vec::<T>::new();
        for e in self.elements.iter() {
            res.push(e.clone());
        }
        res
    }
}
