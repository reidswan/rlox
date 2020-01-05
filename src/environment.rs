use crate::data::literals::Literal;
use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

pub struct Environment {
    stack: LinkedList<HashMap<String, Rc<Literal>>>,
}

impl Environment {
    pub fn new() -> Self {
        let mut stack = LinkedList::new();
        stack.push_front(HashMap::new());
        Environment { stack }
    }

    pub fn join(&mut self) -> Result<(), String> {
        self.stack.pop_front().map(|_| ()).ok_or(String::from("Attempted to join in a non-forked environment"))
    }

    pub fn fork(&mut self) {
        self.stack.push_front(HashMap::new())
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.stack.front_mut().map(|values| values.insert(name, Rc::new(value)));
    }

    pub fn get(&self, name: &String) -> Option<Rc<Literal>> {
        for values in self.stack.iter() {
            if let Some(i) = values.get(name) {
                return Some(i.clone())
            }
        }
        None
    }

    pub fn assign(&mut self, name: String, value: Literal) -> Result<Rc<Literal>, String> {
        let value = Rc::new(value);
        self.assign_reference(name, value)
    }

    pub fn assign_reference(
        &mut self,
        name: String,
        value: Rc<Literal>,
    ) -> Result<Rc<Literal>, String> {
        for values in self.stack.iter_mut() {
            if values.contains_key(&name) {
                values.insert(name, value.clone());
                return Ok(value.clone())
            }
        };
        Err(format!("Attempted to assign to '{}' before declaration", name))
    }
}
