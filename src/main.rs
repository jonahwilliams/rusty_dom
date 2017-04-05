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
        keymap: BTreeMap<Key, usize>,
        attributes: Option<BTreeMap<String, String>>,
        children: Vec<Element>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
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
            Parent { ref key, ref name, ref attributes, ref children, ref keymap } => {
                Parent {
                    key: *key,
                    name: name.clone(),
                    keymap: keymap.clone(),
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

    pub fn diff(&self, other: &Element) -> Option<DiffTree> {
        match (self, other) {
            (&Text { value: ref left, .. }, &Text { value: ref right, .. }) => {
                if left != right {
                    Some(DiffTree {
                        changes: Some(Box::new([Change::UpdateText(right.to_string())])),
                        children: None,
                    })
                } else {
                    None
                }
            }
            (&Void { name: ref left, .. }, &Void { name: ref right, .. }) => {
                if left == right {
                    None
                } else {
                    Some(DiffTree {
                        changes: Some(Box::new([Change::ReplaceNode(other.clone())])),
                        children: None,
                    })
                }
            }
            (&Parent { name: ref left,
                       children: ref left_children,
                       keymap: ref left_keymap,
                       .. },
             &Parent { name: ref right,
                       children: ref right_children,
                       keymap: ref right_keymap,
                       .. }) if left == right => {
                let mut changes = vec![];
                let mut child_changes = vec![];
                let mut order = false;

                for (&key, &value) in left_keymap.iter() {
                    if let Some(&value_) = right_keymap.get(&key) {
                        if value != value_ {
                            order = true;
                        }
                        if let Some(child_tree) = left_children[value]
                            .diff(&right_children[value_]) {
                            child_changes.push((key, child_tree));
                        }
                    } else {
                        changes.push(Change::RemoveChild(key));
                    }
                }
                for (key, &value) in right_keymap.iter() {
                    if let Some(&value_) = left_keymap.get(&key) {
                        if value != value_ {
                            order = true;
                        }
                    } else {
                        changes.push(Change::InsertChild(right_children[value].clone()));
                    }
                }
                if order {
                    let keys: Vec<Key> = right_children.iter()
                        .map(|x| x.to_key())
                        .collect();
                    changes.push(Change::SortChildren(keys.into_boxed_slice()));
                }

                if child_changes.len() == 0 {
                    Some(DiffTree {
                        changes: Some(changes.into_boxed_slice()),
                        children: None,
                    })
                } else {
                    Some(DiffTree {
                        changes: Some(changes.into_boxed_slice()),
                        children: Some(child_changes.into_boxed_slice()),
                    })
                }
            }
            _ => {
                Some(DiffTree {
                    changes: Some(Box::new([Change::ReplaceNode(other.clone())])),
                    children: None,
                })
            }
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

#[derive(Debug, PartialEq)]
pub struct DiffTree {
    changes: Option<Box<[Change]>>,
    children: Option<Box<[(Key, DiffTree)]>>,
}

#[derive(Debug, PartialEq)]
pub enum Change {
    RemoveChild(Key),
    InsertChild(Element),
    SortChildren(Box<[Key]>),
    UpdateText(String),
    ReplaceNode(Element),
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! el {
        ($name:ident[key=$value:expr]) => (
            {
                Element::Void{
                    key: Key::Local($value),
                    name: stringify!($name).to_string(),
                    attributes: None,
                }
            }
        );
        ($name:ident[]) => (
            {
                Element::Void{
                    key: Key::Local(0),
                    name: stringify!($name).to_string(),
                    attributes: None,
                }
            }
        );
        ($name:ident[key=$value:expr, $($child:expr),* ]) => (
            {
                let mut children = vec![];
                let mut keymap = BTreeMap::new();
                let mut index = 0;
                $(
                    children.push($child);
                    keymap.insert($child.to_key(), index);
                    index += 1;
                )*

                Element::Parent{
                    key: Key::Local($value),
                    name: stringify!($name).to_string(),
                    keymap: keymap,
                    attributes: None,
                    children: children,
                }
            }
        );
    }

    #[test]
    fn test_remove_single() {
        let left = el!(div[
            key=0,
            el!(div[key=1]),
            el!(div[key=2]),
            el!(div[key=3])
        ]);
        let right = el!(div[
            key=0,
            el!(div[key=1]),
            el!(div[key=2])
        ]);
        let diff = left.diff(&right);

        assert_eq!(diff, Some(DiffTree{
            changes: Some(vec![
                Change::RemoveChild(Key::Local(3)),
            ].into_boxed_slice()),
            children: None,
        }));
    }

    #[test]
    fn test_remove_many() {
        let left = el!(div[
            key=0,
            el!(div[key=1]),
            el!(div[key=2]),
            el!(div[key=3])
        ]);

        let right = el!(div[
            key=0,
            el!(div[key=1])
        ]);
        let diff = left.diff(&right);

        assert_eq!(diff, Some(DiffTree{
            changes: Some(vec![
                Change::RemoveChild(Key::Local(2)),
                Change::RemoveChild(Key::Local(3)),
            ].into_boxed_slice()),
            children: None,
        }));
    }

    #[test]
    fn test_nested_remove() {
        let left = el!(div[
            key=0,
            el!(div[
                key=0,
                el!(div[])
            ])
        ]);

        let right = el!(div[
            key=0,
            el!(div[])
        ]);

        let diff = left.diff(&right);

        assert_eq!(diff, Some(DiffTree{
            changes: None,
            children: Some(vec![
                (Key::Local(0), DiffTree{
                    changes: Some(vec![
                        Change::ReplaceNode(el!(div[]))
                    ].into_boxed_slice()),
                    children: None,
                })
            ].into_boxed_slice()),
        }));
    }

    #[test]
    fn test_insert_single() {
        let left = el!(div[
            key=0,
            el!(div[key=1]),
            el!(div[key=2])
        ]);

        let right = el!(div[
            key=0,
            el!(div[key=0]),
            el!(div[key=1]),
            el!(div[key=2])
        ]);

        let diff = left.diff(&right);

        assert_eq!(diff, Some(DiffTree{
            changes: Some(vec![
                Change::InsertChild(el!(div[key=0])),
                Change::SortChildren(vec![
                    Key::Local(0),
                    Key::Local(1),
                    Key::Local(2),
                ].into_boxed_slice()),
            ].into_boxed_slice()),
            children: None,
        }));
    }

}
