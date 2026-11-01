use crate::{
    app::DataPro,
    config::path_to_config_file,
    data::SessionResults,
    quick_error,
    utils::{overwrite_file, ui_elements::DataProUiElements},
};

fn edit_box(
    ui: &mut egui::Ui,
    label: &str,
    source: &mut String,
    check_if_session_can_start: &mut bool,
    interactive: bool,
) -> egui::Response {
    ui.label(label);
    let widget = ui.add(egui::TextEdit::singleline(source).interactive(interactive));
    if widget.changed() {
        *check_if_session_can_start = true;
    }
    widget
}

const EXAMPLE_TEXT: &'static str = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\n\nMen act upon the world, and change it, and are changed in turn by the consequences of their actions. Certain processes, which the human organism shares with other species, alter behavior so that it achieves a safer and more useful interchange with a particular environment. When selfropriate behavior has been established, its consequences work through similar processes to keep it in force. If by chance the environment changes, old forms of behavior disselfear, while new consequences build new forms.";

impl DataPro {
    pub fn view_debug_page(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            self.client_picker(ui);

            egui::Grid::new("client_and_session_info_grid")
                .min_col_width(120.0)
                .min_row_height(22.0)
                .show(ui, |ui| {
                    let mut check_if_session_can_start = false;

                    // Saved to client file.
                    if edit_box(
                        ui,
                        "Location",
                        &mut self.data.client.location,
                        &mut check_if_session_can_start,
                        true,
                    )
                    .lost_focus()
                    {
                        check_if_session_can_start = true;
                        quick_error!(self.overwrite_client_data());
                    }
                    ui.end_row();

                    // Saved to client file.
                    ui.label("Date of Admission\n(MM-DD-YYYY)");
                    if self.prep_session.edit_doa {
                        let doa = ui.add(egui::TextEdit::singleline(
                            &mut self.data.client.date_of_admission,
                        ));
                        if doa.lost_focus() {
                            quick_error!(self.overwrite_client_data());
                        }
                        if doa.changed() {
                            check_if_session_can_start = true;
                        }
                        // The rest of these are non-editable display versions of the DOA
                    } else {
                        match self.data.client.days_since_admission() {
                            Ok(n) => {
                                // emphasize negative DOA with red text
                                if n.is_negative() {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut format!("{n} days ago"))
                                            .text_color(ui.visuals().error_fg_color)
                                            .interactive(false),
                                    );
                                } else {
                                    // normal DOA information
                                    ui.add(
                                        egui::TextEdit::singleline(&mut format!("{n} days ago"))
                                            .interactive(false),
                                    );
                                }
                            }
                            Err(_e) => {
                                // indicate invalid date with ERROR, red text, and hover text explanation
                                ui.add(
                                    egui::TextEdit::singleline(&mut format!("ERROR"))
                                        .text_color(ui.visuals().error_fg_color)
                                        .interactive(false),
                                );
                            }
                        }
                    }
                    if ui
                        .lock_unlock_button(&mut self.prep_session.edit_doa)
                        .clicked()
                    {
                        quick_error!(self.overwrite_client_data());
                        check_if_session_can_start = true;
                    }
                    ui.end_row();

                    // Saved to assessments file.
                    ui.label("Session Number");
                    let session_number = ui.add(
                        egui::DragValue::new(&mut self.data.current_session).range(1..=u32::MAX),
                    );
                    if session_number.lost_focus() {
                        quick_error!(self.overwrite_assessments());
                    }
                    if session_number.changed() {
                        let current_session = self.data.current_session;
                        if let Some(condtions) = self.data.chosen_assessment() {
                            condtions.session = current_session;
                        }
                        check_if_session_can_start = true;
                    }
                    ui.end_row();

                    // Saved to client file.
                    edit_box(
                        ui,
                        "Case Manager",
                        &mut self.data.client.case_manager,
                        &mut check_if_session_can_start,
                        self.prep_session.edit_case_manager,
                    );
                    if ui
                        .lock_unlock_button(&mut self.prep_session.edit_case_manager)
                        .clicked()
                    {
                        check_if_session_can_start = true;
                        quick_error!(self.overwrite_client_data());
                    };
                    ui.end_row();

                    // Saved to client file.
                    edit_box(
                        ui,
                        "Primary Therapist",
                        &mut self.data.client.primary_therapist,
                        &mut check_if_session_can_start,
                        self.prep_session.edit_primary_therapist,
                    );
                    if ui
                        .lock_unlock_button(&mut self.prep_session.edit_primary_therapist)
                        .clicked()
                    {
                        check_if_session_can_start = true;
                        quick_error!(self.overwrite_client_data());
                    }
                    ui.end_row();

                    // Not saved to any file.
                    edit_box(
                        ui,
                        "Session Therapist",
                        &mut self.data.session.therapist,
                        &mut check_if_session_can_start,
                        true,
                    );
                    ui.end_row();

                    // Not saved to any file.
                    edit_box(
                        ui,
                        "Data Collector",
                        &mut self.data.session.data_collector,
                        &mut check_if_session_can_start,
                        true,
                    );
                    ui.end_row();

                    ui.label("Assessment");
                    let assessment_text = match self.data.assessment_chosen() {
                        true => egui::RichText::new(&self.data.session.chosen_assessment),
                        false => egui::RichText::new("NONE").color(ui.visuals().error_fg_color),
                    };
                    egui::ComboBox::from_id_salt("assessment")
                        .selected_text(assessment_text)
                        .show_ui(ui, |ui| {
                            for (name, assessment) in self.data.assessments.iter() {
                                if ui
                                    .selectable_value(
                                        &mut self.data.session.chosen_assessment,
                                        name.clone(),
                                        name.clone(),
                                    )
                                    .clicked()
                                {
                                    self.data.session.chosen_condition = assessment
                                        .first_condition()
                                        .unwrap_or(&String::new())
                                        .clone();
                                    self.data.current_session = assessment.session;
                                    if self.data.ksfs.contains_key(&assessment.preferred_ksf) {
                                        self.data.session.chosen_ksf_name =
                                            assessment.preferred_ksf.clone();
                                    }
                                    check_if_session_can_start = true;
                                }
                            }
                        });
                    ui.end_row();

                    ui.label("Condition");
                    let condition_text = match self.data.condition_chosen() {
                        true => egui::RichText::new(&self.data.session.chosen_condition),
                        false => egui::RichText::new("NONE").color(ui.visuals().error_fg_color),
                    };
                    egui::ComboBox::from_id_salt("condition")
                        .selected_text(condition_text)
                        .show_ui(ui, |ui| {
                            if let Some(conds) = self
                                .data
                                .assessments
                                .get(self.data.chosen_assessment_name())
                            {
                                for cond in conds.conditions.iter() {
                                    if ui
                                        .selectable_value(
                                            &mut self.data.session.chosen_condition,
                                            cond.to_string(),
                                            cond,
                                        )
                                        .clicked()
                                    {
                                        check_if_session_can_start = true;
                                    }
                                }
                            }
                        });
                    ui.end_row();
                });

            ui.collapsing("Client and Session Data", |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("debug scroller")
                    .max_height(200.0)
                    .min_scrolled_height(200.0)
                    .max_width(500.0)
                    .show(ui, |ui| {
                        ui.monospace(format!("{}", self.data.to_json().unwrap()));
                    });
            });
            ui.add_space(5.0);
            ui.collapsing("Config Data", |ui| {
                if ui.button("Save Config Data").clicked() {
                    quick_error!(overwrite_file(
                        path_to_config_file(),
                        &self.config.to_json().unwrap()
                    ))
                }
                egui::ScrollArea::vertical()
                    .id_salt("config scroller")
                    .max_height(200.0)
                    .min_scrolled_height(200.0)
                    .max_width(500.0)
                    .show(ui, |ui| {
                        ui.monospace(format!("{}", self.config.to_json().unwrap()));
                    });
            });
            ui.add_space(5.0);

            ui.collapsing("Font Examples", |ui| {
                ui.label("Proportional");
                ui.label(EXAMPLE_TEXT);
                ui.add_space(15.0);

                ui.monospace("Monospace");
                ui.monospace(EXAMPLE_TEXT);
            });
            ui.add_space(5.0);

            ui.collapsing("Simulate Data", |ui| {
                ui.add_enabled_ui(self.data.client_loaded(), |ui| {
                    if ui.button("simulate 10 sessions (with reli)").clicked() {
                        for i in self.data.current_session..=(self.data.current_session + 10) {
                            SessionResults::simulate_session_results(
                                self.path_assessment_condition_dir(),
                                &self.data,
                                i,
                                true,
                                1234567890,
                            );
                        }
                        self.data.current_session += 10;
                    }
                    if ui.button("simulate 10 primay sessions").clicked() {
                        for i in self.data.current_session..=(self.data.current_session + 10) {
                            SessionResults::simulate_session_results(
                                self.path_assessment_condition_dir(),
                                &self.data,
                                i,
                                false,
                                1234567890,
                            );
                        }
                        self.data.current_session += 10;
                    }
                })
            });
        });
    }
}
