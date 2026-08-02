use crate::{
    config::{
        ASSESSMENTS_FILE_NAME, CLIENT_DATA_FILE_NAME, Config, DEFAULT_DIRECTORY, DEFAULT_ZOOM,
        HARDCODED_ROOT_DIR, HARDCODED_ZOOM, IOA_DATA_FOLDER_NAME, KSF_FILE_NAME,
        SESSION_DATA_FOLDER_NAME,
    },
    data::{AssessmentsData, ClientData, Data, KsfData},
    display_control::{DisplayControl, Page},
    ioa::IoaPage,
    pages::{
        EditAssessments, EditKsfData, NewClient, PrepareSession, RandomServices, SessionPage,
        Settings, Sidebar, Timers,
    },
    utils::{date_time_string, overwrite_file, windows_error_dialog},
};
use anyhow::{Context, Result};
use chrono::Local;
use egui::{FontDefinitions, RichText, TextBuffer, Visuals};
use egui_file_dialog::FileDialog;
use std::path::{Path, PathBuf};

pub const NO_CLIENT: &'static str = "no client loaded";
pub const NO_KSF: &'static str = "no KSF loaded";
pub const NO_ASSESSMENT: &'static str = "no assessment chosen";
pub const NO_CONDITION: &'static str = "no condition chosen";
pub const INVALID_DATE: &'static str = "Date of Admission is not valid";

pub struct DataPro {
    pub pick_root_directory: FileDialog,
    pub root_directory: PathBuf,

    pub data: Data,
    pub display_info: DisplayControl,

    pub randomness_page: RandomServices,
    pub timers: Timers,

    pub prep_session: PrepareSession,
    pub session_page: SessionPage,

    pub ioa_page: IoaPage,
    pub new_client_page: NewClient,
    pub edit_ksfs: EditKsfData,
    pub edit_assessments: EditAssessments,
}

impl Default for DataPro {
    fn default() -> Self {
        // provided directory should always be valid on Windows and we are not handling any other OS
        let root_directory = DEFAULT_DIRECTORY.get_or_init(|| HARDCODED_ROOT_DIR.into());

        // If the default directory doesn't exist crate it.
        if !root_directory.exists() {
            if let Err(e) =
                std::fs::create_dir(&root_directory).context("cannot create root directory")
            {
                windows_error_dialog(e);
            };
        }

        Self {
            data: Data::default(),

            display_info: DisplayControl {
                active_page: Page::PrepareSession,
                timers_open: false,
                random_open: false,
                sidebar_open: true,
                zoom: *DEFAULT_ZOOM.get_or_init(|| HARDCODED_ZOOM),
            },

            pick_root_directory: FileDialog::default().initial_directory(root_directory.clone()),
            root_directory: root_directory.clone(),

            randomness_page: RandomServices::default(),
            timers: Timers::default(),

            prep_session: PrepareSession::default(),
            session_page: SessionPage::default(),

            ioa_page: IoaPage::default(),
            new_client_page: NewClient::default(),
            edit_ksfs: EditKsfData::default(),
            edit_assessments: EditAssessments::default(),
        }
    }
}

impl DataPro {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let configs = Config::from_current_dir();
        DEFAULT_DIRECTORY
            .set(configs.root_dir)
            .expect("failed to set default directory");
        cc.egui_ctx
            .set_pixels_per_point(*DEFAULT_ZOOM.get_or_init(|| configs.zoom));
        cc.egui_ctx.set_visuals(Visuals::dark());

