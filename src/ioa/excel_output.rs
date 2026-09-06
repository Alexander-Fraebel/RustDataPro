use crate::{
    data::{IoaData, OutputData},
    ioa::quick_file_name,
};
use anyhow::Result;
use egui::Key;
use rust_xlsxwriter::*;
use std::path::PathBuf;

fn write_excel_line<'a>(
    worksheet: &'a mut Worksheet,
    row: u32,
    name: &'static str,
    it: impl Iterator<Item = (&'a Key, &'a f32)>,
    format: &Format,
) -> Result<()> {
    worksheet.write(row, 0, name)?;
    let mut col = 1;
    for (_, n) in it {
        worksheet.write_number_with_format(row, col, *n, format)?;
        col += 1;
    }
    Ok(())
}

pub fn save_excel_workbook(
    ioa_data: &IoaData,
    file_stem: &str,
    prim_data: &Vec<(OutputData, PathBuf)>,
    reli_data: &Vec<(OutputData, PathBuf)>,
) -> Result<()> {
    let mut workbook = Workbook::new();
    let data_summary = workbook.add_worksheet();
    data_summary.set_name("Summary")?;
    data_summary.set_column_width(0, 22)?;
    data_summary.set_column_range_width(1, 20, 10)?;

    let num_format = Format::new().set_num_format("0%");

    let map = prim_data[0].0.ksf.create_map();
    let mut col = 1;
    for (k, _) in ioa_data.sixty_sec_interval.iter() {
        data_summary.write(0, col, map.get(k).unwrap_or(&String::from("UNKNOWN KEY")))?;
        col += 1;
    }
    write_excel_line(
        data_summary,
        1,
        "60 Second Interval",
        ioa_data.sixty_sec_interval.iter(),
        &num_format,
    )?;
    write_excel_line(
        data_summary,
        2,
        "10 Second Interval",
        ioa_data.ten_sec_interval.iter(),
        &num_format,
    )?;
    write_excel_line(
        data_summary,
        3,
        "Total Count",
        ioa_data.total_count.iter(),
        &num_format,
    )?;
    write_excel_line(
        data_summary,
        4,
        "Total Duration",
        ioa_data.total_duration.iter(),
        &num_format,
    )?;

    let data_sources = workbook.add_worksheet();
    data_sources.set_name("Sources")?;
    data_sources.set_column_width(0, 16)?;
    data_sources.set_column_width(2, 22)?;
    data_sources.set_column_width(3, 22)?;
    data_sources.write(0, 0, "Data Taken from")?;
    let mut ctr = 0;
    for (p, r) in prim_data.iter().zip(reli_data.iter()) {
        data_sources.write(ctr, 2, quick_file_name(&p.1))?;
        data_sources.write(ctr, 3, quick_file_name(&r.1))?;
        ctr += 1;
    }

    workbook.save(&format!("{}.xlsx", file_stem))?;
    Ok(())
}
