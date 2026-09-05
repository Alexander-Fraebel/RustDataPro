use crate::{
    config::{
        ASSESSMENTS_FILE_NAME, CLIENT_DATA_FILE_NAME, Config, IOA_DATA_FOLDER_NAME, KSF_FILE_NAME,
        SESSION_DATA_FOLDER_NAME, path_to_config_file,
    },
    data::{AssessmentsData, ClientAndSessionData, ClientData, KsfsData, NO_CLIENT},
    display_control::{DisplayControl, Page},
    ioa::{IoaPage, validate_files::validate_files},
    pages::{
        CreateClient, EditAssessments, EditKsfData, PrepareSession, SessionPage, Shuffler, Timers,
    },
    preference_assessment::PreferenceAssessment,
    quick_error,
    utils::{date_time_string, overwrite_file, windows_error_dialog},
};
use anyhow::{Context, Result};
use chrono::Local;
use egui::{FontDefinitions, Response, RichText, Visuals};
use egui_file_dialog::FileDialog;
use rand::{make_rng, rngs::StdRng};
use std::path::{Path, PathBuf};

pub struct DataPro {
    pub config: Config,

    pub pick_root_directory: FileDialog,
    pub root_directory: PathBuf,

    pub rng: StdRng, // StdRng is currently ChaCha12 initalized from SysRng, any similar prng is more than sufficient

    pub data: ClientAndSessionData,
    pub display_info: DisplayControl,

    pub randomness_page: Shuffler,
    pub timers: Timers,

    pub prep_session: PrepareSession,
    pub session: SessionPage,

    pub ioa_page: IoaPage,
    pub new_client_page: CreateClient,
    pub edit_ksfs: EditKsfData,
    pub edit_assessments: EditAssessments,
    pub preference_assessment: PreferenceAssessment,
}

impl Default for DataPro {
    fn default() -> Self {
        let config = match Config::try_from_current_dir() {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        };

        // The provided directory should always be valid on Windows and we are not handling any other OS
        let root_dir = PathBuf::from(&config.root_dir);

        // If the default directory doesn't exist then create it.
        if !root_dir.exists() {
            quick_error!(std::fs::create_dir(&root_dir).context("cannot create root directory"));
        }

        let mut app = Self {
            config,

            data: ClientAndSessionData::default(),

            rng: make_rng(),

            display_info: DisplayControl {
                active_page: Page::About,
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
            new_client_page: CreateClient::default(),
            edit_ksfs: EditKsfData::default(),
            edit_assessments: EditAssessments::default(),
            preference_assessment: PreferenceAssessment::default(),
        };

        // Initialize everything by "unloading" a client
        app.unload_client();
        app
    }
}

impl DataPro {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load the config file information before the application loads
        let config = match Config::try_from_current_dir() {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        };
        cc.egui_ctx.set_pixels_per_point(config.zoom);

        cc.egui_ctx.set_visuals(Visuals::dark());

