extern crate rustc_serialize;

use std::collections::BTreeMap;
use rustc_serialize::json::{self, Json, ToJson};
use std::fmt;


fn main() {}

#[derive(Debug)]
pub struct Element {
    name: String,
    attributes: BTreeMap<String, String>,
    keys: BTreeMap<u32, usize>,
    children: Vec<Element>,
}

// String value representation for Element
impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}", self.name);
        for (key, value) in self.attributes.iter() {
            write!(f, " {}={}", key, value);
        }
        write!(f, ">");
        for el in self.children.iter() {
            write!(f, "\n{}", el);
        }
        write!(f, "</{}>", self.name)
    }
}

// JSON value representation for Element
impl ToJson for Element {
    fn to_json(&self) -> Json {
        let mut res = BTreeMap::new();
        let mut children = Vec::new();
        for child in self.children.iter() {
            children.push(child.to_json());
        }
        res.insert("name".to_string(), self.name.to_json());
        res.insert("attributes".to_string(), self.attributes.clone().to_json());
        res.insert("children".to_string(), children.to_json());
        Json::Object(res)
    }
}

impl Clone for Element {
    fn clone(&self) -> Element {
        Element {
            name: self.name.clone(),
            keys: self.keys.clone(),
            attributes: self.attributes.clone(),
            children: self.children.clone(),
        }
    }
}

// This should only be used for testing/debugging purposes
impl PartialEq for Element {
    fn eq(&self, other: &Element) -> bool {
        self.name == other.name && self.children.len() == other.children.len() &&
        self.children
            .iter()
            .zip(other.children.iter())
            .all(|(left, right)| left == right) &&
        self.attributes.len() == other.attributes.len() &&
        self.attributes
            .iter()
            .all(|(key, value)| {
                if let Some(value_) = other.attributes.get(key) {
                    value == value_
                } else {
                    false
                }
            })
    }
}

impl Element {
    // Creates a new Element
    pub fn new<'a>(name: &'a str) -> Element {
        Element {
            name: name.to_string(),
            attributes: BTreeMap::new(),
            keys: BTreeMap::new(),
            children: Vec::new(),
        }
    }
    // Appends a child to the existing Element
    pub fn add_child(&mut self, key: u32, child: Element) {
        let idx = self.children.len();
        self.keys.insert(key, idx);
        self.children.push(child);
    }

    pub fn add_attr<'a>(&mut self, name: &'a str, value: &'a str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }

    // Performs a diff of the children of two trees
    pub fn child_diff(&self, other: &Element, level: usize) -> DiffTree {
        use ChildChange::*;
        let mut nested: BTreeMap<usize, DiffTree> = BTreeMap::new();
        let mut changes: Vec<ChildChange> = Vec::new();

        for (key, &value) in self.keys.iter() {
            if let Some(&value_) = other.keys.get(key) {
                if value != value_ {
                    changes.push(SwapChild(value, value_));
                }
                let ref left = self.children[value];
                let ref right = other.children[value_];
                let diff = left.child_diff(right, level + 1);
                if diff.has_changes() {
                    nested.insert(value, diff);
                }
            } else {
                changes.push(RemoveChild(value));
            }
        }

        for (key, &value) in other.keys.iter() {
            if !self.keys.contains_key(key) {
                let ref right = other.children[value];
                changes.push(InsertChild(value, right.clone()));
            }
        }

        DiffTree {
            level: level,
            name: self.name.clone(),
            attr_changes: Element::attr_diff(&self.attributes, &other.attributes),
            child_changes: changes,
            nested: nested,
        }
    }

    // TODO
    #[inline(always)]
    fn attr_diff(left: &BTreeMap<String, String>,
                 right: &BTreeMap<String, String>)
                 -> Vec<ChildChange> {
        vec![]
    }
}

// A DiffTree reprents a serializable change that needs to made
// at a single level of DOM.
#[derive(Debug)]
pub struct DiffTree {
    level: usize,
    name: String,
    attr_changes: Vec<ChildChange>,
    child_changes: Vec<ChildChange>,
    nested: BTreeMap<usize, DiffTree>,
}

impl DiffTree {
    fn consume_to_json(self) -> Json {
        let mut res = BTreeMap::new();
        let empty: Vec<String> = Vec::new();
        res.insert("level".to_string(), self.level.to_json());
        res.insert("name".to_string(), self.name.to_json());
        res.insert("attr_changes".to_string(), empty.to_json());
        let values: Vec<Json> = self.child_changes
            .into_iter()
            .map(|x| x.consume_to_json())
            .collect();
        res.insert("child_changes".to_string(), values.to_json());
        Json::Object(res)
    }

