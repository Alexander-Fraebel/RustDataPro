use crate::data::{ClientAndSessionInfo, DataCollectionType, KSF_REGEX, TEMP_TIMELINE};
use crate::data::{Ksf, SessionInfo, timeline::Timeline};
use crate::utils::{date_time_string, rounded_f32};
use anyhow::Result;
use anyhow::{Context, anyhow};
use chrono::Local;
use egui::Key;
use indexmap::IndexMap;
use itertools::Itertools;
use rand::SeedableRng;
use rand_distr::Distribution;
use rust_xlsxwriter::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Output of a single session. Includes the Client and Session data along with the recorded keypresses and times, and the KSF to translate those.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SessionSummary {
    // Client information
    pub client_name: String,
    pub client_id: String,
    pub case_manager: String,
    pub primary_therapist: String,
    pub session_number: u32,
    pub days_since_admission: i32,
    pub location: String,
    // Summary information
    pub total_time: f32,
    pub active_time: f32,
    pub pause_time: f32,
    pub frequency_data: IndexMap<Key, u32>,
    pub duration_data: IndexMap<Key, (u32, f32)>,
    // Notes
    pub notes: String,
    // Session information
    pub session_start_time: String,
    pub assessment: String,
    pub condition: String,
    pub therapist: String,
    pub data_collector: String,
    pub data_collection_type: DataCollectionType,
    pub ksf: Ksf,
    pub timeline: Timeline,
}

impl SessionSummary {
    /// Rebuild a SessionSummary from a temp file.
    pub fn rebuild(text: &str) -> Result<SessionSummary> {
        let mut out = Self::default();

        out.notes = String::from("rebuilt from temp file");

        let mut lines = text.lines();
        out.client_name = lines.next().ok_or(anyhow!("missing line"))?.try_into()?;
        out.client_id = lines.next().ok_or(anyhow!("missing line"))?.try_into()?;
        out.case_manager = lines.next().ok_or(anyhow!("missing line"))?.try_into()?;
        out.primary_therapist = lines.next().ok_or(anyhow!("missing line"))?.try_into()?;
        out.session_number = lines.next().ok_or(anyhow!("missing line"))?.parse()?;
        out.days_since_admission = lines.next().ok_or(anyhow!("missing line"))?.parse()?;
        out.location = lines.next().ok_or(anyhow!("missing line"))?.try_into()?;
        out.session_start_time = lines.next().ok_or(anyhow!("missing line"))?.try_into()?;
        out.assessment = lines.next().ok_or(anyhow!("missing line"))?.try_into()?;
        out.condition = lines.next().ok_or(anyhow!("missing line"))?.try_into()?;
        out.therapist = lines.next().ok_or(anyhow!("missing line"))?.try_into()?;
        out.data_collector = lines.next().ok_or(anyhow!("missing line"))?.try_into()?;
        out.data_collection_type = lines.next().ok_or(anyhow!("missing line"))?.try_into()?;

        out.ksf = Ksf::from_json(
            KSF_REGEX
                .find(text)
                .context("unable to find valid KSF data")?
                .as_str(),
        )?;

        out.timeline = Timeline::from_text(
            TEMP_TIMELINE
                .find(text)
                .context("unable to find a valid Timeline data")?
                .as_str(),
        )?;

        out.frequency_data = out.timeline.calculate_frequency_data(&out.ksf);
        out.duration_data = out.timeline.calculate_duration_data(&out.ksf);

        (out.total_time, out.active_time, out.pause_time) = out.timeline.calculate_times();

        Ok(out)
    }

