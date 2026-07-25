pub mod assessment_data;
pub mod client_data;
pub mod ioa_data;
pub mod ksf_data;
pub mod ksf_data_alt;
pub mod output_data;
pub mod session_data;
pub mod timeline;
pub mod timer;

pub use assessment_data::*;
pub use client_data::*;
pub use ioa_data::*;
pub use ksf_data_alt::*;
pub use output_data::*;
pub use session_data::*;
pub use timeline::*;
pub use timer::*;

#[derive(Debug, Default)]
pub struct Data {
    pub client: ClientData,
    pub session: SessionData,
    pub assessments: AssessmentsData,
    pub ksfs: KsfData,
}

impl Data {
    pub fn clear(&mut self) {
        *self = Self::default()
    }

    pub fn client_loaded(&self) -> bool {
        !self.client.id.is_empty()
    }

    pub fn ksf_loaded(&self) -> bool {
        !self.session.chosen_ksf.is_empty()
    }

    pub fn assessment_chosen(&self) -> bool {
        !self.session.chosen_assessment.is_empty()
    }

    pub fn condition_chosen(&self) -> bool {
        !self.session.chosen_condition.is_empty()
    }

    pub fn chosen_condition(&self) -> &String {
        &self.session.chosen_condition
    }

    pub fn chosen_assessment(&self) -> &String {
        &self.session.chosen_assessment
    }
}