        // Custom monospace font
        let mut font_defs = FontDefinitions::default();
        font_defs.font_data.insert(
            "AtkinsonMono".into(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "..\\AtkinsonHyperlegibleMono-VariableFont_wght.ttf"
            ))),
        );
        font_defs.font_data.insert(
            "AtkinsonNext".into(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "..\\AtkinsonHyperlegibleNext-VariableFont_wght.ttf"
            ))),
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

    pub fn ksf_picker(&mut self, ui: &mut egui::Ui) -> Response {
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
            })
            .response
    }

    pub fn root_dir(&self) -> PathBuf {
        PathBuf::from(&self.config.root_dir)
    }

    pub fn reload_config(&mut self, ui: &mut egui::Ui) {
        match Config::try_from_current_dir() {
            Ok(config) => {
                self.config = config;
                ui.ctx().set_pixels_per_point(self.config.zoom);
            }
            Err(e) => windows_error_dialog(e),
        }
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
        self.path_to(CLIENT_DATA_FILE_NAME)
            .unwrap_or_else(|_| self.root_dir())
    }

    /// Path to assessments.txt if a client has been chosen or the default directory otherwise.
    pub fn path_to_assessments(&self) -> PathBuf {
        self.path_to(ASSESSMENTS_FILE_NAME)
            .unwrap_or_else(|_| self.root_dir())
    }

    /// Path to ksf_data.txt if a client has been chosen or the default directory otherwise.
    pub fn path_to_ksf_data(&self) -> PathBuf {
        if self.data.client.alternate_ksfs_path.is_empty() {
            self.path_to(KSF_FILE_NAME)
                .unwrap_or_else(|_| self.root_dir())
        } else {
            self.data.client.alternate_ksfs_path.clone().into()
        }
    }

    /// Path to Session Records if a client has been chosen or the default directory otherwise.
    pub fn path_to_session_records_dir(&self) -> PathBuf {
        self.path_to(SESSION_DATA_FOLDER_NAME)
            .unwrap_or_else(|_| self.root_dir())
    }

    /// Path to IOA Data if a client has been chosen or the default directory otherwise.
    pub fn path_to_ioa_data_dir(&self) -> PathBuf {
        self.path_to(IOA_DATA_FOLDER_NAME)
            .unwrap_or_else(|_| self.root_dir())
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

    pub fn overwrite_config(&self) -> Result<()> {
        overwrite_file(path_to_config_file(), &self.config.to_json()?)
    }

    pub fn load_ksf(&mut self, path: &PathBuf) {
        match KsfsData::from_file_path(&path) {
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
        match AssessmentsData::from_file_path(&assessments_path) {
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

    pub fn try_create_example_ksfs_file(&self) -> Result<()> {
        let mut writer = std::fs::File::create_new(Path::new(&&self.path_to_ksf_data()))?;
        std::io::Write::write_all(
            &mut writer,
            self.config.default_ksfs_data.to_json()?.as_bytes(),
        )?;
        std::io::Write::flush(&mut writer)?;
        Ok(())
    }

    pub fn try_create_example_assessments_file(&self) -> Result<()> {
        let mut writer = std::fs::File::create_new(Path::new(&self.path_to_assessments()))?;
        std::io::Write::write_all(
            &mut writer,
            self.config.default_assessments_data.to_json()?.as_bytes(),
        )?;
        std::io::Write::flush(&mut writer)?;
        Ok(())
    }

    pub fn unload_client(&mut self) {
        self.data.clear();
        let default_dir = self.root_dir();
        self.edit_assessments
            .prepare(&self.data, default_dir.clone());
        self.prepare_edit_ksf_page();
        self.ioa_page
            .prepare(default_dir.clone(), default_dir.clone());
    }

    pub fn load_client(&mut self, path: &PathBuf) {
        // Determine if the client file exists
        match ClientData::from_file_path(&Path::new(path).join(CLIENT_DATA_FILE_NAME))
            .context("error reading client_data.txt")
        {
            Ok(client) => {
                // Clear all data
                self.data.clear();
                // Load the client data into ClientData
                self.data.client = client;

                // Load the KSF Data
                let ksf_path = self.path_to_ksf_data();
                match KsfsData::from_file_path(&ksf_path) {
                    Ok(ksf_data) => {
                        self.data.ksfs = ksf_data;
                        self.prepare_edit_ksf_page();
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
                                "{} could not be found\na default will be created",
                                KSF_FILE_NAME
                            )));
                            match self.try_create_example_ksfs_file() {
                                Ok(_) => match KsfsData::from_file_path(&ksf_path) {
                                    Ok(new_data) => {
                                        self.data.ksfs = new_data;
                                        self.prepare_edit_ksf_page();
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
                                "unable to read {}\nthe file may be corrupt",
                                KSF_FILE_NAME
                            )));
                        }
                    }
                };

                // Load the Assessments Data
                let assessments_path = self.path_to_assessments();
                match AssessmentsData::from_file_path(&assessments_path) {
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
                                "{} could not be found\na default file will be created",
                                ASSESSMENTS_FILE_NAME
                            )));
                            match self.try_create_example_assessments_file() {
                                Ok(_) => match AssessmentsData::from_file_path(&assessments_path) {
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
                                "unable to read {}\nthe file may be corrupt",
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
                    self.data.session.data_collector = String::from("EXAMPLE");
                    self.data.session.therapist = String::from("EXAMPLE");
                }

                self.data.update_misconfigurations()
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
        self.view_debug_page(ui);

        // ### Top Bar ###
        // To go fully across it must be specified before any other panel
        // Nothing here can be interactable because we use Tab and Space as controls on the Session Page
        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.request_repaint_after_secs(5.0);
                ui.monospace(format!("{}", date_time_string(&Local::now())));

                if cfg!(debug_assertions) {
                    let warn_color = ui.visuals().warn_fg_color;
                    ui.label(RichText::new("⚠ Debug build ⚠").small().color(warn_color))
                        .on_hover_text("egui was compiled with debug assertions enabled.");
                }
            });
        });

        // ### Sidebar ###
        // To show it must go before any other panel
        // It must be not to rendered (even if not shown) when Session is active because it may capture keypresses
        if self.display_info.sidebar_open {
            self.view_sidebar(ui)
        };

        // ### Main Panel ###
        match self.display_info.active_page {
            Page::RunSession => self.view_session(ui),
            Page::Ioa => self.view_ioa(ui),
            Page::PrepareSession => self.view_prep(ui),
            Page::CreateClient => self.view_create_client(ui),
            Page::EditKsfs => self.view_edit_ksf_page(ui),
            Page::EditAssessments => self.view_edit_assessments_page(ui),
            Page::Settings => self.view_settings(ui),
            Page::About => self.view_about_page(ui),
            Page::PreferenceAssessment => self.view_preference_assessment(ui),
            Page::Shuffler => self.view_shuffler(ui),
            Page::Timers => self.view_timers(ui),
        }
    }
}
