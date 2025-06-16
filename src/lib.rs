#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_write_to_device_linux() {
        let result = write_to_device("/dev/usb/lp0", "^FDhello world", Some("Test Document"));
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_write_to_device_windows() {
        let result = write_to_device("ZDesigner ZD220-203dpi ZPL", "^FDhello world", Some("Test Document"));
        assert!(result.is_ok());
    }
}

/// # Platform-specific Behavior
///
/// This function returns a result containing the size of bytes written on success or an error.
///
/// - On Linux and Windows, the result type is `Result<usize, Error>`.
/// - Note: On Windows, the original bytes written are u32 but cast to usize.
///
/// # Examples
///
/// ```
/// let zpl = "^FDhello world";
/// let printer = "/dev/usb/lp0";
/// let result = raw_printer::write_to_device(printer, zpl, Some("My Custom Document"));
/// 
/// assert!(result.is_ok());
/// 
/// ```
#[cfg(target_os = "linux")]
pub fn write_to_device(printer: &str, payload: &str, _document_name: Option<&str>) -> Result<usize, std::io::Error> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let device_path = OpenOptions::new().write(true).open(printer);

    match device_path {
        Ok(mut device) => {
            let bytes_written = device.write(payload.as_bytes())?;
            Ok(bytes_written)
        }
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    }
}

#[cfg(target_os = "windows")]
pub fn write_to_device(printer: &str, payload: &str, document_name: Option<&str>) -> Result<usize, std::io::Error> {
    use std::ffi::CString;
    use std::ptr;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Graphics::Printing::{
        ClosePrinter, EndDocPrinter, EndPagePrinter, OpenPrinterA, StartDocPrinterA,
        StartPagePrinter, WritePrinter, DOC_INFO_1A, PRINTER_ACCESS_USE, PRINTER_DEFAULTSA,
    };

    let printer_name = CString::new(printer).unwrap_or_default(); // null-terminated string
    let mut printer_handle: HANDLE = HANDLE(std::ptr::null_mut());

    // Open the printer
    unsafe {
        let pd = PRINTER_DEFAULTSA {
            pDatatype: windows::core::PSTR(ptr::null_mut()),
            pDevMode: ptr::null_mut(),
            DesiredAccess: PRINTER_ACCESS_USE,
        };

        if OpenPrinterA(
            windows::core::PCSTR(printer_name.as_bytes().as_ptr()),
            &mut printer_handle,
            Some(&pd),
        )
        .is_ok()
        {
            let doc_name = document_name.unwrap_or("Print Job");
            let doc_name_cstring = CString::new(doc_name).unwrap_or_default();
            
            let doc_info = DOC_INFO_1A {
                pDocName: windows::core::PSTR(doc_name_cstring.as_ptr() as *mut u8),
                pOutputFile: windows::core::PSTR::null(),
                pDatatype: windows::core::PSTR("RAW\0".as_ptr() as *mut u8),
            };

            // Start the document
            let job = StartDocPrinterA(printer_handle, 1, &doc_info as *const _ as _);
            if job == 0 {
                return Err(std::io::Error::from(windows::core::Error::from_win32()));
            }

            // Start the page
            if !StartPagePrinter(printer_handle).as_bool() {
                return Err(std::io::Error::from(windows::core::Error::from_win32()));
            }

            let buffer = payload.as_bytes();

            let mut bytes_written: u32 = 0;
            if !WritePrinter(
                printer_handle,
                buffer.as_ptr() as _,
                buffer.len() as u32,
                &mut bytes_written,
            )
            .as_bool()
            {
                return Err(std::io::Error::from(windows::core::Error::from_win32()));
            }

            // End the page and document
            let _ = EndPagePrinter(printer_handle);
            let _ = EndDocPrinter(printer_handle);
            let _ = ClosePrinter(printer_handle);
            return Ok(bytes_written as usize);
        } else {
            return Err(std::io::Error::from(windows::core::Error::from_win32()));
        }
    }
}
