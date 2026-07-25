use crate::{
    data::{AssessmentsData, ClientData, Data, KsfData, SessionData},
    display_controller::{DisplayInfo, Page},
    ioa::IoaPage,
    pages::{
        EditKsfData, NewClient, PrepareSession, RandomServices, SessionPage, Sidebar, Timers,
        edit_assessments::EditAssessments,
    },
    utils::{date_time_string, windows_error_dialog},
};
use anyhow::{Context, Result};
use chrono::Local;
use egui::{TextBuffer, Visuals};
use egui_file_dialog::FileDialog;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

pub const DEFAULT_ROOT_DIRECTORY: &'static str = "C:\\";
pub const DEFAULT_ROOT_DIRECTORY_NAME: &'static str = "DataProClients";
pub const DEFAULT_ZOOM: f32 = 1.5;

pub const CLIENT_DATA_FILE_NAME: &'static str = "client_data.txt";
pub const ASSESSMENTS_FILE_NAME: &'static str = "assessments.txt";
pub const KSF_FILE_NAME: &'static str = "ksf_data.txt";
pub const SESSION_DATA_FOLDER_NAME: &'static str = "Session Records";
pub const IOA_DATA_FOLDER_NAME: &'static str = "IOA Data";

pub const NO_CLIENT: &'static str = "no client loaded";
pub const NO_KSF: &'static str = "no KSF loaded";
pub const NO_ASSESSMENT: &'static str = "no assessment chosen";
pub const NO_CONDITION: &'static str = "no condition chosen";

pub struct DataPro {
    pub pick_root_directory: FileDialog,
    pub root_directory: PathBuf,

    pub data: Data,
    pub display_info: DisplayInfo,

    pub pick_client_folder: FileDialog,
    pub pick_ksf: FileDialog,

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
        // In debug mode use the current directory
        #[cfg(debug_assertions)]
        let root_directory =
            Path::new(&std::env::current_dir().unwrap_or(PathBuf::from(DEFAULT_ROOT_DIRECTORY)))
                .join(DEFAULT_ROOT_DIRECTORY_NAME);

        // In release mode use the C: drive
        #[cfg(not(debug_assertions))]
        let root_directory = Path::new(DEFAULT_ROOT_DIRECTORY).join(DEFAULT_ROOT_DIRECTORY_NAME);

        // If the directory chosen doesn't exist crate it.
        if !root_directory.exists() {
            match std::fs::create_dir(&root_directory) {
                Ok(_) => (),
                Err(e) => windows_error_dialog(e.into()),
            }
        }

        Self {
            data: Data {
                client: ClientData::default(),
                session: SessionData::default(),
                assessments: AssessmentsData::default(),
                ksfs: KsfData::default(),
            },

            display_info: DisplayInfo {
                active_page: Default::default(),
                timers_open: false,
                random_open: false,
                sidebar_open: true,
                zoom: DEFAULT_ZOOM,
            },

            pick_root_directory: FileDialog::new().initial_directory(root_directory.clone()),
            pick_client_folder: FileDialog::new().initial_directory(root_directory.clone()),
            pick_ksf: FileDialog::default().initial_directory(root_directory.clone()),
            root_directory,

            randomness_page: RandomServices::default(),
            timers: Timers::default(),

            prep_session: PrepareSession::default(),
            session_page: SessionPage::new(),

            ioa_page: IoaPage::default(),
            new_client_page: NewClient::default(),
            edit_ksfs: EditKsfData::default(),
            edit_assessments: EditAssessments::default(),
        }
    }
}

impl DataPro {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_pixels_per_point(DEFAULT_ZOOM);
        cc.egui_ctx.set_visuals(Visuals::dark());
        Default::default()
    }

    pub fn ready_to_start_session(&mut self) -> bool {
        if !self.data.client_loaded() {
            self.prep_session.session_start_error = NO_CLIENT;
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
        match self.path_to(name) {
            Ok(pb) => {
                if pb.exists() {
                    std::fs::write(pb, data)?
                } else {
                    let mut writer = BufWriter::new(File::create_new(pb)?);
                    writer.write_all(data.as_bytes())?;
                    writer.flush()?;
                }
            }
            Err(e) => return Err(e),
        }
        Ok(())
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

    pub fn load_client_file(&mut self, path: &PathBuf) {
        // Determine if the client file exists
        match ClientData::from_file(&Path::new(path).join(CLIENT_DATA_FILE_NAME))
            .context("error reading client_data.txt")
        {
            Ok(client) => {
                self.data.client = client;
                self.data.client.current_session += 1; // We are always one session ahead of the last saved value

                // Reset the KSF
                match KsfData::from_file(&Path::new(path).join(KSF_FILE_NAME)) {
                    Ok(ks) => self.data.ksfs = ks,
                    Err(_) => self.data.ksfs = KsfData::default(),
                };
                match self.data.ksfs.first() {
                    Some((name, _ksf)) => self.data.session.chosen_ksf = name.clone(),
                    None => self.data.session.chosen_ksf.clear(),
                }

                // Load assessments from file
                // If it doesen't exist then reset AssessmentsData, errors will be visible and easy to repair in the application
                match AssessmentsData::from_file(&Path::new(path).join(ASSESSMENTS_FILE_NAME)) {
                    Ok(a) => self.data.assessments = a,
                    Err(_) => self.data.assessments = AssessmentsData::default(),
                };
                self.choose_first_assessment_and_condition();
            }
            Err(e) => {
                self.data.clear();
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
        }
    }
}
