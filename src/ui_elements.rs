use crate::app::DataPro;
use egui::{Color32, Response, RichText, Ui};
use egui_file_dialog::FileDialog;
use std::path::PathBuf;

macro_rules! simple_custom_button {
    ($ui:expr, $text:ident, $fill:expr) => {
        $ui.add(
            egui::Button::new(RichText::new($text).monospace().color(Color32::BLACK)).fill($fill),
        )
    };
    (large, $ui:expr, $text:ident, $fill:expr) => {
        $ui.add_sized(
            DEFAULT_LARGE_BUTTOM_DIMS,
            egui::Button::new(RichText::new($text).color(Color32::BLACK)).fill($fill),
        )
    };
}

const DEFAULT_LARGE_BUTTOM_DIMS: (f32, f32) = (120.0, 40.0);
pub trait DataProUiElements {
    fn large_button(&mut self, text: &'static str) -> Response;
    fn large_green_button(&mut self, text: &'static str) -> Response;
    fn green_button(&mut self, text: &'static str) -> Response;
    fn large_red_button(&mut self, text: &'static str) -> Response;
    fn red_button(&mut self, text: &'static str) -> Response;
    fn large_blue_button(&mut self, text: &'static str) -> Response;
    fn blue_button(&mut self, text: &'static str) -> Response;
    fn lock_unlock_button(&mut self, condition: &mut bool);
    fn directory_picker(&mut self, file_dialog: &mut FileDialog, directory_name: &PathBuf);
    fn client_picker(&mut self, app: &mut DataPro, id_salt: &str);
}

impl DataProUiElements for Ui {
    fn large_button(&mut self, text: &'static str) -> Response {
        self.add_sized(DEFAULT_LARGE_BUTTOM_DIMS, egui::Button::new(text))
    }

    fn large_green_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(large, self, text, Color32::LIGHT_GREEN)
    }

    fn green_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(self, text, Color32::LIGHT_GREEN)
    }

    fn large_red_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(large, self, text, Color32::LIGHT_RED)
    }

    fn red_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(self, text, Color32::LIGHT_RED)
    }

    fn large_blue_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(large, self, text, Color32::LIGHT_BLUE)
    }

    fn blue_button(&mut self, text: &'static str) -> Response {
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
                egui::Button::new(
                    egui::RichText::new(directory_name.to_string_lossy()).monospace(),
                )
                .truncate(),
            )
            .on_hover_text(directory_name.to_string_lossy())
            .clicked()
        {
            file_dialog.pick_directory();
        }
    }

    fn client_picker(&mut self, app: &mut DataPro, id_salt: &str) {
        let client_picker_text = match app.data.client_loaded() {
            true => app.data.client.id.clone(),
            false => String::from("Choose Client"),
        };
        self.horizontal(|ui| {
            egui::ComboBox::from_id_salt(id_salt)
                .selected_text(RichText::new(client_picker_text).heading().strong())
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(&mut app.data.client.id, String::new(), "None")
                        .clicked()
                    {
                        app.data.clear();
                        app.edit_ksfs.prepare(&app.data);
                        app.edit_assessments.prepare(&app.data);
                    }
                    if let Ok(entries) = app.root_directory.read_dir() {
                        for entry in entries {
                            if let Ok(e) = entry {
                                if ui
                                    .selectable_value(
                                        &mut app.data.client.id,
                                        e.file_name().to_string_lossy().to_string(),
                                        e.file_name().to_string_lossy().to_string(),
                                    )
                                    .clicked()
                                {
                                    app.load_client_file(&e.path());
                                    app.edit_ksfs.prepare(&app.data);
                                    app.edit_assessments.prepare(&app.data);
                                }
                            }
                        }
                    }
                });
        });
    }
}
