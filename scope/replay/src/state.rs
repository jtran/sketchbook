use std::collections::HashMap;

use piston_window::{G2dTexture, G2dTextureContext, Text};
use piston_window::glyph_cache::rusttype::GlyphCache;
use piston_window::types::Color;

use crate::core::*;
use crate::event::*;
use crate::node::*;
use crate::op::*;

const NAMED_CELL_MARGIN_X: Scalar = 30.0;
const NAMED_CELL_MARGIN_Y: Scalar = 10.0;
const CELL_MARGIN_Y: Scalar = 10.0;

#[derive(Clone, Debug, PartialEq)]
pub struct ProgramGraph {
    gensym: NodeIdGenerator,
    nodes: Vec<Node>,
    ids_by_name: HashMap::<String, NId>,
    names_by_id: HashMap::<NId, String>,
}

pub struct AppState<'font> {
    pub graph: ProgramGraph,

    // Components.

    // Reversible operations.
    pub ops: Vec<Op>,
    pub op_index: usize,
    // Coordinates of each entity, relative to its parent.
    pub positions: Vec<Vec2d>,
    // State used only for drawing.
    pub draw_states: Vec<AppDrawState>,
    // High-level grid coordinates of named cells.
    pub grid_coords: Vec<Option<GridCoord>>,

    // View state.

    pub bg_color: Color,
    pub named_cell_label_width: Scalar,
    pub named_cell_height: Scalar,
    pub index_label_width: Scalar,
    pub cell_bg_color: Color,
    pub cell_label_color: Color,
    pub cell_value_color: Color,
    pub cell_label_text: Text,
    pub cell_value_text: Text,
    pub cell_width: Scalar,
    pub cell_height: Scalar,
    pub glyphs: GlyphCache<'font, G2dTextureContext, G2dTexture>,
}

// This is used for drawing transitions.  The state on the model may change, but
// we need to keep the old value around so that we can draw the transition.
#[derive(Clone, Debug, PartialEq)]
pub struct AppDrawState {
    pub label_width: Scalar,
    pub from_opacity: Scalar,
    pub to_opacity: Scalar,
    pub from_text: String,
    pub to_text: String,
    pub mix: Scalar,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChangeType {
    NoChange,
    Parallel { changes: Vec<ChangeType> },
    // Something changed, but it doesn't affect the layout.
    ValueChange { id: NId, from: String, to: String },
    AddCell { id: NId },
    RemoveCell { id: NId },
    // The layout and possibly eveything else changed.
    #[allow(dead_code)]
    LayoutChange,
}

impl<'font> AppState<'font> {
    pub fn new(glyphs: GlyphCache<'font, G2dTextureContext, G2dTexture>) -> AppState {
        let mut ops = Vec::with_capacity(256);
        // Since our index can't point between ops in the vector, we always keep
        // a no-op at the beginning so we can point to the real beginning.
        ops.push(Op { forward: OpStep::NoOp, reverse: OpStep::NoOp });

        let cell_label_color = [0.58, 0.58, 0.58, 1.0];
        let cell_value_color = [0.97, 0.97, 0.95, 1.0];

        AppState {
            graph: ProgramGraph::new(),
            ops,
            op_index: 0,
            positions: Vec::new(),
            draw_states: Vec::new(),
            grid_coords: Vec::new(),
            bg_color: [0.26, 0.26, 0.24, 1.0],
            named_cell_label_width: 60.0,
            named_cell_height: 20.0,
            index_label_width: 15.0,
            cell_bg_color: [0.16, 0.16, 0.14, 1.0],
            cell_label_color,
            cell_value_color,
            cell_label_text: Text::new_color(cell_label_color, 12),
            cell_value_text: Text::new_color(cell_value_color, 12),
            cell_width: 120.0,
            cell_height: 20.0,
            glyphs,
        }
    }

    pub fn init_layout(&mut self) {
        self.positions.resize(self.graph.nodes.len(), [0.0, 0.0]);
        self.grid_coords.resize(self.graph.nodes.len(), None);
    }

