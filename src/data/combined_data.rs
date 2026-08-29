use std::path::Path;

use crate::data::{Assessment, AssessmentsData, ClientData, Ksf, KsfsData, SessionData};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const NO_CLIENT: &'static str = "no Client loaded";
pub const NO_KSF: &'static str = "no KSF loaded";
pub const NO_ASSESSMENT: &'static str = "no Assessment chosen";
pub const NO_CONDITION: &'static str = "no Condition chosen";
pub const NO_PRIMARY_THERAPIST: &'static str = "no Primary Therapist provided";
pub const NO_CASE_MANAGER: &'static str = "no Case Manager provided";
pub const NO_SESSION_THERAPIST: &'static str = "no Session Therapist provided";
pub const NO_LOCATION: &'static str = "no Location provided";
pub const NO_DATA_COLLECTOR: &'static str = "no Data Collector provided";
pub const INVALID_DATE: &'static str = "Date of Admission is not valid";
pub const INVALID_SESSION: &'static str = "Session Number cannot be 0";
pub const INVALID_MAX_SESSION: &'static str = "Max Session Length cannot be 0.0 seconds";
pub const INVALID_MAX_TOTAL: &'static str = "Max Total Length cannot be 0.0 seconds";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ClientAndSessionData {
    pub client: ClientData,
    pub session: SessionData,
    pub assessments: AssessmentsData,
    pub ksfs: KsfsData,
    pub current_session: u32,
    pub misconfigs: String,
}

impl ClientAndSessionData {
    crate::to_and_from_json!(
        self,
        "unable to make ClientAndSessionData from file",
        "unable to convert ClientAndSessionData to json"
    );

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

    /// Increments the session number for the chosen assessment and set the current_session value to that number.
    /// If there is no chosen assessment or it is invalid current_session is set to u32::MAX
    pub fn increment_current_session(&mut self) {
        if let Some(n) = self.assessments.get_mut(&self.session.chosen_assessment) {
            n.session += 1;
            self.current_session = n.session;
        } else {
            self.current_session = u32::MAX;
        }
    }

    pub fn chosen_ksf_name(&self) -> &String {
        &self.session.chosen_ksf_name
    }

    pub fn chosen_ksf(&self) -> Option<&Ksf> {
        self.ksfs.get(self.chosen_ksf_name())
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
        !self.chosen_ksf_name().is_empty()
    }

    pub fn assessment_chosen(&self) -> bool {
        !self.active_assessment_name().is_empty()
    }

    pub fn condition_chosen(&self) -> bool {
        !self.active_condition_name().is_empty()
    }

    pub fn max_session_length_set_correctly(&self) -> bool {
        // It is false that: session length is limited and the maximum session length is zero
        !(self.session.limit_session_length && self.session.maximum_session_length == 0.0)
    }

    pub fn max_total_length_set_correctly(&self) -> bool {
        !(self.session.limit_total_length && self.session.maximum_total_length == 0.0)
    }

    pub fn update_misconfigurations(&mut self) {
        let mut misconfigs = Vec::new();
        if !self.client_loaded() {
            misconfigs.push(NO_CLIENT);
            self.misconfigs = misconfigs.join("\n"); // short circuit here as all others depend on this
            return ();
        }
        if !self.ksf_loaded() {
            misconfigs.push(NO_KSF);
        }
        if self.client.location.is_empty() {
            misconfigs.push(NO_LOCATION);
        }
        if !self.client_admission_valid() {
            misconfigs.push(INVALID_DATE);
        }
        if self.current_session == 0 {
            misconfigs.push(INVALID_SESSION);
        }
        if self.client.case_manager.is_empty() {
            misconfigs.push(NO_CASE_MANAGER);
        }
        if self.client.primary_therapist.is_empty() {
            misconfigs.push(NO_PRIMARY_THERAPIST);
        }
        if self.session.therapist.is_empty() {
            misconfigs.push(NO_SESSION_THERAPIST);
        }
        if self.session.data_collector.is_empty() {
            misconfigs.push(NO_DATA_COLLECTOR);
        }
        if !self.assessment_chosen() {
            misconfigs.push(NO_ASSESSMENT);
        }
        if !self.condition_chosen() {
            misconfigs.push(NO_CONDITION);
        }
        if !self.max_session_length_set_correctly() {
            misconfigs.push(INVALID_MAX_SESSION);
        }
        if !self.max_total_length_set_correctly() {
            misconfigs.push(INVALID_MAX_TOTAL);
        }

        self.misconfigs = misconfigs.join("\n");
    }
}
