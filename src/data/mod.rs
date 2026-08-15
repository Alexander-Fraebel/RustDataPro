pub mod assessment_data;
pub mod client_data;
pub mod ioa_data;
pub mod ksf_data;
pub mod output_data;
pub mod session_data;
pub mod timeline;

pub use assessment_data::*;
pub use client_data::*;
pub use ioa_data::*;
pub use ksf_data::*;
pub use output_data::*;
pub use session_data::*;
pub use timeline::*;

#[derive(Debug, Default)]
pub struct Data {
    pub client: ClientData,
    pub session: SessionData,
    pub assessments: AssessmentsData,
    pub ksfs: KsfsData,
    pub current_session: u32,
}

impl Data {
    pub fn clear(&mut self) {
        *self = Self::default()
    }

    pub fn active_assessment_name(&self) -> &String {
        &self.session.chosen_assessment
    }

    pub fn active_condition_name(&self) -> &String {
        &self.session.chosen_condition
    }

    pub fn active_assessment_data(&mut self) -> Option<&mut Assessment> {
        self.assessments.get_mut(&self.session.chosen_assessment)
    }

    /// Increements the session number for the chosen assessment and set the current_session value to that number.
    /// If there is no chosen assessment or it is invalid current_session is set to u32::MAX
    pub fn increment_current_session(&mut self) {
        if let Some(n) = self.assessments.get_mut(&self.session.chosen_assessment) {
            n.session += 1;
            self.current_session = n.session;
        } else {
            self.current_session = u32::MAX;
        }
    }

    pub fn chosen_ksf(&self) -> &String {
        &self.session.chosen_ksf
    }

    pub fn client_loaded(&self) -> bool {
        !self.client.id.is_empty()
    }

    pub fn client_admission_valid(&self) -> bool {
        match self.client.days_since_admission() {
            Ok(n) => {
                if n.is_negative() {
                    false
                } else {
                    true
                }
            }
            Err(_) => false,
        }
    }

    pub fn ksf_loaded(&self) -> bool {
        !self.chosen_ksf().is_empty()
    }

    pub fn assessment_chosen(&self) -> bool {
        !self.active_assessment_name().is_empty()
    }

    pub fn condition_chosen(&self) -> bool {
        !self.active_condition_name().is_empty()
    }
}
