#![allow(dead_code)]
use std::collections::BTreeMap;
use Element::*;

fn main() {}

// Represents an HTML element.
#[derive(Debug)]
pub enum Element {
    Text { key: Key, value: String },
    Void {
        key: Key,
        name: String,
        attributes: Option<BTreeMap<String, String>>,
    },
    Parent {
        key: Key,
        name: String,
        attributes: Option<BTreeMap<String, String>>,
        children: Vec<Element>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Key {
    Local(u64),
    Global(u64),
}

impl Clone for Element {
    fn clone(&self) -> Element {
        match *self {
            Text { ref key, ref value } => {
                Text {
                    key: *key,
                    value: value.clone(),
                }
            }
            Void { ref key, ref name, ref attributes } => {
                Void {
                    key: *key,
                    name: name.clone(),
                    attributes: attributes.clone(),
                }
            }
            Parent { ref key, ref name, ref attributes, ref children } => {
                Parent {
                    key: *key,
                    name: name.clone(),
                    attributes: attributes.clone(),
                    children: children.clone(),
                }
            }
        }
    }
}

// Fast equality checks are implemented by comparing references, not values
impl PartialEq for Element {
    fn eq(&self, other: &Element) -> bool {
        self.to_key() == other.to_key()
    }
}

impl Element {
    #[inline(always)]
    pub fn to_key(&self) -> Key {
        match *self {
            Text { key, .. } => key,
            Void { key, .. } => key,
            Parent { key, .. } => key,
        }
    }

    // Tries to add an elment to end of a list of children
    pub fn append_child(&mut self, el: Element) -> Result<Change, DOMError> {
        match *self {
            Parent { ref mut children, .. } => {
                children.push(el.clone());
                Ok(Change::AppendChild(el))
            }
            _ => Err(DOMError::ChildlessElementOpError),
        }
    }

    // Tries to insert an element before the index
    pub fn insert_before(&mut self, index: usize, el: Element) -> Result<Change, DOMError> {
        match *self {
            Parent { ref mut children, .. } => {
                if index < children.len() {
                    let el_ref = children[index].to_key();
                    children.insert(index, el.clone());
                    Ok(Change::InsertBefore(el_ref, el))
                } else {
                    Err(DOMError::IndexOOBError)
                }
            }
            _ => Err(DOMError::ChildlessElementOpError),
        }
    }

    // Tries to insert all elements before the index, in order
    pub fn insert_all(&mut self, index: usize, els: Vec<Element>) -> Result<Change, DOMError> {
        match *self {
            Parent { ref mut children, .. } => {
                if index < children.len() {
                    let el_ref = children[index].to_key();
                    let mut i = index;
                    for el in els.iter() {
                        children.insert(i, el.clone());
                        i += 1;
                    }
                    Ok(Change::InsertAll(el_ref, els.into_boxed_slice()))
                } else {
                    Err(DOMError::IndexOOBError)
                }
            }
            _ => Err(DOMError::ChildlessElementOpError),
        }
    }

    // Tries to append all elements to the end of a list of children
    pub fn append_all(&mut self, els: Vec<Element>) -> Result<Change, DOMError> {
        match *self {
            Parent { ref mut children, .. } => {
                for el in els.iter() {
                    children.push(el.clone());
                }
                Ok(Change::AppendAll(els.into_boxed_slice()))
            }
            _ => Err(DOMError::ChildlessElementOpError),
        }
    }

    // Replaces the nth child with a new element
    pub fn replace_child(&mut self, index: usize, el: Element) -> Result<Change, DOMError> {
        match *self {
            Parent { ref mut children, .. } => {
                if index < children.len() {
                    children.push(el.clone());
                    let old = children.swap_remove(index);
                    Ok(Change::ReplaceChild(old.to_key(), el))
                } else {
                    Err(DOMError::IndexOOBError)
                }
            }
            _ => Err(DOMError::ChildlessElementOpError),
        }
    }

    // Removes the nth child
    pub fn remove_child(&mut self, index: usize) -> Result<Change, DOMError> {
        match *self {
            Parent { ref mut children, .. } => {
                if index < children.len() {
                    let old = children.remove(index);
                    Ok(Change::RemoveChild(old.to_key()))
                } else {
                    Err(DOMError::IndexOOBError)
                }
            }
            _ => Err(DOMError::ChildlessElementOpError),
        }
    }

    // Computes a new ordering of children based on the provided comparison function
    pub fn reorder_children<'a, B: Ord>(&mut self,
                                        cmp: &'a Fn(&Element) -> B)
                                        -> Result<Change, DOMError> {
        match *self {
            Parent { ref mut children, .. } => {
                children.sort_by_key(cmp);
                let keys: Vec<Key> = children.iter().map(|el| el.to_key()).collect();
                Ok(Change::ReorderChildren(keys.into_boxed_slice()))
            }
            _ => Err(DOMError::ChildlessElementOpError),
        }
    }

    pub fn update_text<'a>(&mut self, new: &'a str) -> Result<Change, DOMError> {
        match *self {
            Text { ref mut value, .. } => {
                value.clear();
                value.push_str(new);
                Ok(Change::UpdateText(new.to_string()))
            }
            _ => Err(DOMError::NotATextNode),
        }
    }
}

#[derive(Debug)]
enum Event {
    Click {
        bubbles: bool,
        cancelable: bool,
        target: Key,
        screen_x: f64,
        screeny_y: f64,
    },
    DoubleClick {
        bubbles: bool,
        cancelable: bool,
        target: Key,
        screen_x: f64,
        screen_y: f64,
    },
    MouseDown {
        bubbles: bool,
        cancelable: bool,
        target: Key,
    },
    MouseEnter {
        bubbles: bool,
        cancelable: bool,
        target: Key,
    },
    MouseLeave {
        bubbles: bool,
        cancelable: bool,
        target: Key,
    },
    MouseMove {
        bubbles: bool,
        cancelable: bool,
        target: Key,
    },
    MouseOut {
        bubbles: bool,
        cancelable: bool,
        target: Key,
    },
    MouseUp {
        bubbles: bool,
        cancelable: bool,
        target: Key,
    },
    KeyDown {
        bubbles: bool,
        cancelable: bool,
        target: Key,
        char_code: u32,
    },
    KeyPress {
        bubbles: bool,
        cancelable: bool,
        target: Key,
        char_code: u32,
    },
    KeyUp {
        bubbles: bool,
        cancelable: bool,
        target: Key,
        char_code: u32,
    },
    ContextMenu {
        bubbles: bool,
        cancelable: bool,
        target: Key,
    },
    Change {
        bubbles: bool,
        cancelable: bool,
        target: Key,
        value: String,
    },
}

#[derive(Debug)]
pub struct DiffTree {
    cost: u64,
    changes: Option<Box<[Change]>>,
    children: Option<Box<[(Key, DiffTree)]>>,
}

#[derive(Debug)]
pub enum Change {
    AppendChild(Element),
    AppendAll(Box<[Element]>),
    InsertBefore(Key, Element),
    InsertAll(Key, Box<[Element]>),
    ReplaceChild(Key, Element),
    RemoveChild(Key),
    ReorderChildren(Box<[Key]>),
    UpdateText(String),
}

// Common DOM manipulation errors
pub enum DOMError {
    ChildlessElementOpError,
    IndexOOBError,
    NotATextNode,
}


#[cfg(test)]
mod tests {
    use super::*;

