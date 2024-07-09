extern crate winapi;

use std::ptr;
use winapi::um::winuser::{MessageBoxW, MB_ICONQUESTION, MB_OKCANCEL, MB_SYSTEMMODAL};

pub fn main() -> bool {
    // First confirmation message box notifying user about program
    let confirm_msg1 = "TH3SW4N is malware that will encrypt all user files and delete system files if the countdown reaches zero. Are you sure you wish to proceed?";
    let confirm_title1 = "TH3SW4N Execution";

    let result1 = unsafe {
        MessageBoxW(
            ptr::null_mut(),
            confirm_msg1.encode_utf16().chain(Some(0)).collect::<Vec<u16>>().as_ptr(),
            confirm_title1.encode_utf16().chain(Some(0)).collect::<Vec<u16>>().as_ptr(),
            MB_OKCANCEL | MB_ICONQUESTION | MB_SYSTEMMODAL,
        )
    };

    // Check the result of the first message box
    if result1 == 1 {
        // Second confirmation message box to verify user wishes to proceed
        let confirm_msg2 = "Are you sure you want to proceed with running TH3SW4N malware? You will be held liable for any damages caused.";
        let confirm_title2 = "TH3SW4N Execution";

        let result2 = unsafe {
            MessageBoxW(
                ptr::null_mut(),
                confirm_msg2.encode_utf16().chain(Some(0)).collect::<Vec<u16>>().as_ptr(),
                confirm_title2.encode_utf16().chain(Some(0)).collect::<Vec<u16>>().as_ptr(),
                MB_OKCANCEL | MB_ICONQUESTION | MB_SYSTEMMODAL,
            )
        };

        // Check the result of the second message box
        if result2 == 1 {
            // Proceed with launching TH3SW4N
            return true;
        } else {
            // Close the program
            return false;
        }
    } else {
        // Close the program
        return false;
    }
}