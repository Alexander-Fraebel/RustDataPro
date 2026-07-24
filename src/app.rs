use crate::{
    data::{AssessmentsData, ClientData, Data, KsfData, SessionData},
    display_controller::{DisplayInfo, Page},
    ioa::IoaPage,
    pages::{
        NewClient, NewKsf, PrepareSession, RandomServices, SessionPage, Sidebar, Timers,
        new_assessments::NewAssessments,
    },
    utils::{date_time_string, quick_file_name, windows_error_dialog},
};
use anyhow::{Context, Result};
use chrono::Local;
use egui::{TextBuffer, Visuals};
use egui_file_dialog::FileDialog;
use std::path::{Path, PathBuf};

pub const DEFAULT_ROOT_DIRECTORY: &'static str = "C:\\";
pub const DEFAULT_ROOT_DIRECTORY_NAME: &'static str = "DataProClients";
pub const DEFAULT_ZOOM: f32 = 1.5;

pub const CLIENT_DATA_FILE_NAME: &'static str = "client_data.txt";
pub const ASSESSMENTS_FILE_NAME: &'static str = "assessments.txt";
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
    pub new_ksf_page: NewKsf,
    pub new_assessments_page: NewAssessments,
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
                ksf: KsfData::default(),
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
            new_ksf_page: NewKsf::default(),
            new_assessments_page: NewAssessments::default(),
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

    /// Path to the client data file, if one is available.
    pub fn client_data_file_path(&self) -> Result<PathBuf> {
        if !self.data.client_loaded() {
            return Err(anyhow::anyhow!(
                "cannot find client data file because no client is selected"
            ));
        }
        let path = Path::new(&self.root_directory)
            .join(&self.data.client.id.to_string())
            .join(CLIENT_DATA_FILE_NAME);
        Ok(path.to_path_buf())
    }

    pub fn overwrite_client_data_file(&self) -> Result<()> {
        std::fs::write(
            self.client_data_file_path()?,
            &self
                .data
                .client
                .to_json()
                .with_context(|| "failed to create json version of client data file")?,
        )
        .with_context(|| "while attempting to overwrite client_data.txt")?;

        Ok(())
    }

    pub fn client_session_data_path(&self) -> Result<PathBuf> {
        if !self.data.client_loaded() {
            return Err(anyhow::anyhow!(
                "cannot find {} folder because {}",
                SESSION_DATA_FOLDER_NAME,
                NO_CLIENT
            ));
        }
        let path = Path::new(&self.root_directory)
            .join(&self.data.client.id.to_string())
            .join(SESSION_DATA_FOLDER_NAME);
        Ok(path.to_path_buf())
    }

    pub fn client_ioa_data_path(&self) -> Result<PathBuf> {
        if !self.data.client_loaded() {
            return Err(anyhow::anyhow!(
                "cannot find {} folder because {}",
                IOA_DATA_FOLDER_NAME,
                NO_CLIENT
            ));
        }
        let path = Path::new(&self.root_directory)
            .join(&self.data.client.id.to_string())
            .join(IOA_DATA_FOLDER_NAME);
        Ok(path.to_path_buf())
    }

    pub fn load_ksf(&mut self, path: &PathBuf) {
        match KsfData::from_file(&path) {
            Ok(ksf) => {
                self.data.ksf = ksf;
                self.data.ksf.name = quick_file_name(&path).to_string();
            }
            Err(e) => {
                self.data.ksf = KsfData::default();
                windows_error_dialog(e);
            }
        };
    }

    pub fn load_client_file(&mut self, path: &PathBuf) {
        match ClientData::from_file(&Path::new(path).join(CLIENT_DATA_FILE_NAME))
            .context("error reading client_data.txt")
        {
            Ok(client) => {
                self.data.client = client;
                self.data.client.current_session += 1; // We are always one session ahead of the last saved value
                match AssessmentsData::from_file(&Path::new(path).join(ASSESSMENTS_FILE_NAME))
                    .context("error reading assessments.txt")
                {
                    Ok(a) => self.data.assessments = a,
                    Err(e) => {
                        windows_error_dialog(e);
                        self.data.assessments = AssessmentsData::default();
                    }
                };
                self.data.ksf = KsfData::default();
                // Attempt to load the first assessment and its first condition
                match self.data.assessments.first() {
                    Some((assessment, conds)) => {
                        self.data.session.chosen_assessment = assessment.clone();
                        match conds.first() {
                            Some(cond) => self.data.session.chosen_condition = cond.clone(),
                            None => self.data.session.chosen_condition.clear(),
                        }
                    }
                    None => self.data.session.chosen_assessment.clear(),
                }
            }
            Err(e) => {
                self.data.client = ClientData::default();
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
            Page::CreateKsf => NewKsf::view(self, ui),
            Page::CreateAssessments => NewAssessments::view(self, ui),
        }
    }
}