    pub fn update_layout(&mut self) {
        self.init_layout();
        // For each column, store how many elements we've placed there.
        let mut cols: Vec<usize> = Vec::new();
        let mut col_label_widths = HashMap::<usize, Scalar>::new();
        let mut col_widths = HashMap::<usize, Scalar>::new();
        for (i, node) in self.graph.nodes.iter().enumerate() {
            match node.node_type {
                NodeType::MemCell => continue,
                NodeType::NamedContainer => (),
            }
            let grid_coord = if !node.is_ever_complex() && (cols.is_empty() || cols[0] < 10) {
                // Stack multiple primitives on top of each other in the first
                // column, up to a point.
                if cols.is_empty() { cols.push(0); }
                let row = cols[0];
                cols[0] += 1;

                [0, row]
            } else {
                let index = cols.len();
                // Since this is a large object like an array, it could take up
                // arbitrary vertical space in the column.
                cols.push(std::usize::MAX);

                [index, 0]
            };
            self.grid_coords[i] = Some(grid_coord);
            let col = grid_coord[0];
            // Calculate the width of this node's label to get the label width
            // of each column.
            let label_width = self.graph.node_name(node).map(|label_str| {
                // TODO: actually measure based on the font.
                10.0 * label_str.len() as Scalar
            }).unwrap_or(0.0);
            let max_label_width = *col_label_widths.entry(col).or_insert(0.0);
            if label_width > max_label_width {
                col_label_widths.insert(col, label_width);
            }
            // Calculate the width of this node to get the width of each column.
            let width = self.measure_width(&node, label_width);
            let max_width = *col_widths.entry(col).or_insert(0.0);
            if width > max_width {
                col_widths.insert(col, width);
            }
        }
        // Calculate absolute x of each column.
        let mut abs_x_for_col = Vec::with_capacity(cols.len());
        let mut abs_x = 5.0;
        for col in 0..cols.len() {
            abs_x_for_col.push(abs_x);
            let col_width = *col_widths.get(&col).expect("col width not present");
            abs_x += col_width + NAMED_CELL_MARGIN_X;
        }
        // Set the absolute position of each container node.
        for (i, node) in self.graph.nodes.iter().enumerate() {
            match node.node_type {
                NodeType::MemCell => continue,
                NodeType::NamedContainer => (),
            }
            if let Some(grid_coord) = self.grid_coords[i] {
                let col = grid_coord[0];
                self.draw_states[node.id()].label_width = *col_label_widths.get(&col).expect("col label width not present");
                let x = abs_x_for_col[col];
                self.positions[i] = [x, 0.0];
            }
        }
    }

    fn measure_width(&self, node: &Node, label_width: Scalar) -> Scalar {
        let mut w = self.cell_width;
        w += label_width;
        if node.is_ever_complex() {
            w += self.index_label_width;
        }

        w
    }

    fn world_y_coord_from_grid_cell(&self, coord: GridCoord) -> Scalar {
        let cell_height = self.named_cell_height;
        let y = coord[1] as Scalar * (cell_height + NAMED_CELL_MARGIN_Y);

        y
    }

    pub fn player_reset_to_start(&mut self) {
        for node in self.graph.nodes.iter_mut() {
            node.reset();
        }
        self.op_index = 0;
    }

    pub fn init_draw_states(&mut self) {
        self.draw_states = self.graph.nodes.iter().map(|node| {
            AppDrawState {
                label_width: 0.0,
                from_opacity: 0.0,
                to_opacity: 0.0,
                from_text: "".to_string(),
                to_text: node.value().display_string(),
                mix: 1.0,
            }
        }).collect();
    }

    pub fn add_event(&mut self, event: Event) {
        if let Some(op) = self.graph.process(&event) {
            self.ops.push(op);
            self.op_index += 1;
        }
    }

    pub fn player_advance(&mut self) {
        self.player_step(StepDirection::Forward);
        eprintln!();
    }

    pub fn player_rewind(&mut self) {
        self.player_step(StepDirection::Reverse);
        eprintln!();
    }

    fn player_step(&mut self, direction: StepDirection) {
        if let Some((op_index, new_op_index)) = self.find_step_ops(direction) {
            let op = &self.ops[op_index];
            let op_step = op.step_in(direction);
            let changes = self.graph.step(op_step);
            self.op_index = new_op_index;
            self.process_changes(&changes, false);
        }
    }

    fn find_step_ops(&mut self, direction: StepDirection) -> Option<(usize, usize)> {
        match direction {
            StepDirection::Forward => {
                let new_index = self.op_index.saturating_add(1);
                if new_index == self.ops.len() {
                    return None;
                }

                Some((new_index, new_index))
            }
            StepDirection::Reverse => {
                let new_index = self.op_index.saturating_sub(1);
                if self.op_index == 0 {
                    return None;
                }

                Some((self.op_index, new_index))
            }
        }
    }

