use crate::{
    app::DataPro,
    configs::{
        ASSESSMENTS_FILE_NAME, CLIENT_DATA_FILE_NAME, IOA_DATA_FOLDER_NAME, KSF_FILE_NAME,
        SESSION_DATA_FOLDER_NAME,
    },
    data::{AssessmentsData, ClientData, KsfData},
    ui_elements::DataProUiElements,
    utils::windows_error_dialog,
};
use anyhow::Result;
use chrono::Local;
use egui::{Color32, RichText};
use rand::{RngExt, make_rng, rngs::StdRng};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

pub struct NewClient {
    prng: StdRng,
    client: ClientData,
    created: bool,
}

impl Default for NewClient {
    fn default() -> Self {
        Self {
            prng: make_rng(),
            client: ClientData::default(),
            created: false,
        }
    }
}

impl NewClient {
    fn create_new_client_folder(&mut self, root_directory: &PathBuf) -> Result<()> {
        let client_path = Path::new(root_directory).join(self.client.id.to_string());

        // Create a new directory for the client inside the root
        std::fs::create_dir(&client_path)?;

        // Create the Session Data folder
        std::fs::create_dir(Path::new(&client_path.join(SESSION_DATA_FOLDER_NAME)))?;

        // Create the IOA folder
        std::fs::create_dir(Path::new(&client_path.join(IOA_DATA_FOLDER_NAME)))?;

        // Create the client file inside the new directory, title it client_data.txt
        let mut writer = BufWriter::new(File::create_new(Path::new(
            &client_path.join(CLIENT_DATA_FILE_NAME),
        ))?);
        writer.write_all(self.client.to_json()?.as_bytes())?;
        writer.flush()?;

        // Create a default assessments file with an FA and conditions
        let mut writer = File::create_new(Path::new(&client_path.join(ASSESSMENTS_FILE_NAME)))?;
        writer.write_all(AssessmentsData::fa_conditions().to_json()?.as_bytes())?;
        writer.flush()?;

        // Create a template KSF file
        let mut writer = File::create_new(Path::new(&client_path.join(KSF_FILE_NAME)))?;
        writer.write_all(KsfData::initial_file().to_json()?.as_bytes())?;
        writer.flush()?;

        Ok(())
    }

    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Create a New Client");
            ui.add_space(10.0);
            egui::Grid::new("client_and_session_info_grid")
                .min_col_width(150.0)
                .spacing((10.0, 10.0))
                .show(ui, |ui| {
                    ui.monospace("Client ID");
                    ui.text_edit_singleline(&mut app.new_client_page.client.id);
                    if ui.button("random").clicked() {
                        app.new_client_page.client.id = format!(
                            "{:0<10}",
                            app.new_client_page
                                .prng
                                .random_range(1000000000_i64..=9999999999)
                        )
                    }
                    ui.end_row();

                    ui.monospace("Client Name");
                    ui.text_edit_singleline(&mut app.new_client_page.client.name);
                    ui.end_row();

                    ui.monospace("Case Manager");
                    ui.text_edit_singleline(&mut app.new_client_page.client.case_manager);
                    ui.end_row();

                    ui.monospace("Primary Therapist");
                    ui.text_edit_singleline(&mut app.new_client_page.client.primary_therapist);
                    ui.end_row();

                    ui.monospace("Date of Admission\n(YYYY-MM-DD)");
                    ui.text_edit_singleline(&mut app.new_client_page.client.date_of_admission);
                    if ui.button("today").clicked() {
                        app.new_client_page.client.date_of_admission =
                            Local::now().date_naive().format("%Y-%m-%d").to_string();
                    }
                    ui.end_row();

                    ui.monospace("Location");
                    ui.text_edit_singleline(&mut app.new_client_page.client.location);
                    ui.end_row();
                });

            ui.add_enabled_ui(!app.new_client_page.client.id.is_empty(), |ui| {
                if ui
                    .large_green_button("Save")
                    .on_disabled_hover_text("client must have an ID assigned")
                    .clicked()
                {
                    app.new_client_page.client.trim_all_fields();
                    if let Err(e) = app
                        .new_client_page
                        .create_new_client_folder(&app.root_directory)
                    {
                        windows_error_dialog(e);
                    }
                }
            });

            if ui.large_red_button("Return").clicked() {
                app.display_info.go_to_prep_session();
            }

            if app.new_client_page.created {
                ui.monospace(
                    RichText::new("New Client Created!")
                        .heading()
                        .color(Color32::GREEN),
                );
            }
        });
    }
}
