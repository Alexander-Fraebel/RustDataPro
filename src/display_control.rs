use crate::app::DataPro;

pub struct DisplayControl {
    pub active_page: Page,
    pub timers_open: bool,
    pub random_open: bool,
    pub sidebar_open: bool,
    pub debug_open: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Page {
    PrepareSession,
    RunSession,
    Ioa,
    CreateClient,
    EditKsfs,
    EditAssessments,
    Settings,
    Credits,
    PreferenceAssessment,
}

impl DataPro {
    pub fn go_to_prep_session(&mut self) {
        if self.edit_ksfs.changes_made {
            self.save_and_reload_ksfs();
        }
        if self.edit_assessments.changes_made {
            self.save_and_reload_assessments();
        }
        self.display_info.active_page = Page::PrepareSession;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_run_session(&mut self) {
        self.display_info.active_page = Page::RunSession;
        self.display_info.sidebar_open = false;
        self.display_info.timers_open = false;
        self.display_info.random_open = false;
    }

    pub fn go_to_ioa(&mut self) {
        self.display_info.active_page = Page::Ioa;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_create_client(&mut self) {
        self.display_info.active_page = Page::CreateClient;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_edit_assessments(&mut self) {
        self.edit_assessments
            .prepare(&self.data, self.path_to_assessments());
        self.display_info.active_page = Page::EditAssessments;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_edit_ksf(&mut self) {
        self.edit_ksfs.prepare(&self.data, self.path_to_ksf_data());
        self.display_info.active_page = Page::EditKsfs;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_settings(&mut self) {
        self.display_info.active_page = Page::Settings;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_credits(&mut self) {
        self.display_info.active_page = Page::Credits;
        self.display_info.sidebar_open = true;
    }

    pub fn go_to_preference_assessment(&mut self) {
        self.display_info.active_page = Page::PreferenceAssessment;
    }

    pub fn toggle_debug_display(&mut self) {
        self.display_info.debug_open = !self.display_info.debug_open;
    }

    pub fn toggle_timer_display(&mut self) {
        self.display_info.timers_open = !self.display_info.timers_open;
    }

    pub fn toggle_random_display(&mut self) {
        self.display_info.random_open = !self.display_info.random_open;
    }
}
