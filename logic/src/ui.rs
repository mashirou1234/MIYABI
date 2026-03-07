// logic/src/ui.rs
use serde::{Deserialize, Serialize};

use crate::ffi::{Vec2, Vec4};
use crate::Game;

// 1. Define the Button Component
// =============================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Checks if a point is inside the rectangle.
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Button {
    pub rect: Rect,
    pub text: String,
    pub action_id: String,
    // pub font_size: f32,
    // pub color: Vec4,
    // pub hover_color: Vec4,
    // pub pressed_color: Vec4,
}

// We need a way to register this as a component.
// We'll add a new ComponentType for it.
use crate::Component;
use crate::ComponentType;

impl Component for Button {
    const COMPONENT_TYPE: ComponentType = ComponentType::Button;
}

// 2. UI System Logic
// ==================

/// The UI system handles button hit-testing and drawing, and returns the clicked action id.
pub fn ui_system(game: &mut Game) -> Option<String> {
    let mouse_pos = game.input_state.mouse_pos;
    let mouse_clicked = game.input_state.mouse_clicked;
    let mut queued_action_id: Option<String> = None;

    // Find archetypes with a Button component
    for archetype in &game.world.archetypes {
        if archetype.types.contains(&ComponentType::Button) {
            let buttons = archetype
                .storage
                .get(&ComponentType::Button)
                .unwrap()
                .downcast_ref::<Vec<Button>>()
                .unwrap();

            for button in buttons.iter() {
                // --- Interaction Logic ---
                if mouse_clicked && queued_action_id.is_none() && button.rect.contains(mouse_pos) {
                    queued_action_id = Some(button.action_id.clone());
                }

                // --- Drawing Logic ---
                // For now, just draw the text. A more complex system would also draw the button's rectangle.
                game.text_commands.push(crate::ffi::TextCommand {
                    text: button.text.clone(),
                    // Center the text roughly
                    position: Vec2 {
                        x: button.rect.x + (button.rect.width / 2.0)
                            - (button.text.len() as f32 * 6.0), // Estimate
                        y: button.rect.y + (button.rect.height / 2.0) - 8.0, // Estimate
                    },
                    font_size: 24.0,
                    color: Vec4 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                        w: 1.0,
                    },
                });
            }
        }
    }

    queued_action_id
}
