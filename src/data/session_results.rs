use crate::data::{Ksf, SessionInfo, timeline::Timeline};
use crate::utils::rounded_f32;
use anyhow::Context;
use anyhow::Result;
use egui::Key;
use indexmap::IndexMap;
use itertools::Itertools;
use rust_xlsxwriter::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Output of a single session. Includes the Client and Session data along with the recorded keypresses and times, and the KSF to translate those.
#[derive(Serialize, Deserialize, Clone)]
pub struct SessionResults {
    pub datetime: String,
    pub client_name: String,
    pub client_id: String,
    pub case_manager: String,
    pub primary_therapist: String,
    pub session_number: u32,
    pub days_since_admission: i32,
    pub location: String,
    pub session_data: SessionInfo,
    pub total_time: f32,
    pub active_time: f32,
    pub pause_time: f32,
    pub frequency_data: IndexMap<Key, u32>,
    pub duration_data: IndexMap<Key, (u32, f32)>,
    pub timeline: Timeline,
    pub ksf: Ksf,
}

impl SessionResults {
    pub fn txt_file_name(&self) -> String {
        format!(
            "{}-{}_{:>03}{}.txt", // always format the session number to three digits to help sorting and alignment
            self.session_data.chosen_assessment,
            self.session_data.chosen_condition,
            self.session_number,
            self.session_data.data_collection_type.abbrev()
        )
    }

    pub fn xlsx_file_name(&self) -> String {
        format!(
            "{}-{}_{:>03}{}.xlsx", // always format the session number to three digits to help sorting and alignment
            self.session_data.chosen_assessment,
            self.session_data.chosen_condition,
            self.session_number,
            self.session_data.data_collection_type.abbrev()
        )
    }

    pub fn xlsx_timeline_name(&self) -> String {
        format!(
            "{}-{}_{:>03}{}_timeline.xlsx", // always format the session number to three digits to help sorting and alignment
            self.session_data.chosen_assessment,
            self.session_data.chosen_condition,
            self.session_number,
            self.session_data.data_collection_type.abbrev()
        )
    }

    pub fn timeline_to_xlsx(&self) -> Result<Workbook> {
        let mut workbook = Workbook::new();
        let timeline = &self.timeline;

        let graph = workbook.add_worksheet();
        let mut height_map = std::collections::HashMap::new();
        height_map.insert(&Key::Tab, (0, 1));
        graph.write(0, 0, "Time Keys")?;
        let mut ctr = 2;
        for k in timeline.iter().map(|(k, _)| k).unique() {
            if ![Key::Escape, Key::Space, Key::Tab].contains(k) {
                height_map.insert(k, (ctr / 2, 1));
                graph.write(ctr, 0, k.symbol_or_name())?;
                ctr += 2;
            }
        }

        for (key, time) in timeline.iter() {
            let val = if [Key::Escape, Key::Space, Key::Tab].contains(key) {
                height_map.get_mut(&Key::Tab).unwrap()
            } else {
                height_map.get_mut(key).unwrap()
            };
            graph.write(val.0, val.1, val.0 as f64)?;
            graph.write(val.0 + 1, val.1, *time)?;
            val.1 += 1;
        }

        let mut chart = Chart::new_scatter();
        chart.y_axis().set_hidden(true); // carries no information
        for row in 0..height_map.len() {
            let row = (row as u32) * 2;
            let next_row = row + 1;
            chart
                .add_series()
                .set_name(("Sheet1", row * 2, 0))
                .set_values(("Sheet1", row, 1, row, 10000))
                .set_categories(("Sheet1", next_row, 1, next_row, 10000));
        }
        graph.insert_chart(1 + height_map.len() as u32, 0, &chart)?;

        Ok(workbook)
    }

