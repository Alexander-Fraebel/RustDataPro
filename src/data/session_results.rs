use crate::data::ClientAndSessionInfo;
use crate::data::{Ksf, SessionInfo, timeline::Timeline};
use crate::utils::{date_time_string, rounded_f32};
use anyhow::Context;
use anyhow::Result;
use chrono::Local;
use egui::Key;
use indexmap::IndexMap;
use rand::SeedableRng;
use rand_distr::Distribution;
use rust_xlsxwriter::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Output of a single session. Includes the Client and Session data along with the recorded keypresses and times, and the KSF to translate those.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionResults {
    pub session_start_time: String,
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
    /// Is this file the complement of another file? Do they have the same Ksf, Client ID, Session Number, Assessment, and Condition?
    pub fn complement_of(&self, other: &Self) -> bool {
        self.ksf == other.ksf
            && self.client_id == other.client_id
            && self.session_number == other.session_number
            && self.session_data.chosen_assessment == other.session_data.chosen_assessment
            && self.session_data.chosen_condition == other.session_data.chosen_condition
    }

    pub fn file_name_stem(&self) -> String {
        format!(
            "{}-{}-{}_S{:>03}-{}", // three digit aligned session number is sufficient for six sessions every working day for about seven months
            self.session_data
                .chosen_assessment
                .chars()
                .filter(|c| !c.is_whitespace()) // remove whitespace from file name
                .take(5) // maximum length for better alignment
                .collect::<String>(),
            self.session_data
                .chosen_condition
                .chars()
                .filter(|c| !c.is_whitespace()) // remove whitespace from file name
                .take(5) // maximum length for better alignment
                .collect::<String>(),
            self.client_id,
            self.session_number,
            self.session_data.data_collection_type.abbrev(),
        )
    }

    pub fn xlsx_file_name(&self) -> String {
        let mut stem = self.file_name_stem();
        stem.push_str(".xlsx");
        stem
    }

    pub fn xlsx_timeline_name(&self) -> String {
        let mut stem = self.file_name_stem();
        stem.push_str("_timeline.xlsx");
        stem
    }

    pub fn json_file_name(&self) -> String {
        let mut stem = self.file_name_stem();
        stem.push_str(".json");
        stem
    }

    pub fn to_xlsx(&self) -> Result<Workbook> {
        let mut workbook = Workbook::new();

        /////////////////////////
        // Resuable Formatting //
        /////////////////////////
        let bold = Format::new().set_bold();
        let centered_bold = Format::new().set_align(FormatAlign::Center).set_bold();
        let percentage = Format::new().set_num_format("0.0%");
        let one_decimal = Format::new().set_num_format("0.0");
        let centered_underlined = Format::new()
            .set_align(FormatAlign::Center)
            .set_underline(FormatUnderline::Single);

        ///////////////////////////
        // Information Worksheet //
        ///////////////////////////

        let information = workbook.add_worksheet();
        information.set_name("Information")?;
        information.set_column_width_pixels(column_name_to_number("A"), 90)?;
        information.set_column_width_pixels(column_name_to_number("B"), 90)?;
        information.set_column_width_pixels(column_name_to_number("C"), 20)?;
        information.set_column_width_pixels(column_name_to_number("D"), 90)?;
        information.set_column_width_pixels(column_name_to_number("E"), 90)?;
        information.set_column_width_pixels(column_name_to_number("F"), 20)?;
        information.set_column_width_pixels(column_name_to_number("G"), 90)?;
        information.set_column_width_pixels(column_name_to_number("H"), 90)?;

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

        information.write_with_format(row, col, "Active Time:", &bold)?;
        information.write_with_format(row, col + 1, self.active_time, &one_decimal)?;
        row += 1;

        information.write_with_format(row, col, "Paused Time:", &bold)?;
        information.write_with_format(row, col + 1, self.pause_time, &one_decimal)?;
        row += 1;

        information.write_with_format(row, col, "Total Time:", &bold)?;
        information.write_with_format(row, col + 1, self.total_time, &one_decimal)?;

        row = 1;
        col += 3;
        information.write_with_format(row, col, "Assessment:", &bold)?;
        information.write(row, col + 1, &self.session_data.chosen_assessment)?;
        row += 1;

        information.write_with_format(row, col, "Condition:", &bold)?;
        information.write(row, col + 1, &self.session_data.chosen_condition)?;
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
        information.write_with_format(row, col, "KSF Name:", &bold)?;
        information.write(row, col + 1, &self.session_data.chosen_ksf_name)?;
        row += 2;

        information.write_with_format(row, col, "Key", &centered_underlined)?;
        information.write_with_format(row, col + 1, "Description", &centered_underlined)?;
        row += 1;
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

        information.merge_range(1, column_name_to_number("J"), 6, column_name_to_number("L"), "The matching .json file contains more detailed information including a timeline of keypresses. It can be read in Notepad.", &Format::new().set_text_wrap())?;

        ////////////////////////////
        // Data Summary Worksheet //
        ////////////////////////////
        let summary = workbook.add_worksheet();
        summary.set_name("Data Summary")?;

        ///////////////
        // Frequency //
        ///////////////
        let frow = 1;
        summary.write_with_format(frow, 1, "Frequency Data", &bold)?;
        summary.write_with_format(frow + 2, 1, "Count", &bold)?;
        summary.write_with_format(frow + 3, 1, "RPM", &bold)?;
        let mut col = 2;
        let mins_active = self.active_time / 60.0;
        for (key, count) in self.frequency_data.iter() {
            summary.write_with_format(frow + 1, col, key.symbol_or_name(), &centered_bold)?;
            summary.write(frow + 2, col, *count)?;
            summary.write_with_format(
                frow + 3,
                col,
                (*count as f32) / mins_active,
                &one_decimal,
            )?;
            col += 1;
        }

        //////////////
        // Duration //
        //////////////
        let d_row = 6;
        summary.write_with_format(d_row, 1, "Duration Data", &bold)?;
        summary.write_with_format(d_row + 2, 1, "Duration", &bold)?;
        summary.write_with_format(d_row + 3, 1, "Bouts", &bold)?;
        summary.write_with_format(d_row + 4, 1, "% of AT", &bold)?;

        let mut col = 2;
        let at = self.active_time;

        for (key, (count, duration)) in self.duration_data.iter() {
            summary.write_with_format(d_row + 1, col, key.symbol_or_name(), &centered_bold)?;
            summary.write_with_format(d_row + 2, col, *duration, &one_decimal)?;
            summary.write(d_row + 3, col, *count)?;
            summary.write_with_format(d_row + 4, col, duration / at, &percentage)?;
            col += 1;
        }

        summary.write_with_format(d_row + 1, col, "AT", &centered_bold)?;
        summary.write_with_format(d_row + 2, col, at, &one_decimal)?;
        summary.write(d_row + 3, col, 1)?;
        summary.write_with_format(d_row + 4, col, 1.0, &percentage)?;

        Ok(workbook)
    }

    crate::to_and_from_json!(
        self,
        "unable to make SessionResults from file",
        "unable to convert SessionResults to json"
    );

    pub fn simulate_session_results(
        path: std::path::PathBuf,
        data: &ClientAndSessionInfo,
        session_number: u32,
        include_reli: bool,
        seed: u64,
    ) {
        use rand::{RngExt, make_rng, rngs::StdRng, seq::IndexedRandom};
        use std::fs::File;

        let client = &data.client;

        let mut session_data = SessionInfo::default();
        session_data.chosen_assessment = data.chosen_assessment_name().clone();
        session_data.chosen_condition = data.chosen_condition_name().clone();
        session_data.data_collection_type = crate::data::DataCollectionType::Primary;
        session_data.chosen_ksf_name = data.chosen_ksf_name().clone();
        session_data.therapist = String::from("THERAPIST");
        session_data.data_collector = String::from("DATA COLLECTOR");
        session_data.max_active_time = data.session.max_active_time;
        session_data.max_total_time = data.session.max_total_time;

        let ksf = data.chosen_ksf().unwrap();

        // Fixed seed for consitency across trials
        let mut seeded_rng: StdRng = StdRng::seed_from_u64(seed);
        // System seeded so all trials are different
        let mut rng: StdRng = make_rng();

        let mut frequency: IndexMap<Key, u32> = IndexMap::new();
        let mut fkeys = Vec::new();
        let mut f_rates = Vec::new();
        let (freq, dura) = ksf.freq_and_dura_keys();
        for k in freq {
            frequency.insert(*k, 0);
            f_rates.push((*k, seeded_rng.random::<f32>()));
            fkeys.push(*k);
        }
        let mut duration: IndexMap<Key, (u32, f32)> = IndexMap::new();
        let mut dkeys = Vec::new();
        let mut d_rates = Vec::new();
        let mut d_keys_used = HashMap::new();
        for k in dura {
            duration.insert(*k, (0, 0.0));
            d_rates.push((*k, seeded_rng.random::<f32>()));
            dkeys.push(*k);
        }

        // Reciprocal of the average time between keypresses
        let lambda: f32 = 1.0 / ((rng.random::<f32>() + 1.0) * 3.0);
        let mut key_press_rng: StdRng = make_rng();
        let mut next_press = rand_distr::Exp::new(lambda)
            .unwrap()
            .sample_iter(&mut key_press_rng);

        let mut timeline = Timeline::default();
        let mut session_time = 0.0;
        let max_session_time = 1000.0;
        timeline.push((Key::Tab, rounded_f32(session_time)));
        for _ in 0..270 {
            let gap = next_press
                .next()
                .expect("error producing next value for Exp distribution");
            if session_time + gap > max_session_time {
                break;
            }
            session_time = session_time + gap;

            let t = rounded_f32(session_time);
            if rng.random_bool(0.7) {
                let k = f_rates.choose_weighted(&mut rng, |item| item.1).unwrap().0;
                *frequency.get_mut(&k).unwrap() += 1;
                timeline.push((k, t));
            } else {
                let k = d_rates.choose_weighted(&mut rng, |item| item.1).unwrap().0;
                timeline.push((k, t));
                if let Some(last_time) = d_keys_used.remove(&k) {
                    duration.get_mut(&k).unwrap().0 += 1;
                    duration.get_mut(&k).unwrap().1 += t - last_time;
                } else {
                    d_keys_used.insert(k, t);
                }
                // Invert the rate at which the key appears so that high probability keys don't contantly go on and off
                d_rates.iter_mut().for_each(|(key, r)| {
                    if *key == k {
                        *r = 1.0 - *r;
                    }
                });
            };
        }
        for (k, last_time) in d_keys_used.iter() {
            duration.get_mut(k).unwrap().0 += 1;
            duration.get_mut(k).unwrap().1 += rounded_f32(session_time - last_time);
        }
        timeline.push((Key::Escape, rounded_f32(session_time)));

        let prim = SessionResults {
            session_start_time: format!("SIMULATED ON {}", date_time_string(&Local::now())),
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
            session_number: session_number,
            days_since_admission: client.days_since_admission().unwrap_or(-99999),
            location: client.location.clone(),
        };

        let pfile = File::create(path.join(prim.json_file_name())).unwrap();
        let mut writer = std::io::BufWriter::new(pfile);
        std::io::Write::write_all(&mut writer, prim.to_json().unwrap().as_bytes()).unwrap();
        std::io::Write::flush(&mut writer).unwrap();

        let mut workbook = prim.to_xlsx().unwrap();
        workbook.save(path.join(prim.xlsx_file_name())).unwrap();

        // Create reli data that is slightly different
        if include_reli {
            session_data.data_collection_type = crate::data::DataCollectionType::Reliability;

            // Jitter the timing for the keypresses
            let mut error_val = rand_distr::Normal::new(0.0, 0.5)
                .unwrap()
                .sample_iter(&mut rng);
            for (_k, t) in timeline.iter_mut() {
                *t += error_val
                    .next()
                    .expect("error producing next value for Normal distribution");
            }

            for _ in 0..40 {
                // Introduce omissions
                if rng.random_bool(0.2) {
                    let r = 0..timeline.len();
                    timeline.remove(rng.random_range(r));
                }
            }

            // Recalculate the summary data from the timeline
            frequency.iter_mut().for_each(|x| *x.1 = 0);
            duration.iter_mut().for_each(|x| *x.1 = (0, 0.0));
            d_keys_used.clear();
            for (key, time) in timeline.iter() {
                if fkeys.contains(key) {
                    *frequency.get_mut(key).unwrap() += 1
                } else if dkeys.contains(key) {
                    if let Some(last_time) = d_keys_used.remove(key) {
                        duration.get_mut(key).unwrap().0 += 1;
                        duration.get_mut(key).unwrap().1 += *time - last_time;
                    } else {
                        d_keys_used.insert(*key, *time);
                    }
                }
            }
            for (k, last_time) in d_keys_used.iter() {
                duration.get_mut(k).unwrap().0 += 1;
                duration.get_mut(k).unwrap().1 += rounded_f32(session_time - last_time);
            }

            let reli = SessionResults {
                session_start_time: format!("SIMULATED ON {}", date_time_string(&Local::now())),
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
                session_number: session_number,
                days_since_admission: client.days_since_admission().unwrap_or(i32::MIN),
                location: client.location.clone(),
            };

            let rfile = File::create(path.join(reli.json_file_name())).unwrap();
            let mut writer = std::io::BufWriter::new(rfile);
            std::io::Write::write_all(&mut writer, reli.to_json().unwrap().as_bytes()).unwrap();
            std::io::Write::flush(&mut writer).unwrap();

            let mut workbook = reli.to_xlsx().unwrap();
            workbook.save(path.join(reli.xlsx_file_name())).unwrap();
        }
    }
}
