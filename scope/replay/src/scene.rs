extern crate gfx_device_gl;

use gfx_device_gl::Device;
use piston_window::*;

use crate::core::*;
use crate::math;
use crate::node::{NId, NodeType};
use crate::state::*;

const WINDOW_MARGIN_Y: Scalar = 5.0;
const CELL_MARGIN_X: Scalar = 20.0;

pub fn draw<'font>(state: &mut AppState<'font>,
                   ctx: Context,
                   g: &mut G2d,
                   device: &mut Device) {

    let cell_width = state.cell_width;
    let cell_height = state.cell_height;
    for ((node, position), draw_state) in state.graph.nodes_iter().zip(&state.positions).zip(&state.draw_states) {
        let opacity_mix_val = math::mix_scalar(draw_state.from_opacity, draw_state.to_opacity, math::quadratic_out(draw_state.mix));
        if node.should_show_name() {
            // Named cell label.
            let x = position[0];
            let y = position[1] + WINDOW_MARGIN_Y;
            let label_transform = ctx.transform.trans(x, y + 15.0);
            let label_str = state.graph.node_name(node).unwrap_or("");
            let mut name_color = state.cell_label_color.clone();
            name_color[3] = opacity_mix_val as f32;
            let name_text = Text::new_color(name_color, 12);
            name_text.draw(label_str,
                &mut state.glyphs,
                &ctx.draw_state,
                label_transform,
                g).expect("Draw text failed");
        }
        if !node.should_show_value() {
            continue;
        }

        let mut pos = value_abs_position(&state, node.id());

        if node.has_index_label() {
            // Array index.
            let index_label_width = state.index_label_width;

            let mut label_color = state.cell_label_color.clone();
            let index_str = node.index().to_string();
            label_color[3] = opacity_mix_val as f32;
            let text = Text::new_color(label_color, 10);
            // TODO: This should be right-aligned.
            let text_offset_x = if index_str.len() <= 1 {
                0.0
            } else {
                -14.0
            };
            let transform = ctx.transform.trans(pos[0] + text_offset_x, pos[1] + 14.0);
            text.draw(&index_str,
                      &mut state.glyphs,
                      &ctx.draw_state,
                      transform,
                      g).expect("Draw text failed");

            pos[0] += index_label_width;
        }
        // Cell background.
        let mut bg_color = state.cell_bg_color.clone();
        bg_color[3] = opacity_mix_val as f32;
        piston_window::rectangle(bg_color, [pos[0], pos[1], cell_width, cell_height], ctx.transform, g);
        // Cell value.
        let mut cell_value_color = state.cell_value_color.clone();
        let to_str = &draw_state.to_text;
        let text_mix_val = if draw_state.from_opacity <= draw_state.to_opacity {
            math::mix_scalar(0.0, draw_state.to_opacity, math::quadratic_out(draw_state.mix))
        } else {
            opacity_mix_val
        };
        cell_value_color[3] = text_mix_val as f32;
        let text = Text::new_color(cell_value_color, 12);
        let transform = ctx.transform.trans(pos[0], pos[1] + 15.0);
        text.draw(&to_str,
                  &mut state.glyphs,
                  &ctx.draw_state,
                  transform,
                  g).expect("Draw text failed");
    }
    // Update glyphs before rendering.
    state.glyphs.factory.encoder.flush(device);
}

fn value_abs_position<'a>(state: &AppState<'a>, mut node_id: NId) -> Vec2d {
    let mut pos: Vec2d = [0.0, WINDOW_MARGIN_Y];
    // We don't want to include the width of the given node, only of its
    // parents.
    let rel_pos = state.positions[node_id];
    pos[0] += rel_pos[0];
    pos[1] += rel_pos[1];
    let label_width = state.draw_states[node_id].label_width;
    pos[0] += label_width; // Should only be non-zero for NodeType::NamedContainer.
    let node = &state.graph.node(node_id);
    match node.parent_id() {
        None => return pos,
        Some(parent_id) => {
            node_id = *parent_id;
        }
    }
    loop {
        let rel_pos = &state.positions[node_id];
        pos[0] += rel_pos[0];
        pos[1] += rel_pos[1];
        let label_width = state.draw_states[node_id].label_width;
        pos[0] += label_width; // Should only be non-zero for NodeType::NamedContainer.
        let node = &state.graph.node(node_id);
        match node.node_type {
            NodeType::MemCell => {
                pos[0] += state.cell_width + CELL_MARGIN_X;
            },
            NodeType::NamedContainer => (),
        }
        match node.parent_id() {
            None => break,
            Some(parent_id) => {
                node_id = *parent_id;
            }
        }
    }

    pos
}