    pub fn to_xlsx(&self) -> Result<Workbook> {
        let mut workbook = Workbook::new();

        /////////////////////////
        // Resuable Formatting //
        /////////////////////////
        let bold = Format::new().set_bold();
        let centered_bold = Format::new().set_align(FormatAlign::Center).set_bold();

        ///////////////////////////
        // Information Worksheet //
        ///////////////////////////

        let information = workbook.add_worksheet();
        information.set_name("Information")?;
        information.set_column_width_pixels(0, 70)?;
        information.set_column_width_pixels(3, 90)?;
        information.set_column_width_pixels(3, 90)?;

        ///////////////////////////////
        // Basic Session Information //
        ///////////////////////////////
        let mut row = 1;
        let mut col = 0;
        information.write_with_format(row, col, "Session:", &bold)?;
        information.write(row, col + 1, self.session_number)?;
        row += 1;

        information.write_with_format(row, col, "DOA:", &bold)?;
        information.write(row, col + 1, self.days_since_admission)?;
        row += 1;

        information.write_with_format(row, col, "Location:", &bold)?;
        information.write(row, col + 1, &self.location)?;
        row += 1;

        information.write_with_format(row, col, "Duration:", &bold)?;
        information.write(row, col + 1, self.total_time)?;

        row = 1;
        col += 3;
        information.write_with_format(row, col, "Assessment:", &bold)?;
        information.write(row, col + 1, &self.session_data.chosen_assessment)?;
        row += 1;

        information.write_with_format(row, col, "Condition:", &bold)?;
        information.write(row, col + 1, &self.session_data.chosen_condition)?;
        row += 1;

        information.write_with_format(row, col, "KSF Name:", &bold)?;
        information.write(row, col + 1, &self.session_data.chosen_ksf_name)?;
        row += 1;

        information.write_with_format(row, col, "Data Type:", &bold)?;
        information.write(
            row,
            col + 1,
            self.session_data.data_collection_type.to_string(),
        )?;

        ///////////////////////
        // Summarize the KSF //
        ///////////////////////
        row = 1;
        col += 3;
        for (key, desc) in self.ksf.freq.iter() {
            information.write_with_format(row, col, key.symbol_or_name(), &centered_bold)?;
            information.write(row, col + 1, desc)?;
            row += 1;
        }
        for (key, desc) in self.ksf.dura.iter() {
            information.write_with_format(row, col, key.symbol_or_name(), &centered_bold)?;
            information.write(row, col + 1, desc)?;
            row += 1;
        }

        ////////////////////////////
        // Data Summary Worksheet //
        ////////////////////////////
        let summary = workbook.add_worksheet();
        summary.set_name("Data Summary")?;

        ///////////////
        // Frequency //
        ///////////////
        summary.write_with_format(1, 1, "Frequency Keys", &bold)?;
        summary.write_with_format(3, 1, "Count", &bold)?;
        let mut col = 2;
        for (key, count) in self.frequency_data.iter() {
            summary.write_with_format(2, col, key.symbol_or_name(), &centered_bold)?;
            summary.write(3, col, *count)?;
            col += 1;
        }

        //////////////
        // Duration //
        //////////////
        summary.write_with_format(5, 1, "Duration Keys", &bold)?;
        summary.write_with_format(7, 1, "Duration", &bold)?;
        summary.write_with_format(8, 1, "Bouts", &bold)?;
        summary.write_with_format(9, 1, "% of TT", &bold)?;
        summary.write_with_format(10, 1, "% of AT", &bold)?;
        let tt = self.total_time;
        let at = self.active_time;
        let mut col = 2;
        for (key, (count, duration)) in self.duration_data.iter() {
            summary.write_with_format(6, col, key.symbol_or_name(), &centered_bold)?;
            summary.write(7, col, *duration)?;
            summary.write(8, col, *count)?;
            summary.write(9, col, rounded_f32(duration / tt))?;
            summary.write(10, col, rounded_f32(duration / at))?;
            col += 1;
        }
        summary.write_with_format(6, col, "TT", &centered_bold)?;
        summary.insert_note(6, col, &Note::new("Total Time"))?;
        summary.write(7, col, tt)?;
        summary.write(8, col, 0)?;
        summary.write(9, col, 1)?;
        summary.write(10, col, rounded_f32(tt / at))?;
        col += 1;

        summary.write_with_format(6, col, "AT", &centered_bold)?;
        summary.insert_note(6, col, &Note::new("Active Time"))?;
        summary.write(7, col, at)?;
        summary.write(8, col, 0)?;
        summary.write(9, col, rounded_f32(at / tt))?;
        summary.write(10, col, 1)?;

        Ok(workbook)
    }

    crate::to_and_from_json!(
        self,
        "unable to make SessionResults from file",
        "unable to convert SessionResults to json"
    );
}

#[test]
fn create_test_data() {
    use crate::{data::ClientInfo, utils::rounded_f32};
    use egui::Key;
    use rand::{RngExt, make_rng, rngs::StdRng, seq::IndexedRandom};
    use std::fs::File;

    let mut rng: StdRng = make_rng();

    let mut client = ClientInfo::default();
    client.id = format!("{:0<10}", rng.random_range(1000000000_i64..=9999999999));

    for session in 1..2 {
        // client.current_session = session;
        let mut session_data = SessionInfo::default();
        session_data.chosen_assessment = String::from("ASSESS");
        session_data.chosen_condition = String::from("COND");
        session_data.data_collection_type = crate::data::DataCollectionType::Primary;

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

        let prim = SessionResults {
            datetime: String::from("TEST FILE"),
            session_data: session_data.clone(),
            total_time: rounded_f32(session_time),
            pause_time: 0.0,
            active_time: rounded_f32(session_time),
            frequency_data: frequency.clone(),
            duration_data: duration.clone(),
            timeline: timeline.clone(),
            ksf: ksf.clone(),
            client_name: client.name.clone(),
            client_id: client.id.clone(),
            case_manager: client.case_manager.clone(),
            primary_therapist: client.primary_therapist.clone(),
            session_number: session,
            days_since_admission: client.days_since_admission().unwrap_or(-99999),
            location: client.location.clone(),
        };

        // Jitter the timing for the keypresses
        session_data.data_collection_type = crate::data::DataCollectionType::Reliability;
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

        let reli = SessionResults {
            datetime: String::from("TEST FILE"),
            session_data: session_data.clone(),
            total_time: session_time,
            pause_time: 0.0,
            active_time: session_time,
            frequency_data: frequency.clone(),
            duration_data: duration.clone(),
            timeline: timeline.clone(),
            ksf: ksf.clone(),
            client_name: client.name.clone(),
            client_id: client.id.clone(),
            case_manager: client.case_manager.clone(),
            primary_therapist: client.primary_therapist.clone(),
            session_number: session,
            days_since_admission: client.days_since_admission().unwrap_or(i32::MIN),
            location: client.location.clone(),
        };

        let pfile = File::create(&prim.txt_file_name()).unwrap();
        let mut writer = std::io::BufWriter::new(pfile);
        std::io::Write::write_all(&mut writer, prim.to_json().unwrap().as_bytes()).unwrap();
        std::io::Write::flush(&mut writer).unwrap();

        let mut workbook = prim.to_xlsx().unwrap();
        workbook.save(prim.xlsx_file_name()).unwrap();

        let rfile = File::create(&reli.txt_file_name()).unwrap();
        let mut writer = std::io::BufWriter::new(rfile);
        std::io::Write::write_all(&mut writer, reli.to_json().unwrap().as_bytes()).unwrap();
        std::io::Write::flush(&mut writer).unwrap();

        let mut workbook = reli.to_xlsx().unwrap();
        workbook.save(reli.xlsx_file_name()).unwrap();
    }
}
