use escpos::errors::{PrinterError, Result}; // Corrected path
use std::{
    cell::OnceCell,
    fmt::{self, Debug, Formatter},
    slice,
};

use windows::{
    Win32::Graphics::Printing::{
        EnumPrintersW, PRINTER_ENUM_LOCAL, PRINTER_ENUM_NETWORK, PRINTER_INFO_2W, PRINTER_INFO_4W,
    },
    core::PWSTR,
};

#[derive(Clone)]
pub struct WindowsPrinter {
    raw_vec: Vec<u16>,
    raw_name: PWSTR,
    name: OnceCell<String>,
    is_offline: bool,
}

impl WindowsPrinter {
    pub fn new(printer_name: PWSTR, is_offline: bool) -> Self {
        unsafe {
            let mut raw_vec = printer_name.as_wide().to_vec();
            raw_vec.push(0x0);
            raw_vec.push(0x0);
            Self {
                raw_name: PWSTR(raw_vec.as_mut_ptr()),
                raw_vec,
                name: OnceCell::new(),
                is_offline: is_offline,
            }
        }
    }

    pub fn from_str(printer_name: &str) -> Result<Self> {
        Self::list_printers()?
            .into_iter()
            .find(|printer| printer.get_name() == printer_name)
            .ok_or_else(|| PrinterError::Io(format!("Printer not found: {}", printer_name)))
    }

    pub fn get_raw_vec(&self) -> &Vec<u16> {
        &self.raw_vec
    }
    pub fn get_raw_name(&self) -> PWSTR {
        self.raw_name
    }
    pub fn get_name(&self) -> &str {
        self.name.get_or_init(|| unsafe {
            PWSTR(self.raw_vec.clone().as_mut_ptr())
                .to_string()
                .unwrap()
        })
    }

    pub fn list_printers() -> Result<Vec<WindowsPrinter>> {
        let mut needed = 0;
        let mut returned = 0;
        let mut buffer: Vec<u8>;
        let mut is_offline = false;
        const FLAGS: u32 = PRINTER_ENUM_LOCAL | PRINTER_ENUM_NETWORK;
        const LEVEL: u32 = 2;
        unsafe {
            let _ = EnumPrintersW(
                FLAGS,
                PWSTR::null(),
                LEVEL,
                None,
                &mut needed,
                &mut returned,
            );

            buffer = vec![0; needed as usize];

            let _ = EnumPrintersW(
                FLAGS,
                PWSTR::null(),
                LEVEL,
                Some(buffer.as_mut_slice()),
                &mut needed,
                &mut returned,
            );
            let sliced =
                slice::from_raw_parts(buffer.as_ptr() as *const PRINTER_INFO_2W, returned as usize);

            // Prints the status of the printer
            for info in sliced {
                if info.Attributes & PRINTER_ATTRIBUTE_WORK_OFFLINE != 0 {
                    println!("Printer is offline");
                    is_offline = true;
                } else {
                    println!("Printer is online");
                    is_offline = false;
                }
            }

            // Return printers with status (info.pStatus)
            let printers = sliced
                .iter()
                .map(|info| WindowsPrinter::new(info.pPrinterName, is_offline))
                .collect::<Vec<WindowsPrinter>>();
            Ok(printers)
        }
    }
}

impl Debug for WindowsPrinter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("WindowsPrinter")
            .field("raw_name", &self.raw_name)
            .field("name", &self.get_name())
            .field("is_offline", &self.is_offline)
            .finish()
    }
}
