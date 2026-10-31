use crate::{data::SessionResults, ioa::quick_file_name};
use anyhow::Result;
use std::path::PathBuf;

/// Ensure that:
///     files have been loaded
///     they all use the same KSFs
///     they all have the same client ID
///     each pair of files is for a matching session.
/// Details like date are not checked as reli may be taken via video
pub fn validate_files(
    prim_data: &Vec<(SessionResults, PathBuf)>,
    reli_data: &Vec<(SessionResults, PathBuf)>,
) -> Result<()> {
    // Are there any files?
    match (prim_data.is_empty(), reli_data.is_empty()) {
        (true, true) => return Err(anyhow::anyhow!("no data files selected")),
        (true, false) => return Err(anyhow::anyhow!("no primary data files")),
        (false, true) => return Err(anyhow::anyhow!("no reliability data files")),
        _ => (),
    }

    // Is there a reliability file to go with each primary file?
    if prim_data.len() != reli_data.len() {
        return Err(anyhow::anyhow!(
            "unequal number of primary and reliability files"
        ));
    }

    // Check that all files use the same KSF and client ID
    let ksf = &prim_data[0].0.ksf;
    let id = &prim_data[0].0.client_id;
    let comparison_path = &prim_data[0].1;
    for (o, path) in prim_data.iter().chain(reli_data.iter()).skip(1) {
        if &o.ksf != ksf {
            return Err(anyhow::anyhow!(
                "all files must have the same KSF as {}\nfile {} does not",
                quick_file_name(&comparison_path),
                quick_file_name(&path)
            ));
        }
        if &o.client_id != id {
            return Err(anyhow::anyhow!(
                "all files must have the same Client ID as {}\nfile {} does not",
                quick_file_name(&comparison_path),
                quick_file_name(&path),
            ));
        }
    }

    // Check that each file is complementary
    for ((p, ppath), (r, rpath)) in prim_data.iter().zip(reli_data.iter()) {
        if !p.complement_of(r) {
            return Err(anyhow::anyhow!(
                "files {} and {} cannot be compared",
                quick_file_name(&ppath),
                quick_file_name(&rpath),
            ));
        }
    }

    Ok(())
}
