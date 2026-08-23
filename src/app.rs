use crate::{
    config::{
        ASSESSMENTS_FILE_NAME, CLIENT_DATA_FILE_NAME, Config, DEFAULT_DIRECTORY, DEFAULT_ZOOM,
        HARDCODED_ROOT_DIR, HARDCODED_ZOOM, IOA_DATA_FOLDER_NAME, KSF_FILE_NAME,
        SESSION_DATA_FOLDER_NAME,
    },
    data::{AssessmentsData, ClientData, Data, KsfsData, NO_CLIENT},
    display_control::{DisplayControl, Page},
    ioa::{IoaPage, validate_files::validate_files},
    pages::{
        EditAssessments, EditKsfData, NewClient, PrepareSession, SessionPage, Settings, Shuffler,
        Sidebar, Timers, credits::Credits, debug_page::DebugPage,
    },
    preference_assessment::PreferenceAssessment,
    quick_error,
    utils::{date_time_string, overwrite_file, windows_error_dialog},
};
use anyhow::{Context, Result};
use chrono::Local;
use egui::{FontDefinitions, RichText, Visuals};
use egui_file_dialog::FileDialog;
use rand::{make_rng, rngs::StdRng};
use std::path::{Path, PathBuf};

pub struct DataPro {
    pub pick_root_directory: FileDialog,
    pub root_directory: PathBuf,

    pub rng: StdRng, // StdRng is currently ChaCha12 initalized from SysRng, any similar rng is more than sufficient

    pub data: Data,
    pub display_info: DisplayControl,

    pub randomness_page: Shuffler,
    pub timers: Timers,

    pub prep_session: PrepareSession,
    pub session: SessionPage,

    pub ioa_page: IoaPage,
    pub new_client_page: NewClient,
    pub edit_ksfs: EditKsfData,
    pub edit_assessments: EditAssessments,
    pub settings: Settings,
    pub preference_assessment: PreferenceAssessment,
}

impl Default for DataPro {
    fn default() -> Self {
        // provided directory should always be valid on Windows and we are not handling any other OS
        let root_dir = DEFAULT_DIRECTORY
            .get_or_init(|| HARDCODED_ROOT_DIR.into())
            .clone();

        let config = Config {
            zoom: *DEFAULT_ZOOM.get_or_init(|| HARDCODED_ZOOM),
            root_dir: root_dir.clone(),
        };

        // If the default directory doesn't exist create it.
        if !root_dir.exists() {
            quick_error!(std::fs::create_dir(&root_dir).context("cannot create root directory"));
        }

        let mut app = Self {
            data: Data::default(),

            rng: make_rng(),

            display_info: DisplayControl {
                active_page: Page::PrepareSession,
                timers_open: false,
                random_open: false,
                sidebar_open: true,
                debug_open: false,
            },

            pick_root_directory: FileDialog::default().initial_directory(root_dir.clone()),
            root_directory: root_dir.clone(),

            randomness_page: Shuffler::default(),
            timers: Timers::default(),

            prep_session: PrepareSession::default(),
            session: SessionPage::default(),

            ioa_page: IoaPage::default(),
            new_client_page: NewClient::default(),
            edit_ksfs: EditKsfData::default(),
            edit_assessments: EditAssessments::default(),
            settings: Settings {
                config,
                default_root_dir_string: root_dir.clone().to_string_lossy().to_string(),
            },
            preference_assessment: PreferenceAssessment::default(),
        };

        // Initialize pages by unloading
        app.unload_client();
        app
    }
}