    fn process_changes(&mut self, change: &ChangeType, parallel: bool) {
        eprintln!("process_changes {:?}", change);
        match change {
            ChangeType::NoChange => (),
            ChangeType::Parallel { changes } => {
                for change in changes.iter() {
                    self.process_changes(change, true);
                }
            }
            ChangeType::AddCell { id } => {
                let id = *id;
                let node = &self.graph.nodes[id];
                eprintln!("  position before = {:?}", self.positions[id]);
                let y = if let Some(grid_coord) = &self.grid_coords[id] {
                    self.world_y_coord_from_grid_cell(*grid_coord)
                } else {
                    0.0
                };
                self.positions[id][1] = y + node.index() as Scalar * (self.cell_height + CELL_MARGIN_Y);
                eprintln!("  position after  = {:?}", self.positions[id]);
                let text = node.value().display_string();
                eprintln!("  id={}, parent_id={:?}, index={}, text={:?}", id, node.parent_id(), node.index(), &text);
                self.draw_states[id].from_opacity = 0.0;
                self.draw_states[id].to_opacity = 1.0;
                self.draw_states[id].from_text = text.clone();
                self.draw_states[id].to_text = text;
                self.draw_states[id].mix = 0.0;
            }
            ChangeType::RemoveCell { id } => {
                let id = *id;
                self.draw_states[id].from_opacity = 1.0;
                self.draw_states[id].to_opacity = 0.0;
                self.draw_states[id].mix = 0.0;
            }
            ChangeType::ValueChange { id, from, to } => {
                let id = *id;
                let node = &self.graph.nodes[id];
                eprintln!("  id={}, parent_id={:?}, index={}, text={:?}", id, node.parent_id(), node.index(), &to);
                if !parallel {
                    self.draw_states[id].from_opacity = 1.0;
                    self.draw_states[id].to_opacity = 1.0;
                }
                self.draw_states[id].from_text = from.to_string();
                self.draw_states[id].to_text = to.to_string();
                self.draw_states[id].mix = 0.0;
            }
            ChangeType::LayoutChange => self.update_layout(),
        }
    }
}

impl ProgramGraph {
    pub fn new() -> ProgramGraph {
        ProgramGraph {
            gensym: NodeIdGenerator::new(),
            // tags: HashMap::<String, Tag>::with_capacity(32),
            ids_by_name: HashMap::<String, NId>::with_capacity(128),
            names_by_id: HashMap::<NId, String>::with_capacity(128),
            nodes: Vec::with_capacity(128),
        }
    }

    pub fn nodes_iter(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter()
    }

    pub fn node(&self, node_id: NId) -> &Node {
        &self.nodes[node_id]
    }

    fn next_id(&mut self) -> NId {
        self.gensym.next()
    }

    fn add_unnamed_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    fn add_named_node(&mut self, name: String, node: Node) {
        self.ids_by_name.insert(name.clone(), node.id());
        self.names_by_id.insert(node.id(), name);
        self.nodes.push(node);
    }

    fn add_named_value_node(&mut self, name: String, value: Value) -> NId {
        let id = self.next_id();
        let node = Node::new_named_container(id, value);
        self.add_named_node(name, node);

        id
    }

    fn add_value_node(&mut self, value: Value, parent_id: Option<NId>, index: usize) -> NId {
        let display_type = parent_id.map(|id| self.nodes[id].display_type())
                                    .unwrap_or(DisplayType::Default);
        let id = self.next_id();
        let node = Node::new(id, value, parent_id, index, display_type);
        self.add_unnamed_node(node);

        id
    }

    pub fn node_name(&self, node: &Node) -> Option<&str> {
        self.names_by_id.get(&node.id()).map(|s| s.as_str())
    }

    fn node_id_by_name_implicit_declare(&mut self, name: &str, value: Value) -> (NId, OpStep) {
        match self.ids_by_name.get(name) {
            None => {
                eprintln!("Implicit node create: name={} value={:?}", name, &value);

                let new_id = self.add_named_value_node(name.to_string(), value);

                (new_id, OpStep::Define { id: new_id })
            }
            Some(id) => (*id, OpStep::NoOp),
        }
    }

    fn node_id_by_loc_implicit_declare(&mut self, loc: &Location, value: Value) -> (NId, OpStep) {
        match loc {
            Location::VariableLoc(name) => {
                self.node_id_by_name_implicit_declare(name, value)
            }
            Location::IndexLoc(loc, index) => {
                let (array_node_id, step) = self.node_id_by_loc_implicit_declare_array(loc);
                eprintln!("IndexLoc(loc={:?}, index={}) => {:?}", loc, index, &self.nodes[array_node_id]);

                let array_node = &mut self.nodes[array_node_id];
                let child_id = array_node.children()[*index];

                (child_id, step)
            }
        }
    }

    fn node_id_by_loc_implicit_declare_array(&mut self, loc: &Location) -> (NId, OpStep) {
        self.node_id_by_loc_implicit_declare(loc, Value::UndefinedVal)
    }

