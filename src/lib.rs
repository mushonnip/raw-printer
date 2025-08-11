#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HANDLE;

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
        let result = write_to_device(
            "ZDesigner ZD220-203dpi ZPL",
            "^FDhello world",
            Some("Test Document"),
        );
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
pub fn write_to_device(
    printer: &str,
    payload: &str,
    _document_name: Option<&str>,
) -> Result<usize, std::io::Error> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut device = OpenOptions::new().write(true).open(printer)?;
    let bytes_written = device.write(payload.as_bytes())?;
    device.flush()?; // Ensure data is written
    Ok(bytes_written)
}

#[cfg(target_os = "windows")]
pub fn write_to_device(
    printer: &str,
    payload: &str,
    document_name: Option<&str>,
) -> Result<usize, std::io::Error> {
    use std::ffi::CString;
    use std::ptr;
    use windows::core::PCSTR;
    use windows::Win32::Graphics::Printing::{
        EndDocPrinter, EndPagePrinter, OpenPrinterA, StartDocPrinterA, StartPagePrinter,
        WritePrinter, DOC_INFO_1A, PRINTER_ACCESS_USE, PRINTER_DEFAULTSA,
    };

    let printer_name = CString::new(printer)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let mut printer_handle: HANDLE = HANDLE(std::ptr::null_mut());

    // Open the printer
    unsafe {
        let pd = PRINTER_DEFAULTSA {
            pDatatype: windows::core::PSTR(ptr::null_mut()),
            pDevMode: ptr::null_mut(),
            DesiredAccess: PRINTER_ACCESS_USE,
        };

        if OpenPrinterA(
            PCSTR(printer_name.as_ptr() as *const u8),
            &mut printer_handle,
            Some(&pd),
        )
        .is_err()
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to open printer: {}", printer),
            ));
        }

        // Ensure proper cleanup with RAII-style wrapper
        let _cleanup = PrinterGuard::new(printer_handle);

        let doc_name = document_name.unwrap_or("RAW_Print");
        let doc_name_cstring = CString::new(doc_name)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        let datatype_cstring = CString::new("RAW").unwrap();

        let doc_info = DOC_INFO_1A {
            pDocName: windows::core::PSTR(doc_name_cstring.as_ptr() as *mut u8),
            pOutputFile: windows::core::PSTR::null(),
            pDatatype: windows::core::PSTR(datatype_cstring.as_ptr() as *mut u8),
        };

        // Start the document
        let job_id = StartDocPrinterA(printer_handle, 1, &doc_info as *const _ as _);
        if job_id == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to start document",
            ));
        }

        // Start the page
        if !StartPagePrinter(printer_handle).as_bool() {
            let _ = EndDocPrinter(printer_handle);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to start page",
            ));
        }

        let buffer = payload.as_bytes();
        let mut bytes_written: u32 = 0;

        let write_result = WritePrinter(
            printer_handle,
            buffer.as_ptr() as _,
            buffer.len() as u32,
            &mut bytes_written,
        );

        // Always end the page and document, regardless of write success
        let _ = EndPagePrinter(printer_handle);
        let _ = EndDocPrinter(printer_handle);

        if !write_result.as_bool() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write to printer",
            ));
        }

        Ok(bytes_written as usize)
    }
}

#[cfg(target_os = "windows")]
struct PrinterGuard {
    handle: HANDLE,
}

#[cfg(target_os = "windows")]
impl PrinterGuard {
    fn new(handle: HANDLE) -> Self {
        Self { handle }
    }
}

#[cfg(target_os = "windows")]
impl Drop for PrinterGuard {
    fn drop(&mut self) {
        unsafe {
            use windows::Win32::Graphics::Printing::ClosePrinter;
            let _ = ClosePrinter(self.handle);
        }
    }
}