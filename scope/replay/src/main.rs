extern crate find_folder;
extern crate piston_window;

mod core;
mod event;
mod input;
mod math;
mod node;
mod op;
mod scene;
mod state;

use piston_window::{PistonWindow, UpdateEvent, WindowSettings};

use state::*;
use event::{DisplayType, Event as NodeEvent, Location, Value};

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Replay", [800, 600])
        .build()
        .unwrap_or_else(|e| { panic!("Failed to build window: {}", e) });

    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").expect("Couldn't find assets folder");
    let glyphs = window.load_font(assets.join("fonts").join("liberation_mono").join("LiberationMono-Regular.ttf")).expect("Couldn't load font: liberation mono");

    let mut state = AppState::new(glyphs);
    state.add_event(NodeEvent::Set(Location::VariableLoc("num".to_string()), Value::I32Val(22)));
    state.add_event(NodeEvent::Set(Location::VariableLoc("factor".to_string()), Value::F64Val(7.5)));
    state.add_event(NodeEvent::Set(Location::VariableLoc("arr".to_string()), Value::ArrayVal(vec![
        Value::StringVal("alpha".to_string()),
        Value::StringVal("beta".to_string()),
        Value::StringVal("charlie".to_string()),
    ])));
    state.add_event(NodeEvent::Set(Location::VariableLoc("display".to_string()), Value::StringVal("runtime data".to_string())));
    let alphabet_loc = Location::VariableLoc("alphabet".to_string());
    state.add_event(NodeEvent::Push(alphabet_loc.clone(), Value::StringVal("A".to_string())));
    state.add_event(NodeEvent::Push(alphabet_loc.clone(), Value::StringVal("A".to_string())));
    state.add_event(NodeEvent::Set(Location::IndexLoc(Box::new(alphabet_loc.clone()), 1), Value::StringVal("B".to_string())));
    state.add_event(NodeEvent::Push(alphabet_loc.clone(), Value::StringVal("C".to_string())));
    state.add_event(NodeEvent::Pop(alphabet_loc.clone()));
    state.add_event(NodeEvent::Push(alphabet_loc.clone(), Value::StringVal("C".to_string())));
    state.add_event(NodeEvent::Push(alphabet_loc.clone(), Value::StringVal("D".to_string())));
    let arr_nested_loc = Location::VariableLoc("arr_nested".to_string());
    state.add_event(NodeEvent::Display(arr_nested_loc.clone(), DisplayType::Tree));
    state.add_event(NodeEvent::Push(arr_nested_loc.clone(), Value::StringVal("A".to_string())));
    state.add_event(NodeEvent::Push(arr_nested_loc.clone(), Value::ArrayVal(vec![])));
    state.add_event(NodeEvent::Push(Location::IndexLoc(Box::new(arr_nested_loc.clone()), 1), Value::StringVal("1 Nested in B".to_string())));
    state.add_event(NodeEvent::Push(Location::IndexLoc(Box::new(arr_nested_loc.clone()), 1), Value::StringVal("2 Nested in B".to_string())));
    state.add_event(NodeEvent::Push(arr_nested_loc.clone(), Value::StringVal("C".to_string())));
    state.add_event(NodeEvent::Push(arr_nested_loc.clone(), Value::StringVal("D".to_string())));
    state.add_event(NodeEvent::Set(Location::VariableLoc("num".to_string()), Value::I32Val(23)));
    state.add_event(NodeEvent::Set(Location::VariableLoc("num".to_string()), Value::I32Val(24)));
    state.add_event(NodeEvent::Set(Location::VariableLoc("num".to_string()), Value::I32Val(25)));
    state.add_event(NodeEvent::Set(Location::VariableLoc("display".to_string()), Value::StringVal("CHANGED".to_string())));
    state.player_reset_to_start();
    state.init_draw_states();
    state.update_layout();

    while let Some(event) = window.next() {
        // Handle input.
        input::handle(&mut state, &event);

        event.update(|args| {
            // TODO: Change this to 0.1 to run at 1/10th the speed.
            let transition_factor = 1.0;
            for ds in state.draw_states.iter_mut() {
                ds.mix = math::clamp(ds.mix + args.dt * transition_factor);
            }
        });

        window.draw_2d(&event, |ctx, g, device| {
            piston_window::clear(state.bg_color, g);
            scene::draw(&mut state, ctx, g, device);
        });
    }
}
