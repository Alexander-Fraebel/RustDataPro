use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

pub fn overwrite_file(pathbuf: Result<PathBuf>, data: &str) -> Result<()> {
    match pathbuf {
        Ok(pb) => {
            if pb.exists() {
                std::fs::write(pb, data)?
            } else {
                let mut writer = BufWriter::new(
                    File::create_new(&pb)
                        .with_context(|| format!("error creating file named: {:?}", pb))?,
                );
                writer.write_all(data.as_bytes())?;
                writer.flush()?;
            }
        }
        Err(e) => return Err(e),
    }
    Ok(())
}
