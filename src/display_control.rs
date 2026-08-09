pub struct DisplayControl {
    pub active_page: Page,
    pub timers_open: bool,
    pub random_open: bool,
    pub sidebar_open: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Page {
    PrepareSession,
    RunSession,
    Ioa,
    CreateClient,
    CreateKsf,
    CreateAssessments,
    Settings,
    Credits,
    Debug,
}

impl DisplayControl {
    pub fn go_to_prep_session(&mut self) {
        self.active_page = Page::PrepareSession;
        self.sidebar_open = true;
    }

    pub fn go_to_run_session(&mut self) {
        self.active_page = Page::RunSession;
        self.sidebar_open = false;
        self.timers_open = false;
        self.random_open = false;
    }

    pub fn go_to_ioa(&mut self) {
        self.active_page = Page::Ioa;
        self.sidebar_open = false;
    }

    pub fn go_to_new_client(&mut self) {
        self.active_page = Page::CreateClient;
        self.sidebar_open = false;
    }

    pub fn go_to_new_assessments(&mut self) {
        self.active_page = Page::CreateAssessments;
        self.sidebar_open = false;
    }

    pub fn go_to_new_ksf(&mut self) {
        self.active_page = Page::CreateKsf;
        self.sidebar_open = false;
    }

    pub fn go_to_settings(&mut self) {
        self.active_page = Page::Settings;
        self.sidebar_open = false;
    }

    pub fn go_to_credits(&mut self) {
        self.active_page = Page::Credits;
        self.sidebar_open = false;
    }

    pub fn go_to_debug(&mut self) {
        self.active_page = Page::Debug;
        self.sidebar_open = false;
    }

    pub fn toggle_timer_display(&mut self) {
        self.timers_open = !self.timers_open;
    }

    pub fn toggle_random_display(&mut self) {
        self.random_open = !self.random_open;
    }
}
