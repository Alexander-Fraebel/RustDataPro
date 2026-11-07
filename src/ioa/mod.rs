pub mod calculations;
pub mod excel_output;
pub mod ioa_page;
pub use ioa_page::IoaPage;

use crate::data::{DataCollectionType, SessionResults};
use anyhow::{Context, Result};
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

pub fn quick_file_name(pathbuf: &std::path::Path) -> std::borrow::Cow<'_, str> {
    pathbuf
        .file_name()
        .unwrap_or(&std::ffi::OsStr::new("INVALID FILE NAME"))
        .to_string_lossy()
}

#[derive(Default)]
pub struct IoaDataMap(
    std::collections::BTreeMap<
        u32,
        (
            Option<(SessionResults, PathBuf)>,
            Option<(SessionResults, PathBuf)>,
        ),
    >,
);

impl Deref for IoaDataMap {
    type Target = std::collections::BTreeMap<
        u32,
        (
            Option<(SessionResults, PathBuf)>,
            Option<(SessionResults, PathBuf)>,
        ),
    >;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for IoaDataMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IoaDataMap {
    fn insert(&mut self, results: SessionResults, buf: PathBuf) -> Result<()> {
        match results.session_data.data_collection_type {
            DataCollectionType::Primary => {
                self.entry(results.session_number)
                    .and_modify(|e| e.0 = Some((results.clone(), buf.clone())))
                    .or_insert((Some((results.clone(), buf.clone())), None));
            }
            DataCollectionType::Reliability => {
                self.entry(results.session_number)
                    .and_modify(|e| e.1 = Some((results.clone(), buf.clone())))
                    .or_insert((None, Some((results.clone(), buf.clone()))));
            }
        }
        Ok(())
    }

    pub fn validated(
        &self,
    ) -> Result<Vec<(&(SessionResults, PathBuf), &(SessionResults, PathBuf))>> {
        if self.is_empty() {
            return Err(anyhow::anyhow!("no files have been selected"));
        }

        let mut out = Vec::new();

        for (n, (prim, reli)) in self.0.iter() {
            let p = prim
                .as_ref()
                .context(format!("no primary data for S{n:>03}"))?;
            let r = reli
                .as_ref()
                .context(format!("no reliability data for S{n:>03}"))?;

            // if !p.0.complement_of(&r.0) {
            //     return Err(anyhow::anyhow!(
            //         "the primary and reliability files for S{n:>03} cannot be compared\ncheck that the Assessment, Condition, Client ID, KSF, and Session Number all match"
            //     ));
            // }

            out.push((p, r));
        }

        let ksf = &out[0].0.0.ksf;
        let client_id = &out[0].0.0.client_id;
        for ((p, _), (r, _)) in out.iter() {
            if &p.ksf != ksf {
                return Err(anyhow::anyhow!(
                    "the KSF for the primary file for S{:>03} does not match the KSF for the first file\nall files must have the same KSF",
                    p.session_number
                ));
            }
            if &r.ksf != ksf {
                return Err(anyhow::anyhow!(
                    "the KSF for the reliability file for S{:>03} does not match the KSF for the first file\nall files must have the same KSF",
                    r.session_number
                ));
            }
            if &p.client_id != client_id {
                return Err(anyhow::anyhow!(
                    "the Client ID for the primary file for S{:>03} does not match the Client ID for the first file\nall files must have the same Client ID",
                    p.session_number
                ));
            }
            if &r.client_id != client_id {
                return Err(anyhow::anyhow!(
                    "the Client ID for the reliability file for S{:>03} does not match the Client ID for the first file\nall files must have the same Client ID",
                    r.session_number
                ));
            }
        }

        Ok(out)
    }
}
