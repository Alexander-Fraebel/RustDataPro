use crate::app::DataPro;

pub struct DisplayControl {
    pub active_page: Page,
    pub sidebar_open: bool,
    pub debug_open: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Page {
    About,
    CreateClient,
    EditAssessments,
    EditKsfs,
    Ioa,
    PreferenceAssessment,
    PrepareSession,
    RunSession,
    Settings,
    Shuffler,
    Timers,
}

impl DataPro {
    fn save_and_reload_ksfs_and_assessments(&mut self) {
        if self.edit_ksfs.changes_made {
            self.save_and_reload_ksfs();
            self.edit_ksfs.changes_made = false;
        }
        if self.edit_assessments.changes_made {
            self.save_and_reload_assessments();
            self.edit_assessments.changes_made = false;
        }
    }

    pub fn go_to_prep_session(&mut self) {
        self.save_and_reload_ksfs_and_assessments();
        self.display_info.active_page = Page::PrepareSession;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_run_session(&mut self) {
        self.save_and_reload_ksfs_and_assessments();
        self.display_info.active_page = Page::RunSession;
        self.display_info.sidebar_open = false;
    }

    pub fn go_to_ioa(&mut self) {
        self.save_and_reload_ksfs_and_assessments();
        self.display_info.active_page = Page::Ioa;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_create_client(&mut self) {
        self.save_and_reload_ksfs_and_assessments();
        self.display_info.active_page = Page::CreateClient;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_edit_assessments(&mut self) {
        self.save_and_reload_ksfs_and_assessments();
        self.edit_assessments
            .prepare(&self.data, self.path_to_assessments());
        self.display_info.active_page = Page::EditAssessments;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_edit_ksf(&mut self) {
        self.save_and_reload_ksfs_and_assessments();
        self.edit_ksfs.prepare(&self.data, self.path_to_ksf_data());
        self.display_info.active_page = Page::EditKsfs;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_settings(&mut self) {
        self.save_and_reload_ksfs_and_assessments();
        self.display_info.active_page = Page::Settings;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_about(&mut self) {
        self.save_and_reload_ksfs_and_assessments();
        self.display_info.active_page = Page::About;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_preference_assessment(&mut self) {
        self.save_and_reload_ksfs_and_assessments();
        self.display_info.active_page = Page::PreferenceAssessment;
    }

    pub fn go_to_timers(&mut self) {
        self.save_and_reload_ksfs_and_assessments();
        self.display_info.active_page = Page::Timers;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_shuffler(&mut self) {
        self.save_and_reload_ksfs_and_assessments();
        self.display_info.active_page = Page::Shuffler;
        self.display_info.sidebar_open = true;
    }

    pub fn toggle_debug_window(&mut self) {
        self.display_info.debug_open = !self.display_info.debug_open;
    }
}
