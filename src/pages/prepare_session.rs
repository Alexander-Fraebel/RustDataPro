use crate::{
    app::{DataPro, NO_CLIENT, SESSION_DATA_FOLDER_NAME},
    data::DataType,
    utils::{DataProUiElements, windows_error_dialog},
};
use egui::{Color32, TextStyle};
use egui_file_dialog::FileDialog;
use indexmap::IndexSet;
use std::path::Path;

pub struct PrepareSession {
    pub can_start_session: bool,
    pub session_start_error: &'static str,
}

impl Default for PrepareSession {
    fn default() -> Self {
        Self {
            can_start_session: true,
            session_start_error: NO_CLIENT,
        }
    }
}

impl PrepareSession {
    fn client_and_session_information(app: &mut DataPro, ui: &mut egui::Ui) {
        ui.add_enabled_ui(app.data.client_loaded(), |ui| {
            egui::Grid::new("client_and_session_info_grid")
                .min_col_width(150.0)
                .min_row_height(22.0)
                .show(ui, |ui| {
                    // For reasons of client privacy this is probably best not to display
                    // ui.monospace("Client Name");
                    // ui.monospace(&app.data.client.name);
                    // ui.end_row();

                    ui.monospace("Client ID");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.data.client.id.to_string())
                            .font(TextStyle::Monospace)
                            .interactive(false),
                    );
                    ui.end_row();

                    ui.monospace("Location");
                    if ui
                        .text_edit_singleline(&mut app.data.client.location)
                        .lost_focus()
                    {
                        if let Err(e) = app.overwrite_client_data_file() {
                            windows_error_dialog(e)
                        }
                    }
                    ui.end_row();

                    ui.monospace("Date of Admission");
                    match app.data.client.days_since_admission() {
                        Ok(n) => {
                            // emphasize negative DOA with red text
                            if n.is_negative() {
                                ui.add(
                                    egui::TextEdit::singleline(&mut format!("{n} days ago"))
                                        .font(TextStyle::Monospace)
                                        .text_color(Color32::RED)
                                        .interactive(false),
                                )
                                .on_hover_text(&app.data.client.date_of_admission);
                                app.prep_session.can_start_session = false;
                            } else {
                                // normal DOA information
                                ui.add(
                                    egui::TextEdit::singleline(&mut format!("{n} days ago"))
                                        .font(TextStyle::Monospace)
                                        .interactive(false),
                                )
                                .on_hover_text(&app.data.client.date_of_admission);
                            }
                        }
                        Err(_e) => {
                            // indicate invalid date with ERROR, red text, and hover text explanation
                            ui.add(
                                egui::TextEdit::singleline(&mut format!("ERROR"))
                                    .font(TextStyle::Monospace)
                                    .text_color(Color32::RED)
                                    .interactive(false),
                            )
                            .on_hover_text(format!(
                                "{} is an invalid date\nformat date as YYYY-MM-DD",
                                app.data.client.date_of_admission
                            ));
                            app.prep_session.can_start_session = false;
                        }
                    }
                    ui.end_row();

                    ui.monospace("Session Number");
                    if ui
                        .add(egui::DragValue::new(&mut app.data.client.current_session))
                        .lost_focus()
                    {
                        if let Err(e) = app.overwrite_client_data_file() {
                            windows_error_dialog(e)
                        }
                    }
                    ui.end_row();

                    ui.monospace("Case Manager");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.data.client.case_manager)
                            .font(TextStyle::Monospace)
                            .interactive(false),
                    );
                    ui.end_row();

                    ui.monospace("Primary Therapist");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.data.client.primary_therapist)
                            .font(TextStyle::Monospace)
                            .interactive(false),
                    );
                    ui.end_row();

                    ui.monospace("Session Therapist");
                    ui.text_edit_singleline(&mut app.data.session.therapist);
                    ui.end_row();

                    ui.monospace("Data Collector");
                    ui.text_edit_singleline(&mut app.data.session.data_collector);
                    ui.end_row();

                    ui.monospace("Primary/Reli Data");
                    egui::ComboBox::from_id_salt("datatype")
                        .selected_text(app.data.session.data_type.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.data.session.data_type,
                                DataType::Primary,
                                "Primary",
                            );
                            ui.selectable_value(
                                &mut app.data.session.data_type,
                                DataType::Reliability,
                                "Reliability",
                            );
                        });
                    ui.end_row();

                    ui.monospace("Assessment");
                    let assessment_text = match app.data.assessment_chosen() {
                        true => egui::RichText::new(&app.data.session.chosen_assessment),
                        false => egui::RichText::new("NONE").color(Color32::RED),
                    };
                    let assessment_box =
                        ui.text_edit_singleline(&mut app.data.session.chosen_assessment);
                    if assessment_box.changed() {
                        app.data.session.chosen_condition.clear();
                    };
                    if assessment_box.lost_focus() {
                        app.data
                            .assessments
                            .entry(app.data.session.chosen_assessment.clone())
                            .or_insert(IndexSet::new());
                    }
                    egui::ComboBox::from_id_salt("assessment")
                        .selected_text(assessment_text)
                        .show_ui(ui, |ui| {
                            for (assessment, conditions) in app.data.assessments.iter() {
                                if ui
                                    .selectable_value(
                                        &mut app.data.session.chosen_assessment,
                                        assessment.clone(),
                                        assessment.clone(),
                                    )
                                    .clicked()
                                {
                                    app.data.session.chosen_condition =
                                        conditions.first().unwrap_or(&String::new()).clone();
                                }
                            }
                        });

                    ui.end_row();

                    ui.monospace("Condition");
                    let condition_text = match app.data.condition_chosen() {
                        true => egui::RichText::new(&app.data.session.chosen_condition),
                        false => egui::RichText::new("NONE").color(Color32::RED),
                    };
                    let condition_box =
                        ui.text_edit_singleline(&mut app.data.session.chosen_condition);
                    if condition_box.lost_focus() {
                        if let Some(conds) = app
                            .data
                            .assessments
                            .get_mut(&app.data.session.chosen_assessment)
                        {
                            conds.insert(app.data.session.chosen_condition.clone());
                        }
                    }
                    egui::ComboBox::from_id_salt("condition")
                        .selected_text(condition_text)
                        .show_ui(ui, |ui| {
                            if let Some(conds) =
                                app.data.assessments.get(app.data.chosen_assessment())
                            {
                                for cond in conds {
                                    ui.selectable_value(
                                        &mut app.data.session.chosen_condition,
                                        cond.to_string(),
                                        cond,
                                    );
                                }
                            }
                        });

                    ui.end_row();
                });
            ui.add_space(10.0);
        });
    }

    fn ksf_display(app: &mut DataPro, ui: &mut egui::Ui) {
        let ksf_text = match app.data.ksf_loaded() {
            true => egui::RichText::new(app.data.session.chosen_ksf.clone()),
            false => egui::RichText::new("NONE").color(Color32::RED),
        };

        egui::ComboBox::from_id_salt("ksfs_dropdown")
            .selected_text(ksf_text)
            .show_ui(ui, |ui| {
                for name in app.data.ksfs.iter().map(|(name, _ksf)| name).cloned() {
                    ui.selectable_value(&mut app.data.session.chosen_ksf, name.clone(), name);
                }
            });
        if app.data.ksf_loaded() {
            ui.group(|ui| {
                if let Some(ksf) = app.data.ksfs.get(&app.data.session.chosen_ksf) {
                    let (freq, dura) = ksf.pairs();
                    ui.add_space(10.0);
                    ui.strong("Frequency Keys");
                    for (key, desc) in freq {
                        ui.monospace(format!("{:>2} {}", key.symbol_or_name(), desc));
                    }
                    ui.add_space(10.0);
                    ui.strong("Duration Keys");
                    for (key, desc) in dura {
                        ui.monospace(format!("{:>2} {}", key.symbol_or_name(), desc));
                    }
                } else {
                    ui.monospace(egui::RichText::new("NO KSF").color(Color32::RED));
                }
            });
        }
    }

    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        app.pick_ksf.update(ui.ctx());
        if let Some(path) = app.pick_ksf.take_picked() {
            app.load_ksf(&path);
        }

        app.pick_client_folder.update(ui.ctx());
        if let Some(path) = app.pick_client_folder.take_picked() {
            app.load_client_file(&path.clone());
            app.pick_ksf = FileDialog::new().initial_directory(path.clone());
            app.ioa_page.file_dialog = FileDialog::new()
                .initial_directory(Path::new(&path).join(SESSION_DATA_FOLDER_NAME));
        }

        app.prep_session.can_start_session = app.ready_to_start_session();

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add_space(25.0);
                    if ui.large_blue_button("Select Client").clicked() {
                        app.pick_client_folder.pick_directory();
                    }
                    ui.add_space(5.0);
                    PrepareSession::client_and_session_information(app, ui);
                });
                ui.add_space(50.0);
                ui.vertical(|ui| {
                    ui.add_space(25.0);
                    ui.add_enabled_ui(app.data.client_loaded(), |ui| {
                        if ui
                            .large_blue_button("Select KSF")
                            .on_disabled_hover_text(NO_CLIENT)
                            .clicked()
                        {
                            app.pick_ksf.pick_file();
                        }

                        ui.add_space(5.0);
                        PrepareSession::ksf_display(app, ui);
                    });
                });
            });

            ui.horizontal(|ui| {
                ui.add_enabled(
                    app.session_page.limit_session_length,
                    egui::DragValue::new(&mut app.session_page.maximum_session_length)
                        .suffix("  secs")
                        .range(0.0..=100_000.0),
                );
                ui.checkbox(
                    &mut app.session_page.limit_session_length,
                    "Limit Session Length",
                );
            });
            ui.add_space(10.0);

            ui.add_enabled_ui(app.prep_session.can_start_session, |ui| {
                if ui
                    .large_green_button("BEGIN SESSION")
                    .on_disabled_hover_text(app.prep_session.session_start_error)
                    .clicked()
                {
                    // Update the client file with any changes
                    // This is only relevant if the user changes a client field and then immediately clicks BEGIN SESSION
                    // If they do anything else the file will update when they switch pages
                    if let Err(e) = app.overwrite_client_data_file() {
                        windows_error_dialog(e)
                    } else {
                        // Load the data and switch pages.
                        app.session_page.load_ksf(&app.data);
                        app.timers.pause_all_timers();
                        app.display_info.go_to_run_session();
                    }
                }
            })
        });
    }
}
