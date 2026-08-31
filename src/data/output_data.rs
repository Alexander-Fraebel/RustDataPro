use crate::data::{
    DataCollectionType::{self},
    Ksf, SessionData,
    timeline::Timeline,
};
use anyhow::{Context, Result};
use egui::Key;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Output of a single session. Includes the Client and Session data along with the recorded keypresses and times, and the KSF to translate those.
#[derive(Serialize, Deserialize, Clone)]
pub struct OutputData {
    pub datetime: String,
    pub client_name: String,
    pub client_id: String,
    pub case_manager: String,
    pub primary_therapist: String,
    pub session_number: u32,
    pub days_since_admissions: i32,
    pub location: String,
    pub session: SessionData,
    pub session_duration: f32,
    pub frequency: IndexMap<Key, u32>,
    pub duration: IndexMap<Key, (u32, f32)>,
    pub timeline: Timeline,
    pub ksf: Ksf,
}

impl OutputData {
    pub fn session_number(&self) -> u32 {
        self.session_number
    }

    pub fn data_type(&self) -> DataCollectionType {
        self.session.data_collection_type
    }

    pub fn client_initials(&self) -> String {
        self.client_name
            .chars()
            .filter(|c| c.is_ascii_uppercase())
            .join("")
    }

    crate::to_and_from_json!(
        self,
        "unable to make OutputData from file",
        "unable to convert OutputData to json"
    );
}

#[test]
fn create_test_data() {
    use crate::{data::ClientData, utils::rounded_f32};
    use egui::Key;
    use rand::{RngExt, make_rng, rngs::StdRng, seq::IndexedRandom};
    use std::fs::File;

    let mut rng: StdRng = make_rng();

    let mut client = ClientData::default();
    client.name = String::from("BW");

    for session in 11..16 {
        // client.current_session = session;
        let mut session_data = SessionData::default();
        session_data.data_collection_type = DataCollectionType::Primary;

        let ksf = Ksf::example();
        let mut fkeys = Vec::new();

        let mut frequency: IndexMap<Key, u32> = IndexMap::new();
        let (freq, dura) = ksf.keys();
        for k in freq {
            frequency.insert(*k, 0);
            fkeys.push(*k);
        }
        let mut duration: IndexMap<Key, (u32, f32)> = IndexMap::new();
        let mut dkeys = Vec::new();
        for k in dura {
            let n: u32 = rng.random_range(..50);
            let f: f32 = rng.random::<f32>() * 50.0;
            duration.insert(*k, (n, rounded_f32(f)));
            dkeys.push(*k);
        }

        let mut timeline = Timeline::default();
        let mut session_time = 0.0;
        timeline.push((Key::Tab, rounded_f32(session_time)));
        for _ in 0..150 {
            session_time = session_time + rng.random::<f32>() * 4.0;
            if rng.random_bool(0.9) {
                let t = rounded_f32(session_time);
                if rng.random_bool(0.5) {
                    let k = fkeys.choose(&mut rng).unwrap();
                    *frequency.get_mut(k).unwrap() += 1;
                    timeline.push((*k, t));
                } else {
                    let k = dkeys.choose(&mut rng).unwrap();
                    timeline.push((*k, t));
                };
            }
        }
        timeline.push((Key::Escape, session_time));

        let prim = OutputData {
            datetime: String::from("TEST FILE"),
            session: session_data.clone(),
            session_duration: rounded_f32(session_time),
            frequency: frequency.clone(),
            duration: duration.clone(),
            timeline: timeline.clone(),
            ksf: ksf.clone(),
            client_name: client.name.clone(),
            client_id: client.id.clone(),
            case_manager: client.case_manager.clone(),
            primary_therapist: client.primary_therapist.clone(),
            session_number: session,
            days_since_admissions: client.days_since_admission().unwrap_or(-99999),
            location: client.location.clone(),
        };

        // Jitter the timing for the keypresses
        session_data.data_collection_type = DataCollectionType::Reliability;
        for (_k, t) in timeline.iter_mut() {
            *t += (rng.random::<f32>() - 0.5) * 0.7;
        }
        let (freq, dura) = ksf.keys();
        // Jitter the duration lengths and counts
        for k in dura {
            let f: f32 = (rng.random::<f32>() - 0.5) * 5.0;
            let d = duration.get_mut(k).unwrap();
            d.1 += f;
            if d.1.is_sign_negative() {
                d.1 = 0.0;
            }

            let f: u32 = rng.random_range(..5);
            if rng.random_bool(0.5) {
                duration.get_mut(k).unwrap().0 += f;
            } else {
                duration.get_mut(k).unwrap().0 = duration.get_mut(k).unwrap().0.saturating_sub(f);
            }
        }
        // Jitter the jitter the frequency counts
        for k in freq {
            let f: u32 = rng.random_range(..5);
            if rng.random_bool(0.5) {
                *frequency.get_mut(k).unwrap() += f;
            } else {
                *frequency.get_mut(k).unwrap() = frequency.get_mut(k).unwrap().saturating_sub(f);
            }
        }

        let reli = OutputData {
            datetime: String::from("TEST FILE"),
            session: session_data.clone(),
            session_duration: session_time,
            frequency: frequency.clone(),
            duration: duration.clone(),
            timeline: timeline.clone(),
            ksf: ksf.clone(),
            client_name: client.name.clone(),
            client_id: client.id.clone(),
            case_manager: client.case_manager.clone(),
            primary_therapist: client.primary_therapist.clone(),
            session_number: session,
            days_since_admissions: client.days_since_admission().unwrap_or(-99999),
            location: client.location.clone(),
        };

        let pfile = File::create(&format!(
            "{}{}_{}.txt",
            client.initials(),
            prim.session_number,
            prim.session.data_collection_type.abbrev()
        ))
        .unwrap();
        let mut writer = std::io::BufWriter::new(pfile);
        std::io::Write::write_all(&mut writer, prim.to_json().unwrap().as_bytes()).unwrap();
        std::io::Write::flush(&mut writer).unwrap();

        let rfile = File::create(&format!(
            "{}{}_{}.txt",
            client.initials(),
            reli.session_number,
            reli.session.data_collection_type.abbrev()
        ))
        .unwrap();
        let mut writer = std::io::BufWriter::new(rfile);
        std::io::Write::write_all(&mut writer, reli.to_json().unwrap().as_bytes()).unwrap();
        std::io::Write::flush(&mut writer).unwrap();
    }
}
