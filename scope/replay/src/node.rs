use std::mem;

use crate::event::*;

pub type NId = usize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeIdGenerator {
    next: NId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tag {
    id: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    id: NId,
    pub node_type: NodeType,
    parent_id: Option<NId>,
    // Index in parent.
    index: usize,
    children: Vec<NId>,
    num_child_values: usize,
    tags: Vec<Tag>,
    value: Value,
    is_complex: bool,
    display_type: DisplayType,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NodeType {
    MemCell,
    NamedContainer,
}

impl Node {
    pub fn new(id: NId,
               value: Value,
               parent_id: Option<NId>,
               index: usize,
               display_type: DisplayType) -> Node {
        let is_complex = !value.is_simple();

        Node {
            id,
            node_type: NodeType::MemCell,
            parent_id,
            index,
            children: Vec::new(),
            num_child_values: 0,
            tags: Vec::new(),
            value,
            is_complex,
            display_type,
        }
    }

    pub fn new_named_container(id: NId, value: Value) -> Node {
        let is_complex = !value.is_simple();

        Node {
            id,
            node_type: NodeType::NamedContainer,
            parent_id: None,
            index: 0,
            children: Vec::new(),
            num_child_values: 0,
            tags: Vec::new(),
            value,
            is_complex,
            display_type: DisplayType::Default,
        }
    }

    pub fn reset(&mut self) {
        self.num_child_values = 0;
        self.tags.clear();
        self.set_value(Value::UndefinedVal);
    }

    pub fn id(&self) -> NId {
        self.id
    }

    pub fn parent_id(&self) -> Option<&NId> {
        self.parent_id.as_ref()
    }

    pub fn set_parent_id(&mut self, parent_id: Option<NId>) {
        self.parent_id = parent_id;
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn children(&self) -> &Vec<NId> {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Vec<NId> {
        &mut self.children
    }

    pub fn num_children(&self) -> usize {
        self.num_child_values
    }

    pub fn increment_num_children(&mut self) {
        self.num_child_values += 1;
        assert!(self.num_child_values <= self.children.len());
    }

    pub fn decrement_num_children(&mut self) {
        assert!(self.num_child_values > 0);
        self.num_child_values -= 1;
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn set_value(&mut self, value: Value) {
        self.value = value;
    }

    pub fn replace_value(&mut self, value: Value) -> Value {
        mem::replace(&mut self.value, value)
    }

    pub fn should_show_name(&self) -> bool {
        self.is_ever_complex() || self.value != Value::UndefinedVal
    }

    pub fn should_show_value(&self) -> bool {
        match self.display_type {
            DisplayType::Default => self.is_always_simple() && self.value != Value::UndefinedVal,
            DisplayType::Tree => true,
        }
    }

    pub fn is_always_simple(&self) -> bool {
        !self.is_ever_complex()
    }

    pub fn is_ever_complex(&self) -> bool {
        self.is_complex
    }

    pub fn set_complex(&mut self) {
        self.is_complex = true;
    }

    pub fn has_index_label(&self) -> bool {
        self.parent_id.is_some()
    }

    pub fn display_type(&self) -> DisplayType {
        self.display_type
    }

    pub fn set_display_type(&mut self, display_type: DisplayType) {
        self.display_type = display_type;
    }
}

impl NodeIdGenerator {
    pub fn new() -> NodeIdGenerator {
        NodeIdGenerator { next: 0 }
    }

    pub fn next(&mut self) -> NId {
        let cur = self.next;
        let n = self.next.checked_add(1).expect("NodeIdGenerator::next: integer overflow");
        self.next = n;

        cur
    }
}