    fn element_eq(left: &Element, right: &Element) {
        match (left, right) {
            (&Element::Text { ref value, .. }, &Element::Text { value: ref value_, .. }) => {
                assert!(value == value_,
                        "Element::Text does not match\nExpected {}, found {}",
                        value,
                        value_);
            }
            (&Element::Void { ref name, .. }, &Element::Void { name: ref name_, .. }) => {
                assert!(name == name_,
                        "Element::Void does not match\nExpected type {}, found {}",
                        name,
                        name_);
            }
            (&Element::Parent { ref name, ref children, .. },
             &Element::Parent { name: ref name_, children: ref children_, .. }) => {
                assert!(name == name_,
                        "Element::Parent does not match\nExpected type {}, found {}",
                        name,
                        name_);
                if children.len() != children_.len() {
                    assert!(false,
                            "Element::Parent have different numbers of children\nExpected {}, \
                             found {}",
                            children.len(),
                            children_.len());
                }
                for (left, right) in children.into_iter().zip(children_.into_iter()) {
                    element_eq(left, right);
                }
            }
            _ => {
                assert!(false, "Element types do not match");
            }
        }
    }

    macro_rules! el {
        ($name:ident[]) => (
            {
                Element::Parent{
                    key: Key::Local(0),
                    name: stringify!($name).to_string(),
                    attributes: None,
                    children: vec![],
                }
            }
        );
        ($name:ident[[$($child:expr),* ]]) => (
            {
                let mut children = vec![];
                $(
                    children.push($child);
                )*
                Element::Parent{
                    key: Key::Local(0),
                    name: stringify!($name).to_string(),
                    attributes: None,
                    children: children,
                }
            }
        );
    }

    #[test]
    fn test_append_child() {
        let mut el = el!(div[]);
        let changes = el.append_child(el!(hr[]));
        assert!(changes.is_ok());
        element_eq(&el, &el!(div[[el!(hr[])]]));
    }
}
