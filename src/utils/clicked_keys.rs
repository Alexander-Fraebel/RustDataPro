use egui::{InputState, Key};
use itertools::Itertools;

/// Detect keys that have been pressed and ignore repeated events.
pub struct ClickedKeys(Vec<Key>);

impl ClickedKeys {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn contains_key(&self, key: &Key) -> bool {
        self.0.iter().contains(key)
    }

    pub fn update(&mut self, input: &InputState) {
        self.clear();

        for event in &input.events {
            if let egui::Event::Key {
                key,
                physical_key: _,
                pressed,
                repeat,
                modifiers: _,
            } = event
            {
                if *repeat {
                    continue;
                }
                if *pressed {
                    self.0.push(*key);
                }
            }
        }
    }
}