    fn node_push_implicit_create(&mut self, node_id: NId, value: Value) -> (NId, OpStep) {
        let node = &mut self.nodes[node_id];
        let capacity = node.children().len();
        let index = node.num_children();
        if index < capacity {
            // The node is already there.  Reuse it.
            node.increment_num_children();
            let child_id = node.children()[index];
            let child_node = &mut self.nodes[child_id];
            child_node.set_value(value.clone());

            return (child_id, OpStep::NoOp);
        }

        // Create a new node.
        let child_id = self.add_value_node(value.clone(), Some(node_id), index);
        let node = &mut self.nodes[node_id];
        node.children_mut().push(child_id);
        node.set_complex();
        node.increment_num_children();

        (child_id, OpStep::Define { id: child_id })
    }

    fn node_push_id(&mut self, node_id: NId, child_id: NId) {
        let node = &self.nodes[node_id];
        let index = node.num_children();
        let child_node = &mut self.nodes[child_id];
        child_node.set_parent_id(Some(node_id));
        child_node.set_index(index);
        let node = &mut self.nodes[node_id];
        node.children_mut().push(child_id);
        node.increment_num_children();
    }

    fn node_pop(&mut self, node_id: NId) -> Option<NId> {
        let node = &mut self.nodes[node_id];
        let len = node.num_children();
        if len == 0 {
            return None;
        }
        let child_id = node.children().get(len - 1).map(|n| *n).expect("child_id should be present");
        node.decrement_num_children();
        // Note: Do not unlink the child from the parent so that it can still be
        // drawn in the correct place.

        Some(child_id)
    }

    // Used by State.
    pub(self) fn process(&mut self, event: &Event) -> Option<Op> {
        match event {
            Event::NoOp => Some(Op { forward: OpStep::NoOp, reverse: OpStep::NoOp }),
            Event::Display(loc, display_type) => {
                let (node_id, step) = self.node_id_by_loc_implicit_declare(loc, Value::UndefinedVal);
                let node = &mut self.nodes[node_id];
                node.set_display_type(*display_type);

                Some(step.into_op())
            }
            Event::Set(loc, value) => {
                let (node_id, prev_step) = self.node_id_by_loc_implicit_declare(loc, Value::UndefinedVal);
                let node = &mut self.nodes[node_id];
                let from = node.replace_value(value.clone());
                let forward = OpStep::Set { id: node.id(), value: value.clone() };
                let reverse = OpStep::Set { id: node.id(), value: from };

                Some(Op::from_steps(forward, reverse, prev_step))
            }
            Event::Push(loc, value) => {
                let (node_id, step1) = self.node_id_by_loc_implicit_declare_array(loc);
                let (child_id, step2) = self.node_push_implicit_create(node_id, value.clone());
                let prev_step = step1.then(step2);
                let forward = OpStep::Push { id: node_id, child_id, value: value.clone() };

                Some(forward.into_op_with_previous(prev_step))
            }
            Event::Pop(loc) => {
                let (node_id, prev_step) = self.node_id_by_loc_implicit_declare_array(loc);
                let popped = self.node_pop(node_id);

                let forward = OpStep::Pop { id: node_id };
                let reverse = match popped {
                    None => OpStep::NoOp,
                    Some(popped_id) => {
                        let child_node = &self.nodes[popped_id];
                        let value = child_node.value().clone();

                        OpStep::Push { id: node_id, child_id: child_node.id(), value }
                    }
                };

                Some(Op::from_steps(forward, reverse, prev_step))
            }
        }
    }

    pub(self) fn step(&mut self, op_step: &OpStep) -> ChangeType {
        eprintln!("OpStep {:?}", op_step);
        match op_step {
            OpStep::NoOp => ChangeType::NoChange,
            OpStep::Atomic { steps } => {
                let changes: Vec<_> = steps.iter().map(|step| {
                    self.step(step)
                }).collect();

                ChangeType::Parallel { changes }
            }
            OpStep::Define { id } => {
                ChangeType::AddCell { id: *id }
            }
            OpStep::Forget { id } => {
                ChangeType::RemoveCell { id: *id }
            }
            OpStep::Set { id, value } => {
                let node = &mut self.nodes[*id];
                let from = node.value().display_string();
                // TODO: Should we be cloning here?
                node.set_value(value.clone());

                ChangeType::ValueChange { id: *id, from, to: value.display_string() }
            }
            OpStep::Push { id, child_id, value } => {
                let id = *id;
                let value = value.clone();
                let child_id = *child_id;
                let child = &mut self.nodes[child_id];
                child.set_value(value);
                self.node_push_id(id, child_id);

                ChangeType::AddCell { id: child_id }
            }
            OpStep::Pop { id } => {
                let id = *id;
                let child_id = self.node_pop(id);

                match child_id {
                    Some(child_id) => ChangeType::RemoveCell { id: child_id },
                    None => ChangeType::NoChange,
                }
            }
        }
    }
}
