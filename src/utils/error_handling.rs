use itertools::Itertools;

/// Create a windows style error dialog
pub fn windows_error_dialog(message: anyhow::Error) {
    win_msgbox::error::<win_msgbox::Okay>(&message.chain().map(|e| e.to_string()).join("\n"))
        .title("Error")
        .set_foreground()
        .show()
        .expect("unable to create dialog box");
}

/// Ask if the user is sure. Return true if they click Yes and return false if they click No.
pub fn are_you_sure_dialog(message: &str) -> bool {
    win_msgbox::warning::<win_msgbox::YesNo>(message)
        .title("Are you sure?")
        .set_foreground()
        .show()
        .expect("unable to create dialog box")
        == win_msgbox::YesNo::Yes
}

/// Pop up an error dialog if the Result is Err while ignoring Ok.
#[macro_export]
macro_rules! quick_error {
    ($result:expr) => {
        if let Err(e) = $result {
            win_msgbox::error::<win_msgbox::Okay>(&itertools::Itertools::join(
                &mut e.chain().map(|e| e.to_string()),
                "\n",
            ))
            .title("Error")
            .set_foreground()
            .show()
            .expect("unable to create dialog box");
        }
    };
}
