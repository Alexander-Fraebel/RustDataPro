pub mod calculations;
pub mod excel_output;
pub mod ioa_page;
pub mod validate_files;

pub use ioa_page::IoaPage;

pub fn quick_file_name(pathbuf: &std::path::Path) -> std::borrow::Cow<'_, str> {
    pathbuf
        .file_name()
        .unwrap_or(&std::ffi::OsStr::new("INVALID FILE NAME"))
        .to_string_lossy()
}
