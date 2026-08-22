use crate::{
    app::DataPro,
    data::{Assessment, DataType},
    quick_error,
    ui_elements::DataProUiElements,
    utils::windows_error_dialog,
};
use egui::RichText;

pub struct PrepareSession {
    pub can_start_session: bool,
    pub session_start_error: &'static str,
    pub edit_primary_therapist: bool,
    pub edit_case_manager: bool,
    pub edit_client_id: bool,
    pub edit_doa: bool,
    pub limit_session_length: bool,
    pub maximum_session_length: f32,
}

impl Default for PrepareSession {
    fn default() -> Self {
        Self {
            can_start_session: false,
            session_start_error: "configuration error, this should not be visible",
            edit_primary_therapist: false,
            edit_case_manager: false,
            edit_client_id: false,
            edit_doa: false,
            limit_session_length: false,
            maximum_session_length: 0.0,
        }
    }
}

impl PrepareSession {
    fn client_and_session_information(app: &mut DataPro, ui: &mut egui::Ui) {
        ui.add_enabled_ui(app.data.client_loaded(), |ui| {
            egui::Grid::new("client_and_session_info_grid")
                .min_col_width(120.0)
                .min_row_height(22.0)
                .show(ui, |ui| {
                    ui.label("Location");
                    if ui
                        .text_edit_singleline(&mut app.data.client.location)
                        .lost_focus()
                    {
                        quick_error!(app.overwrite_client_data());
                    }
                    ui.end_row();

                    ui.label("Date of Admission");
                    if app.prep_session.edit_doa {
                        if ui
                            .add(egui::TextEdit::singleline(
                                &mut app.data.client.date_of_admission,
                            ))
                            .on_hover_text("format date as YYYY-MM-DD")
                            .lost_focus()
                        {
                            quick_error!(app.overwrite_client_data());
                        }
                    } else {
                        match app.data.client.days_since_admission() {
                            Ok(n) => {
                                // emphasize negative DOA with red text
                                if n.is_negative() {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut format!("{n} days ago"))
                                            .text_color(ui.visuals().error_fg_color)
                                            .interactive(false),
                                    )
                                    .on_hover_text(&app.data.client.date_of_admission);
                                    app.prep_session.can_start_session = false;
                                } else {
                                    // normal DOA information
                                    ui.add(
                                        egui::TextEdit::singleline(&mut format!("{n} days ago"))
                                            .interactive(false),
                                    )
                                    .on_hover_text(&app.data.client.date_of_admission);
                                }
                            }
                            Err(_e) => {
                                // indicate invalid date with ERROR, red text, and hover text explanation
                                ui.add(
                                    egui::TextEdit::singleline(&mut format!("ERROR"))
                                        .text_color(ui.visuals().error_fg_color)
                                        .interactive(false),
                                )
                                .on_hover_text(&app.data.client.date_of_admission);
                                app.prep_session.can_start_session = false;
                            }
                        }
                    }
                    ui.lock_unlock_button(&mut app.prep_session.edit_doa);
                    ui.end_row();

                    ui.label("Session Number");
                    if ui
                        .add(egui::DragValue::new(&mut app.data.current_session))
                        .lost_focus()
                    {
                        let current_session = app.data.current_session;
                        if let Some(condtions) = app.data.active_assessment_data() {
                            condtions.session = current_session;
                        }
                        quick_error!(app.overwrite_assessments());
                    }
                    ui.end_row();

                    ui.label("Case Manager");
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut app.data.client.case_manager)
                                .interactive(app.prep_session.edit_case_manager),
                        )
                        .lost_focus()
                    {
                        quick_error!(app.overwrite_client_data());
                    }
                    ui.lock_unlock_button(&mut app.prep_session.edit_case_manager);
                    ui.end_row();

                    ui.label("Primary Therapist");
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut app.data.client.primary_therapist)
                                .interactive(app.prep_session.edit_primary_therapist),
                        )
                        .lost_focus()
                    {
                        quick_error!(app.overwrite_client_data());
                    }
                    ui.lock_unlock_button(&mut app.prep_session.edit_primary_therapist);

                    ui.end_row();

                    ui.label("Session Therapist");
                    ui.text_edit_singleline(&mut app.data.session.therapist);
                    ui.end_row();

                    ui.label("Data Collector");
                    ui.text_edit_singleline(&mut app.data.session.data_collector);
                    ui.end_row();

                    ui.label("Primary/Reliability");
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

                    ui.label("Assessment");
                    let assessment_text = match app.data.assessment_chosen() {
                        true => egui::RichText::new(&app.data.session.chosen_assessment),
                        false => egui::RichText::new("NONE").color(ui.visuals().error_fg_color),
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
                            .or_insert(Assessment::default());
                    }
                    egui::ComboBox::from_id_salt("assessment")
                        .selected_text(assessment_text)
                        .show_ui(ui, |ui| {
                            for (name, assessment) in app.data.assessments.iter() {
                                if ui
                                    .selectable_value(
                                        &mut app.data.session.chosen_assessment,
                                        name.clone(),
                                        name.clone(),
                                    )
                                    .clicked()
                                {
                                    app.data.session.chosen_condition =
                                        assessment.first().unwrap_or(&String::new()).clone();
                                    app.data.current_session = assessment.session;
                                    if app.data.ksfs.contains_key(&assessment.preferred_ksf) {
                                        app.data.session.chosen_ksf =
                                            assessment.preferred_ksf.clone();
                                    }
                                }
                            }
                        });
                    ui.end_row();

                    ui.label("Condition");
                    let condition_text = match app.data.condition_chosen() {
                        true => egui::RichText::new(&app.data.session.chosen_condition),
                        false => egui::RichText::new("NONE").color(ui.visuals().error_fg_color),
                    };
                    let condition_box =
                        ui.text_edit_singleline(&mut app.data.session.chosen_condition);
                    if condition_box.lost_focus() {
                        if let Some(conds) = app
                            .data
                            .assessments
                            .get_mut(&app.data.session.chosen_assessment)
                        {
                            conds
                                .conditions
                                .insert(app.data.session.chosen_condition.clone());
                        }
                    }
                    egui::ComboBox::from_id_salt("condition")
                        .selected_text(condition_text)
                        .show_ui(ui, |ui| {
                            if let Some(conds) =
                                app.data.assessments.get(app.data.active_assessment_name())
                            {
                                for cond in conds.conditions.iter() {
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
        ui.spacing_mut().item_spacing = (0.0, 0.0).into();

        let ksf_text = match app.data.ksf_loaded() {
            true => egui::RichText::new(app.data.chosen_ksf().clone()),
            false => egui::RichText::new("NONE").color(ui.visuals().error_fg_color),
        };

        egui::ComboBox::from_id_salt("ksfs_dropdown")
            .selected_text(ksf_text)
            .show_ui(ui, |ui| {
                for name in app.data.ksfs.iter().map(|(name, _ksf)| name).cloned() {
                    ui.selectable_value(&mut app.data.session.chosen_ksf, name.clone(), name);
                }
            });
        ui.add_space(10.0);
        if app.data.ksf_loaded() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if let Some(ksf) = app.data.ksfs.get(app.data.chosen_ksf()) {
                        let (freq, dura) = ksf.pairs();
                        ui.vertical(|ui| {
                            ui.strong("Frequency Keys");
                            ui.add_space(2.0);
                            for (key, desc) in freq {
                                ui.add(egui::Label::new(
                                    RichText::from(format!("{:>2} {}", key.symbol_or_name(), desc))
                                        .monospace()
                                        .size(12.0),
                                ));
                            }
                        });
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            ui.strong("Duration Keys");
                            ui.add_space(2.0);
                            for (key, desc) in dura {
                                ui.add(egui::Label::new(
                                    RichText::from(format!("{:>2} {}", key.symbol_or_name(), desc))
                                        .monospace()
                                        .size(12.0),
                                ));
                            }
                        });
                    } else {
                        ui.monospace(
                            egui::RichText::new("ERROR INVALID KSF NAME")
                                .color(ui.visuals().error_fg_color),
                        );
                    }
                });
            });
        }
    }

    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        // TODO: This probably doesn't need to run every frame
        app.prep_session.can_start_session = app.ready_to_start_session();

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add_space(15.0);
                    app.client_picker(ui);
                    ui.add_space(5.0);

                    PrepareSession::client_and_session_information(app, ui);
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.add_enabled(
                            app.prep_session.limit_session_length,
                            egui::DragValue::new(&mut app.prep_session.maximum_session_length)
                                .suffix("  secs")
                                .range(0.0..=999_999.0),
                        );

                        ui.checkbox(
                            &mut app.prep_session.limit_session_length,
                            "Limit Session Length",
                        );
                    });
                    ui.add_space(5.0);

                    ui.add_enabled_ui(app.prep_session.can_start_session, |ui| {
                        if ui
                            .large_green_button("BEGIN SESSION")
                            .on_disabled_hover_text(app.prep_session.session_start_error)
                            .clicked()
                        {
                            if let Err(e) = app.overwrite_client_data() {
                                windows_error_dialog(e)
                            } else {
                                // Update the client file with any changes
                                // This is only relevant if the user changes a client field and then immediately clicks BEGIN SESSION
                                // If they do anything else the file will update when they switch selections
                                // Load the data and switch pages.
                                if let Some(conditions) =
                                    app.data.assessments.get(app.data.active_assessment_name())
                                {
                                    app.data.current_session = conditions.session;
                                }
                                app.session.load_ksf(&app.data);
                                app.timers.stop_all_timers();
                                app.display_info.go_to_run_session();
                            }
                        }
                    })
                });
                ui.add_space(50.0);
                ui.vertical(|ui| {
                    ui.add_space(15.0);
                    ui.add_enabled_ui(app.data.client_loaded(), |ui| {
                        ui.heading("KSF");
                        ui.add_space(5.0);
                        PrepareSession::ksf_display(app, ui);
                    });
                });
            });

            ui.add_space(10.0);
        });
    }
}
