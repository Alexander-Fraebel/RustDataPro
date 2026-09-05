use crate::{
    app::DataPro,
    config::{
        ASSESSMENTS_FILE_NAME, CLIENT_DATA_FILE_NAME, IOA_DATA_FOLDER_NAME, KSF_FILE_NAME,
        SESSION_DATA_FOLDER_NAME,
    },
    data::{AssessmentsData, ClientData, KsfsData},
    quick_error,
    ui_elements::DataProUiElements,
};
use anyhow::Result;
use chrono::Local;
use egui::{Color32, RichText};
use rand::RngExt;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

pub struct CreateClient {
    client: ClientData,
    created: bool,
}

impl Default for CreateClient {
    fn default() -> Self {
        Self {
            client: ClientData::default(),
            created: false,
        }
    }
}

impl CreateClient {
    fn create_new_client_folder(&mut self, root_directory: &PathBuf) -> Result<()> {
        let client_path = Path::new(root_directory).join(self.client.id.to_string());

        // Do these first to catch errors before any files and folders are created.
        let ksf = KsfsData::example().to_json()?;
        let assessments = AssessmentsData::example().to_json()?;
        let client = self.client.to_json()?;

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
        writer.write_all(client.as_bytes())?;
        writer.flush()?;

        // Create a default assessments file with an FA and conditions
        let mut writer = File::create_new(Path::new(&client_path.join(ASSESSMENTS_FILE_NAME)))?;
        writer.write_all(assessments.as_bytes())?;
        writer.flush()?;

        // Create a template KSF file
        let mut writer = File::create_new(Path::new(&client_path.join(KSF_FILE_NAME)))?;
        writer.write_all(ksf.as_bytes())?;
        writer.flush()?;

        Ok(())
    }
}

impl DataPro {
    pub fn view_create_client(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Create a New Client");
            ui.add_space(10.0);
            egui::Grid::new("client_and_session_info_grid")
                .min_col_width(150.0)
                .spacing((10.0, 10.0))
                .show(ui, |ui| {
                    ui.monospace("Client ID");
                    if ui
                        .text_edit_singleline(&mut self.new_client_page.client.id)
                        .changed()
                    {
                        self.new_client_page.created = false;
                    };
                    if cfg!(debug_assertions) {
                        if ui
                            .button(RichText::from("random").color(ui.visuals().warn_fg_color))
                            .clicked()
                        {
                            self.new_client_page.client.id = format!(
                                "{:0<10}",
                                self.rng.random_range(1000000000_i64..=9999999999) // collisions become higly likely after created 94868 IDs, alphanumeric might be better
                            );
                            self.new_client_page.created = false;
                        }
                    }
                    ui.end_row();

                    ui.monospace("Client Name");
                    if ui
                        .text_edit_singleline(&mut self.new_client_page.client.name)
                        .changed()
                    {
                        self.new_client_page.created = false;
                    }
                    ui.end_row();

                    ui.monospace("Case Manager");
                    if ui
                        .text_edit_singleline(&mut self.new_client_page.client.case_manager)
                        .changed()
                    {
                        self.new_client_page.created = false;
                    }
                    ui.end_row();

                    ui.monospace("Primary Therapist");
                    if ui
                        .text_edit_singleline(&mut self.new_client_page.client.primary_therapist)
                        .changed()
                    {
                        self.new_client_page.created = false;
                    }
                    ui.end_row();

                    ui.monospace("Date of Admission\n(MM-DD-YYYY)");
                    if ui
                        .text_edit_singleline(&mut self.new_client_page.client.date_of_admission)
                        .changed()
                    {
                        self.new_client_page.created = false;
                    }
                    if ui.button("today").clicked() {
                        self.new_client_page.client.date_of_admission =
                            Local::now().date_naive().format("%m-%d-%Y").to_string();
                        self.new_client_page.created = false;
                    }
                    ui.end_row();

                    ui.monospace("Location");
                    if ui
                        .text_edit_singleline(&mut self.new_client_page.client.location)
                        .changed()
                    {
                        self.new_client_page.created = false;
                    }
                    ui.end_row();
                });

            ui.add_enabled_ui(!self.new_client_page.client.id.is_empty(), |ui| {
                if ui
                    .large_green_button("SAVE")
                    .on_disabled_hover_text("client must have an ID assigned")
                    .clicked()
                {
                    self.new_client_page.client.trim_all_fields();
                    quick_error!(
                        self.new_client_page
                            .create_new_client_folder(&self.root_directory)
                    );
                    self.new_client_page.created = true;
                }
            });
            ui.add_space(5.0);

            if self.new_client_page.created {
                ui.monospace(
                    RichText::new("New Client Created!")
                        .heading()
                        .color(Color32::GREEN),
                );
            }
        });
    }
}