    /// Merge any number of SessionSummary files. Total time, active time, pause time, frequency data, duration data, and timeline are merged.
    /// Everything else is copied from the first file.
    pub fn merge(results: &[SessionSummary]) -> Result<SessionSummary> {
        let mut out = Self::default();
        if results.is_empty() {
            return Ok(out);
        }
        // Clone from the first result
        out.session_start_time = results[0].session_start_time.clone();
        out.client_name = results[0].client_name.clone();
        out.client_id = results[0].client_id.clone();
        out.case_manager = results[0].case_manager.clone();
        out.primary_therapist = results[0].primary_therapist.clone();
        out.session_number = results[0].session_number;
        out.days_since_admission = results[0].days_since_admission;
        out.location = results[0].location.clone();
        out.ksf = results[0].ksf.clone();

        // Combine from all of them
        for result in results.iter() {
            if result.client_id != out.client_id
                || result.client_name != out.client_name
                || result.ksf != out.ksf
                || result.assessment != out.assessment
                || result.condition != out.condition
            {
                return Err(anyhow::anyhow!(
                    "when merging files that must all have the same client_id, client_name, assessment, condition, and ksf"
                ));
            }

            out.timeline.merge(&result.timeline);

            out.total_time += result.total_time;
            out.active_time += result.active_time;
            out.pause_time += result.pause_time;

            for (key, count) in result.frequency_data.iter() {
                out.frequency_data
                    .entry(*key)
                    .and_modify(|n| *n += count)
                    .or_insert(*count);
            }
            for (key, (count, time)) in result.duration_data.iter() {
                out.duration_data
                    .entry(*key)
                    .and_modify(|(c, t)| {
                        *c += count;
                        *t += time;
                    })
                    .or_insert((*count, *time));
            }
        }
        out.notes = format!(
            "Merged from sessions: {}",
            results.iter().map(|s| s.session_number).join(", ")
        );
        Ok(out)
    }

    /// Is this file the complement of another file? Do they have the same Ksf, Client ID, Session Number, Assessment, and Condition?
    pub fn complement_of(&self, other: &Self) -> Result<()> {
        let mut err = Vec::new();
        if self.ksf != other.ksf {
            err.push("KSFs do not match");
        }
        if self.client_id != other.client_id {
            err.push("Client IDs do not match");
        }
        if self.session_number != other.session_number {
            err.push("Session Numbers do not match");
        }
        if self.assessment != other.assessment {
            err.push("Assessments do not match");
        }
        if self.condition != other.condition {
            err.push("Conditions do not match");
        }

        if err.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(err.join("\n")))
        }
    }

    pub fn file_name_stem(&self) -> String {
        format!(
            "{}-{}-{}_S{:>03}-{}", // three digit aligned session number is sufficient for six sessions every working day for about seven months
            self.assessment
                .chars()
                .filter(|c| !c.is_whitespace()) // remove whitespace from file name
                .take(5) // maximum length for better alignment
                .collect::<String>(),
            self.condition
                .chars()
                .filter(|c| !c.is_whitespace()) // remove whitespace from file name
                .take(5) // maximum length for better alignment
                .collect::<String>(),
            self.client_id,
            self.session_number,
            self.data_collection_type.abbrev(),
        )
    }

    pub fn xlsx_file_name(&self) -> String {
        let mut stem = self.file_name_stem();
        stem.push_str(".xlsx");
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
        information.write(row, col + 1, &self.assessment)?;
        row += 1;

        information.write_with_format(row, col, "Condition:", &bold)?;
        information.write(row, col + 1, &self.condition)?;
        row += 1;

        information.write_with_format(row, col, "Data Type:", &bold)?;
        information.write(row, col + 1, self.data_collection_type.to_string())?;

        ///////////////////////
        // Summarize the KSF //
        ///////////////////////
        row = 1;
        col += 3;
        // information.write_with_format(row, col, "KSF Name:", &bold)?;
        // information.write(row, col + 1, &self.session_data.chosen_ksf_name)?;
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

    crate::to_and_from_json!(self, "SessionSummary");

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

        let prim = SessionSummary {
            session_start_time: format!("{}", date_time_string(&Local::now())),
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
            notes: String::from("SIMULATED"),
            assessment: session_data.chosen_assessment.clone(),
            condition: session_data.chosen_condition.clone(),
            therapist: session_data.therapist.clone(),
            data_collector: session_data.data_collector.clone(),
            data_collection_type: session_data.data_collection_type.clone(),
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

            let reli = SessionSummary {
                session_start_time: format!(" {}", date_time_string(&Local::now())),
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
                assessment: session_data.chosen_assessment.clone(),
                condition: session_data.chosen_condition.clone(),
                therapist: session_data.therapist.clone(),
                data_collector: session_data.data_collector.clone(),
                data_collection_type: session_data.data_collection_type.clone(),
                notes: String::from("SIMULATED"),
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
