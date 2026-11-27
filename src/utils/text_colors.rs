use egui::{Color32, RichText, Ui};

pub fn red_text<T: Into<String>>(text: T) -> RichText {
    RichText::new(text).color(Color32::RED)
}

pub fn warning_text<T: Into<String>>(text: T, ui: &Ui) -> RichText {
    RichText::new(text).color(ui.visuals().warn_fg_color)
}