impl DataPro {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load the config file information before the application loads
        let configs = Config::from_current_dir();
        DEFAULT_DIRECTORY.get_or_init(|| configs.root_dir);
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
        font_defs.font_data.insert(
            "AtkinsonNext".into(),
            std::sync::Arc::new(
                // .ttf and .otf supported
                egui::FontData::from_static(include_bytes!(
                    "..\\AtkinsonHyperlegibleNext-VariableFont_wght.ttf"
                )),
            ),
        );
        font_defs
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "AtkinsonMono".to_owned());

        font_defs
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "AtkinsonNext".to_owned());
        cc.egui_ctx.set_fonts(font_defs);
        Default::default()
    }

    pub fn client_picker(&mut self, ui: &mut egui::Ui) {
        let client_picker_text = match self.data.client_loaded() {
            true => &self.data.client.id,
            false => "Select Client",
        };

        egui::ComboBox::from_id_salt("client picker")
            .width(200.0)
            .selected_text(
                RichText::new(client_picker_text)
                    .size(ui.text_style_height(&egui::TextStyle::Heading))
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

    pub fn ksf_picker(&mut self, ui: &mut egui::Ui) {
        let ksf_picker_text = match self.data.ksf_loaded() {
            true => egui::RichText::new(self.data.chosen_ksf_name().clone()).strong(),
            false => egui::RichText::new("NONE").color(ui.visuals().error_fg_color),
        };

        egui::ComboBox::from_id_salt("ksf picker")
            .width(200.0)
            .selected_text(ksf_picker_text)
            .show_ui(ui, |ui| {
                for name in self.data.ksfs.keys() {
                    ui.selectable_value(&mut self.data.session.chosen_ksf_name, name.clone(), name);
                }
            });
    }

    pub fn check_if_ready_to_start_session(&mut self) {
        self.data.update_misconfigurations();
        self.prep_session.can_start_session = self.data.misconfigs.is_empty();
    }

    /// Create the path to a file or folder that is inside the top of the active client folder. Returns an error if no client is loaded.
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

    /// Path to client_data.txt if a client has been chosen or the default directory otherwise.
    pub fn path_to_client_data(&self) -> PathBuf {
        self.path_to(CLIENT_DATA_FILE_NAME).unwrap_or_else(|_| {
            DEFAULT_DIRECTORY
                .get_or_init(|| HARDCODED_ROOT_DIR.into())
                .clone()
        })
    }

    /// Path to assessments.txt if a client has been chosen or the default directory otherwise.
    pub fn path_to_assessments(&self) -> PathBuf {
        self.path_to(ASSESSMENTS_FILE_NAME).unwrap_or_else(|_| {
            DEFAULT_DIRECTORY
                .get_or_init(|| HARDCODED_ROOT_DIR.into())
                .clone()
        })
    }

    /// Path to ksf_data.txt if a client has been chosen or the default directory otherwise.
    pub fn path_to_ksf_data(&self) -> PathBuf {
        self.path_to(KSF_FILE_NAME).unwrap_or_else(|_| {
            DEFAULT_DIRECTORY
                .get_or_init(|| HARDCODED_ROOT_DIR.into())
                .clone()
        })
    }

    /// Path to Session Records if a client has been chosen or the default directory otherwise.
    pub fn path_to_session_records_dir(&self) -> PathBuf {
        self.path_to(SESSION_DATA_FOLDER_NAME).unwrap_or_else(|_| {
            DEFAULT_DIRECTORY
                .get_or_init(|| HARDCODED_ROOT_DIR.into())
                .clone()
        })
    }

    /// Path to IOA Data if a client has been chosen or the default directory otherwise.
    pub fn path_to_ioa_data_dir(&self) -> PathBuf {
        self.path_to(IOA_DATA_FOLDER_NAME).unwrap_or_else(|_| {
            DEFAULT_DIRECTORY
                .get_or_init(|| HARDCODED_ROOT_DIR.into())
                .clone()
        })
    }

    pub fn save_new_ioa_data(&mut self) -> Result<()> {
        let path = self.path_to_ioa_data_dir();
        let ioa_page = &mut self.ioa_page;
        if !ioa_page.ioa_finished {
            validate_files(&ioa_page.prim_data, &ioa_page.reli_data)?;
            ioa_page.calculate_ioa(&path)?;
            ioa_page.ioa_finished = true;
            Ok(())
        } else {
            Err(anyhow::anyhow!("IoaData already saved"))
        }
    }

    pub fn overwrite_client_data(&self) -> Result<()> {
        overwrite_file(Ok(self.path_to_client_data()), &self.data.client.to_json()?)
    }

    pub fn overwrite_assessments(&self) -> Result<()> {
        overwrite_file(
            Ok(self.path_to_assessments()),
            &self.data.assessments.to_json()?,
        )
    }

    pub fn overwrite_ksf_data(&self) -> Result<()> {
        overwrite_file(Ok(self.path_to_ksf_data()), &self.data.ksfs.to_json()?)
    }

    pub fn load_ksf(&mut self, path: &PathBuf) {
        match KsfsData::from_file(&path) {
            Ok(ksf) => {
                self.data.ksfs = ksf;
            }
            Err(e) => {
                self.data.ksfs = KsfsData::default();
                windows_error_dialog(e);
            }
        };
    }

    pub fn load_assessments(&mut self) -> Result<()> {
        let assessments_path = self.path_to_assessments();
        match AssessmentsData::from_file(&assessments_path) {
            Ok(assessments_data) => {
                self.data.assessments = assessments_data;
                self.edit_assessments
                    .prepare(&self.data, assessments_path.clone());
                self.choose_first_assessment_and_condition();
            }
            Err(e) => {
                windows_error_dialog(e.context(format!(
                    "unable to read {}, the file may be missing or corrupt",
                    ASSESSMENTS_FILE_NAME
                )));
                self.overwrite_assessments()?;
            }
        }
        Ok(())
    }

    /// Attempt to load the first assessment and its first condition
    pub fn choose_first_assessment_and_condition(&mut self) {
        match self.data.assessments.first() {
            Some((assessment, conds)) => {
                self.data.session.chosen_assessment = assessment.clone();
                match conds.first_condition() {
                    Some(cond) => self.data.session.chosen_condition = cond.clone(),
                    None => self.data.session.chosen_condition.clear(),
                }
                self.data.current_session = conds.session;
            }
            None => {
                self.data.session.chosen_assessment.clear();
                self.data.session.chosen_condition.clear()
            }
        }
    }

    pub fn create_example_ksfs_file(&self) -> Result<()> {
        let mut writer = std::fs::File::create_new(Path::new(&&self.path_to_ksf_data()))?;
        std::io::Write::write_all(&mut writer, KsfsData::example().to_json()?.as_bytes())?;
        std::io::Write::flush(&mut writer)?;
        Ok(())
    }

    pub fn create_example_assessments_file(&self) -> Result<()> {
        let mut writer = std::fs::File::create_new(Path::new(&self.path_to_assessments()))?;
        std::io::Write::write_all(
            &mut writer,
            AssessmentsData::example().to_json()?.as_bytes(),
        )?;
        std::io::Write::flush(&mut writer)?;
        Ok(())
    }

    pub fn unload_client(&mut self) {
        self.data.clear();
        let default_dir = DEFAULT_DIRECTORY
            .get_or_init(|| HARDCODED_ROOT_DIR.into())
            .clone();
        self.edit_assessments
            .prepare(&self.data, default_dir.clone());
        self.edit_ksfs.prepare(&self.data, default_dir.clone());
        self.ioa_page
            .prepare(default_dir.clone(), default_dir.clone());
    }

    pub fn load_client(&mut self, path: &PathBuf) {
        // Determine if the client file exists
        match ClientData::from_file(&Path::new(path).join(CLIENT_DATA_FILE_NAME))
            .context("error reading client_data.txt")
        {
            Ok(client) => {
                // Clear all data
                self.data.clear();
                // Load the client data into ClientData
                self.data.client = client;

                // Load the KSF Data
                let ksf_path = self.path_to_ksf_data();
                match KsfsData::from_file(&ksf_path) {
                    Ok(ksf_data) => {
                        self.data.ksfs = ksf_data;
                        self.edit_ksfs.prepare(&self.data, ksf_path.clone());
                        if let Some((name, _)) = self.data.ksfs.first() {
                            self.data.session.chosen_ksf_name = name.clone()
                        }
                    }
                    Err(e) => {
                        if *&e
                            .to_string()
                            .contains("The system cannot find the file specified")
                        {
                            windows_error_dialog(anyhow::anyhow!(format!(
                                "{} could not be found, a default file has been created",
                                KSF_FILE_NAME
                            )));
                            match self.create_example_ksfs_file() {
                                Ok(_) => match KsfsData::from_file(&ksf_path) {
                                    Ok(new_data) => {
                                        self.data.ksfs = new_data;
                                        self.edit_ksfs.prepare(&self.data, ksf_path.clone());
                                        if let Some((name, _)) = self.data.ksfs.first() {
                                            self.data.session.chosen_ksf_name = name.clone()
                                        }
                                    }
                                    Err(e) => windows_error_dialog(e),
                                },
                                Err(e) => windows_error_dialog(e),
                            }
                        } else {
                            windows_error_dialog(e.context(format!(
                                "unable to read {}, the file may be corrupt",
                                KSF_FILE_NAME
                            )));
                        }
                    }
                };

                // Load the Assessments Data
                let assessments_path = self.path_to_assessments();
                match AssessmentsData::from_file(&assessments_path) {
                    Ok(assessments_data) => {
                        self.data.assessments = assessments_data;
                        self.edit_assessments
                            .prepare(&self.data, assessments_path.clone());
                        self.choose_first_assessment_and_condition();
                    }
                    Err(e) => {
                        if *&e
                            .to_string()
                            .contains("The system cannot find the file specified")
                        {
                            windows_error_dialog(anyhow::anyhow!(format!(
                                "{} could not be found, a default file has been created",
                                ASSESSMENTS_FILE_NAME
                            )));
                            match self.create_example_assessments_file() {
                                Ok(_) => match AssessmentsData::from_file(&assessments_path) {
                                    Ok(new_data) => {
                                        self.data.assessments = new_data;
                                        self.edit_assessments
                                            .prepare(&self.data, assessments_path.clone());
                                        self.choose_first_assessment_and_condition();
                                    }
                                    Err(e) => windows_error_dialog(e),
                                },
                                Err(e) => windows_error_dialog(e),
                            }
                        } else {
                            windows_error_dialog(e.context(format!(
                                "unable to read {}, the file may be corrupt",
                                ASSESSMENTS_FILE_NAME
                            )));
                        }
                    }
                }

                self.ioa_page.prepare(
                    self.path_to_session_records_dir(),
                    self.path_to_ioa_data_dir(),
                );

                if cfg!(debug_assertions) {
                    self.data.session.data_collector = String::from("EX");
                    self.data.session.therapist = String::from("EX");
                }

                self.check_if_ready_to_start_session();
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
        Shuffler::view(self, ui);
        DebugPage::view(self, ui);

        // ### Top Bar ###
        // To go fully across it must be specified before any other panel
        // Nothing here can be interactable because we use Tab and Space as controls on the Session Page
        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.request_repaint_after_secs(5.0);
                ui.label(format!("{}", date_time_string(&Local::now())));

                if cfg!(debug_assertions) {
                    let warn_color = ui.visuals().warn_fg_color;
                    ui.label(RichText::new("⚠ Debug build ⚠").small().color(warn_color))
                        .on_hover_text("egui was compiled with debug assertions enabled.");
                    let dt = ui.input(|i| i.unstable_dt);
                    let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };
                    ui.label(
                        RichText::from(format!("FPS: {:.0}", fps.round()))
                            .monospace()
                            .color(warn_color),
                    );
                }
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
            Page::RunSession => self.view_session(ui),
            Page::Ioa => IoaPage::view(self, ui),
            Page::PrepareSession => PrepareSession::view(self, ui),
            Page::CreateClient => NewClient::view(self, ui),
            Page::CreateKsf => EditKsfData::view(self, ui),
            Page::CreateAssessments => EditAssessments::view(self, ui),
            Page::Settings => Settings::view(self, ui),
            Page::Credits => Credits::view(self, ui),
            Page::PreferenceAssessment => PreferenceAssessment::view(self, ui),
        }
    }
}
