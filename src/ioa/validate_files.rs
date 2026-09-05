use crate::{data::OutputData, utils::quick_file_name};
use anyhow::Result;
use std::path::PathBuf;

/// Assure that files have been loaded, that they all use the same KSFs, they all have the same client ID, and that each pair of files is for a matching session. Return a meaningful error is necessary.
pub fn validate_files(
    prim_data: &Vec<(OutputData, PathBuf)>,
    reli_data: &Vec<(OutputData, PathBuf)>,
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

    // Assume the first primary file is correct and check that all files use the save KSF and have the same client ID
    // let ksf = &prim_data[0].0.ksf;
    let id = &prim_data[0].0.client_id;
    let comparison_path = &prim_data[0].1;
    for (o, path) in prim_data.iter().chain(reli_data.iter()).skip(1) {
        // if &o.ksf != ksf {
        //     return Err(anyhow::anyhow!(
        //         "all files must have the same KSF as {}\nfile {} does not",
        //         quick_file_name(&comparison_path),
        //         quick_file_name(&path)
        //     ));
        // }
        if &o.client_id != id {
            return Err(anyhow::anyhow!(
                "all files must have the same Client ID as {}\nfile {} does not",
                quick_file_name(&comparison_path),
                quick_file_name(&path),
            ));
        }
    }

    // Check that each file is for the same session
    for ((p, ppath), (r, rpath)) in prim_data.iter().zip(reli_data.iter()) {
        if p.session_number != r.session_number {
            return Err(anyhow::anyhow!(
                "files {} and {} are not for the same session",
                quick_file_name(&ppath),
                quick_file_name(&rpath),
            ));
        }
    }

    Ok(())
}
