use crate::Node;
use bevy_app::{EventReader, Events};
use bevy_core::FloatOrd;
use bevy_ecs::prelude::*;
use bevy_input::{mouse::MouseButton, Input};
use bevy_math::Vec2;
use bevy_transform::components::Transform;
use bevy_window::{CursorMoved, Windows};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Click {
    Released,
    Pressed,
}

impl Default for Click {
    fn default() -> Self {
        Click::Released
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Hover {
    Hovered,
    NotHovered,
}

impl Default for Hover {
    fn default() -> Self {
        Hover::NotHovered
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FocusPolicy {
    Block,
    Pass,
}

impl Default for FocusPolicy {
    fn default() -> Self {
        FocusPolicy::Block
    }
}

#[derive(Default)]
pub struct State {
    cursor_moved_event_reader: EventReader<CursorMoved>,
    cursor_position: Vec2,
}

pub fn ui_focus_system(
    mut state: Local<State>,
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    mut node_query: Query<(
        &Node,
        &Transform,
        Option<&mut Click>,
        Option<&mut Hover>,
        Option<&FocusPolicy>,
    )>,
) {
    if let Some(cursor_moved) = state.cursor_moved_event_reader.latest(&cursor_moved_events) {
        state.cursor_position = cursor_moved.position;
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        for (_node, _transform, click, _hover, _focus_policy) in &mut node_query.iter() {
            if let Some(mut click) = click {
                if *click == Click::Pressed {
                    *click = Click::Released;
                }
            }
        }
    }

    let mouse_clicked = mouse_button_input.just_pressed(MouseButton::Left);
    let window = windows.get_primary().unwrap();

    let mut query_iter = node_query.iter();
    let mut moused_over_z_sorted_nodes = query_iter
        .iter()
        .filter_map(|(node, transform, click, hover, focus_policy)| {
            let position = transform.value.w_axis();
            // TODO: ui transform is currently in world space, so we need to move it to ui space. we should make these transforms ui space
            let ui_position = position.truncate().truncate()
                + Vec2::new(window.width as f32 / 2.0, window.height as f32 / 2.0);
            let extents = node.size / 2.0;
            let min = ui_position - extents;
            let max = ui_position + extents;
            // if the current cursor position is within the bounds of the node, consider it for clicking
            if (min.x()..max.x()).contains(&state.cursor_position.x())
                && (min.y()..max.y()).contains(&state.cursor_position.y())
            {
                Some((focus_policy, click, hover, FloatOrd(position.z())))
            } else {
                if let Some(mut hover) = hover {
                    if *hover == Hover::Hovered {
                        *hover = Hover::NotHovered;
                    }
                }
                None
            }
        })
        .collect::<Vec<_>>();

    // TODO: sort by negative when we move back to a right handed coordinate system
    moused_over_z_sorted_nodes.sort_by_key(|(_, _, _, z)| *z);
    for (focus_policy, click, hover, _) in moused_over_z_sorted_nodes {
        if mouse_clicked {
            // only consider nodes with ClickState "clickable"
            if let Some(mut click) = click {
                if *click == Click::Released {
                    *click = Click::Pressed;
                }
            }
        }
        // only consider nodes with Hover "hoverable"
        if let Some(mut hover) = hover {
            if *hover == Hover::NotHovered {
                *hover = Hover::Hovered;
            }
        }
        match focus_policy.cloned().unwrap_or(FocusPolicy::Block) {
            FocusPolicy::Block => {
                break;
            }
            FocusPolicy::Pass => { /* allow the next node to be hovered/clicked */ }
        }
    }
}