    // Are there any actual changes in this structure>
    fn has_changes(&self) -> bool {
        self.attr_changes.len() != 0 ||
        self.child_changes.len() != 0 ||
        self.nested.len() != 0
    }
}

// This should only be used for testing/debugging purposes
impl PartialEq for DiffTree {
    fn eq(&self, other: &DiffTree) -> bool {
        self.level == other.level && self.name == other.name &&
        self.child_changes.len() == other.child_changes.len() &&
        self.child_changes
            .iter()
            .zip(other.child_changes.iter())
            .all(|(left, right)| left == right) &&
        self.attr_changes.len() == other.attr_changes.len() &&
        self.attr_changes
            .iter()
            .zip(other.attr_changes.iter())
            .all(|(left, right)| left == right) &&
        self.nested.len() == other.nested.len() &&
        self.nested
            .keys()
            .all(|key| {
                let value = self.nested.get(key).unwrap();
                if let Some(value_) = other.nested.get(key) {
                    value == value_
                } else {
                    false
                }
            })

    }
}

// A Change is an atomic action performed as part of a tree diff.
#[derive(Debug)]
pub enum ChildChange {
    // Insert a new child at the given index.
    InsertChild(usize, Element),
    // Remove a child at the given index.
    RemoveChild(usize),
    // Swap the elements at the given indexes.
    SwapChild(usize, usize),
}

impl ChildChange {
    fn consume_to_json(self) -> Json {
        use ChildChange::*;
        let mut res = BTreeMap::new();
        match self {
            InsertChild(idx, node) => {
                res.insert("kind".to_string(), "insert".to_json());
                res.insert("index".to_string(), idx.to_json());
                res.insert("value".to_string(), node.to_json());
            }
            RemoveChild(idx) => {
                res.insert("kind".to_string(), "remove".to_json());
                res.insert("index".to_string(), idx.to_json());
            }
            SwapChild(idx, idx_) => {
                res.insert("kind".to_string(), "swap".to_json());
                res.insert("value_left".to_string(), idx.to_json());
                res.insert("value_right".to_string(), idx_.to_json());
            }
        }
        Json::Object(res)
    }
}

impl PartialEq for ChildChange {
    fn eq(&self, other: &ChildChange) -> bool {
        use ChildChange::*;
        match (self, other) {
            (&SwapChild(idx, idx2), &SwapChild(idx_, idx2_)) => idx == idx_ && idx2 == idx2_,
            (&RemoveChild(idx), &RemoveChild(idx_)) => idx == idx_,
            (&InsertChild(idx, ref node), &InsertChild(idx_, ref node_)) => {
                idx == idx_ && node == node_
            }
            _ => false,
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[macro_export]
    macro_rules! map(
        { $($key:expr => $value:expr),* } => {
            {
            let mut m = BTreeMap::new();
            $(
                m.insert($key, $value);
            )*
            m
            }
        };
    );

    #[test]
    fn finds_differences() {
        let mut tree_1 = Element::new("root");
        tree_1.add_child(0, Element::new("div"));
        tree_1.add_child(1, Element::new("h1"));

        let mut tree_2 = Element::new("root");
        tree_2.add_child(1, Element::new("h1"));
        tree_2.add_child(0, Element::new("div"));

        let diff = tree_1.child_diff(&tree_2, 0);

        assert_eq!(diff,
                   DiffTree {
                       level: 0,
                       name: "root".to_string(),
                       attr_changes: vec![],
                       nested: map!{},
                       child_changes: vec![
                           ChildChange::SwapChild(0, 1),
                           ChildChange::SwapChild(1, 0),
                        ],
                   });
    }

    #[test]
    fn finds_nested_differences() {
        let mut tree_1 = Element::new("root");
        let mut leaf_1 =  Element::new("div");
        leaf_1.add_child(0, Element::new("br"));
        tree_1.add_child(0, leaf_1);
        tree_1.add_child(1, Element::new("h1"));

        let mut tree_2 = Element::new("root");
        tree_2.add_child(1, Element::new("h1"));
        tree_2.add_child(0, Element::new("div"));

        let diff = tree_1.child_diff(&tree_2, 0);
        assert_eq!(diff,
                   DiffTree{
                       level: 0,
                       name: "root".to_string(),
                       attr_changes: vec![],
                       child_changes: vec![
                         ChildChange::SwapChild(0, 1),
                         ChildChange::SwapChild(1, 0),
                       ],
                       nested: map!{
                           0 => DiffTree{
                               level: 1,
                               name: "div".to_string(),
                               attr_changes: vec![],
                               nested: map!{},
                               child_changes: vec![
                                   ChildChange::RemoveChild(0),
                               ]
                           }
                       },
                   });
    }
}
