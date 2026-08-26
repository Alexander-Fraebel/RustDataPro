use egui::{Color32, Response, RichText, Ui};
use egui_file_dialog::FileDialog;
use std::path::PathBuf;

macro_rules! simple_custom_button {
    ($ui:expr, $text:ident, $fill:expr) => {
        $ui.add_sized(
            DEFAULT_LARGE_BUTTOM_DIMS,
            egui::Button::new(egui::RichText::new($text).color(egui::Color32::BLACK)).fill($fill),
        )
    };
}

pub const DEFAULT_LARGE_BUTTOM_DIMS: (f32, f32) = (110.0, 40.0);
pub trait DataProUiElements {
    fn large_button(&mut self, text: &'static str) -> Response;
    fn large_green_button(&mut self, text: &'static str) -> Response;
    fn large_red_button(&mut self, text: &'static str) -> Response;
    fn large_blue_button(&mut self, text: &'static str) -> Response;
    fn lock_unlock_button(&mut self, condition: &mut bool);
    fn directory_picker(&mut self, file_dialog: &mut FileDialog, directory_name: &PathBuf);
}

impl DataProUiElements for Ui {
    fn large_button(&mut self, text: &'static str) -> Response {
        self.add_sized(
            DEFAULT_LARGE_BUTTOM_DIMS,
            egui::Button::new(RichText::new(text)),
        )
    }

    fn large_green_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(self, text, Color32::LIGHT_GREEN)
    }

    fn large_red_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(self, text, Color32::LIGHT_RED)
    }

    fn large_blue_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(self, text, Color32::LIGHT_BLUE)
    }

    /// Small button that shows an unlocked icon when condition is true and a locked icon when condition is false. Toggles condition on click.
    fn lock_unlock_button(&mut self, condition: &mut bool) {
        if *condition {
            if self.small_button("🔓").clicked() {
                *condition = false;
            }
        } else {
            if self.small_button("🔒").clicked() {
                *condition = true;
            }
        }
    }

    fn directory_picker(&mut self, file_dialog: &mut FileDialog, directory_name: &PathBuf) {
        if self
            .add(
                egui::Button::new(egui::RichText::new(directory_name.to_string_lossy())).truncate(),
            )
            .on_hover_text(directory_name.to_string_lossy())
            .clicked()
        {
            file_dialog.pick_directory();
        }
    }

    // fn return_button<F>(&mut self, app: &mut DataPro, mut closure: F)
    // where
    //     F: FnMut(&mut DataPro),
    // {
    //     if self.large_red_button("RETURN").clicked() {
    //         closure(app);
    //         app.go_to_prep_session();
    //     }
    // }
}