        // Custom monospace font
        let mut font_defs = FontDefinitions::default();
        font_defs.font_data.insert(
            "AtkinsonMono".into(),
            std::sync::Arc::new(
                // .ttf and .otf supported
                egui::FontData::from_static(include_bytes!(
                    "..\\AtkinsonHyperlegibleMono-VariableFont_wght.ttf"
                )),
            ),
        );
        // font_defs.font_data.insert(
        //     "AtkinsonNext".into(),
        //     std::sync::Arc::new(
        //         // .ttf and .otf supported
        //         egui::FontData::from_static(include_bytes!(
        //             "..\\AtkinsonHyperlegibleNext-VariableFont_wght.ttf"
        //         )),
        //     ),
        // );
        font_defs
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "AtkinsonMono".to_owned());

        // font_defs
        //     .families
        //     .get_mut(&egui::FontFamily::Proportional)
        //     .unwrap()
        //     .insert(0, "AtkinsonNext".to_owned());
        cc.egui_ctx.set_fonts(font_defs);
        Default::default()
    }

    pub fn client_picker(&mut self, ui: &mut egui::Ui) {
        let client_picker_text = match self.data.client_loaded() {
            true => format!("{:>11}", self.data.client.id),
            false => String::from("Pick Client"),
        };

        egui::ComboBox::from_id_salt("client picker")
            .selected_text(
                RichText::new(client_picker_text)
                    .size(ui.text_style_height(&egui::TextStyle::Heading))
                    .monospace()
                    .strong(),
            )
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(&mut self.data.client.id, String::new(), "None")
                    .clicked()
                {
                    self.unload_client();
                }
                if let Ok(entries) = self.root_directory.read_dir() {
                    for entry in entries {
                        if let Ok(e) = entry {
                            if ui
                                .selectable_value(
                                    &mut self.data.client.id,
                                    e.file_name().to_string_lossy().to_string(),
                                    e.file_name().to_string_lossy().to_string(),
                                )
                                .clicked()
                            {
                                self.unload_client();
                                self.load_client(&e.path());
                            }
                        }
                    }
                }
            });
    }

    pub fn ready_to_start_session(&mut self) -> bool {
        if !self.data.client_loaded() {
            self.prep_session.session_start_error = NO_CLIENT;
            false
        } else if !self.data.client_admission_valid() {
            self.prep_session.session_start_error = INVALID_DATE;
            false
        } else if !self.data.ksf_loaded() {
            self.prep_session.session_start_error = NO_KSF;
            false
        } else if !self.data.assessment_chosen() {
            self.prep_session.session_start_error = NO_ASSESSMENT;
            false
        } else if !self.data.condition_chosen() {
            self.prep_session.session_start_error = NO_CONDITION;
            false
        } else if !self.time_limit_set() {
            self.prep_session.session_start_error = "time limit cannot be 0.0 seconds";
            false
        } else {
            self.prep_session.session_start_error.clear();
            true
        }
    }

    pub fn time_limit_set(&self) -> bool {
        // It is false that: session length is limited and the maximum session length is zero
        !(self.session_page.limit_session_length && self.session_page.maximum_session_length == 0.0)
    }

    /// Search inside the top of the active client folder for a file or folder name
    pub fn path_to(&self, name: &str) -> Result<PathBuf> {
        if !self.data.client_loaded() {
            return Err(anyhow::anyhow!(
                "cannot find {} because {}",
                name,
                NO_CLIENT
            ));
        } else {
            Ok(Path::new(&self.root_directory)
                .join(&self.data.client.id.to_string())
                .join(name))
        }
    }

    /// Path to client_data.txt if a client has been chosen.
    pub fn path_to_client_data(&self) -> Result<PathBuf> {
        self.path_to(CLIENT_DATA_FILE_NAME)
    }

    /// Path to assessments.txt if a client has been chosen.
    pub fn path_to_assessments(&self) -> Result<PathBuf> {
        self.path_to(ASSESSMENTS_FILE_NAME)
    }

    /// Path to ksf_data.txt if a client has been chose.
    pub fn path_to_ksf_data(&self) -> Result<PathBuf> {
        self.path_to(KSF_FILE_NAME)
    }

    /// Path to Session Records if a client has been chose.
    pub fn path_to_sessions_data(&self) -> Result<PathBuf> {
        self.path_to(SESSION_DATA_FOLDER_NAME)
    }

    /// Path to IOA Data if a client has been chose.
    pub fn path_to_ioa_data(&self) -> Result<PathBuf> {
        self.path_to(IOA_DATA_FOLDER_NAME)
    }

    pub fn overwrite_file(&self, name: &str, data: &str) -> Result<()> {
        overwrite_file(self.path_to(name), data)
    }

    pub fn overwrite_client_data(&self) -> Result<()> {
        self.overwrite_file(CLIENT_DATA_FILE_NAME, &self.data.client.to_json()?)
    }

    pub fn overwrite_assessments(&self) -> Result<()> {
        self.overwrite_file(ASSESSMENTS_FILE_NAME, &self.data.assessments.to_json()?)
    }

    pub fn overwrite_ksf_data(&self) -> Result<()> {
        self.overwrite_file(KSF_FILE_NAME, &self.data.ksfs.to_json()?)
    }

    pub fn load_ksf(&mut self, path: &PathBuf) {
        match KsfData::from_file(&path) {
            Ok(ksf) => {
                self.data.ksfs = ksf;
            }
            Err(e) => {
                self.data.ksfs = KsfData::default();
                windows_error_dialog(e);
            }
        };
    }

    /// Attempt to load the first assessment and its first condition
    pub fn choose_first_assessment_and_condition(&mut self) {
        match self.data.assessments.first() {
            Some((assessment, conds)) => {
                self.data.session.chosen_assessment = assessment.clone();
                match conds.first() {
                    Some(cond) => self.data.session.chosen_condition = cond.clone(),
                    None => self.data.session.chosen_condition.clear(),
                }
            }
            None => {
                self.data.session.chosen_assessment.clear();
                self.data.session.chosen_condition.clear()
            }
        }
    }

    pub fn unload_client(&mut self) {
        self.data.clear();
        self.edit_assessments.prepare(
            &self.data,
            DEFAULT_DIRECTORY
                .get_or_init(|| HARDCODED_ROOT_DIR.into())
                .clone(),
        );
        self.edit_ksfs.prepare(
            &self.data,
            DEFAULT_DIRECTORY
                .get_or_init(|| HARDCODED_ROOT_DIR.into())
                .clone(),
        );
        self.ioa_page.reset();
    }

    pub fn load_client(&mut self, path: &PathBuf) {
        // Determine if the client file exists
        match ClientData::from_file(&Path::new(path).join(CLIENT_DATA_FILE_NAME))
            .context("error reading client_data.txt")
        {
            Ok(client) => {
                // Clear all data
                self.data.clear();

                // Load the client
                // We are always one session ahead of the last saved value
                self.data.client = client;
                self.data.client.current_session += 1;

                // Load the KSF Data
                let ksf_path = Path::new(path).join(KSF_FILE_NAME);
                match KsfData::from_file(&ksf_path) {
                    Ok(ksf_data) => {
                        self.data.ksfs = ksf_data;
                        self.edit_ksfs.prepare(&self.data, ksf_path.clone());
                        self.edit_ksfs.save_new_path = ksf_path.clone();
                        self.edit_ksfs.file_dialog = FileDialog::new().initial_directory(ksf_path)
                    }
                    Err(e) => {
                        windows_error_dialog(e.context(format!("unable to read {}", KSF_FILE_NAME)))
                    }
                };
                if let Some((name, _)) = self.data.ksfs.first() {
                    self.data.session.chosen_ksf = name.clone()
                }

                // Load the Assessments Data
                let assessments_path = Path::new(path).join(ASSESSMENTS_FILE_NAME);
                match AssessmentsData::from_file(&assessments_path) {
                    Ok(assessments_data) => {
                        self.data.assessments = assessments_data;
                        self.edit_assessments
                            .prepare(&self.data, assessments_path.clone());
                        self.edit_assessments.save_new_path = assessments_path.clone();
                        self.edit_assessments.file_dialog =
                            FileDialog::new().initial_directory(assessments_path)
                    }
                    Err(e) => windows_error_dialog(
                        e.context(format!("unable to read {}", ASSESSMENTS_FILE_NAME)),
                    ),
                }
                self.choose_first_assessment_and_condition();

                // Update the IOA page
                self.ioa_page.reset();
                self.ioa_page.file_dialog =
                    FileDialog::new().initial_directory(path.join(SESSION_DATA_FOLDER_NAME));
            }
            Err(e) => {
                self.unload_client();
                windows_error_dialog(e);
            }
        };
    }
}

impl eframe::App for DataPro {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // ### Windows ###
        Timers::view(self, ui);
        RandomServices::view(self, ui);

        // ### Top Bar ###
        // To go fully across it must be specified before any other panel
        // Nothing here can be interactable because we use Tab and Space as controls on the Session Page
        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.request_repaint_after_secs(5.0);
                ui.label(format!("{}", date_time_string(&Local::now())));
            });
        });

        // ### Sidebar ###
        // To show it must go before any other panel
        // It must be not to rendered (even if not shown) when Session is active because it may capture keypresses
        if self.display_info.sidebar_open {
            Sidebar::view(self, ui);
        };

        // ### Main Panel ###
        match self.display_info.active_page {
            Page::RunSession => SessionPage::view(self, ui),
            Page::Ioa => IoaPage::view(self, ui),
            Page::PrepareSession => PrepareSession::view(self, ui),
            Page::CreateClient => NewClient::view(self, ui),
            Page::CreateKsf => EditKsfData::view(self, ui),
            Page::CreateAssessments => EditAssessments::view(self, ui),
            Page::Settings => Settings::view(self, ui),
        }
    }
}
