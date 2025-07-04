// Conditionally compile and import the windows printing module
#[cfg(feature = "windows-support")]
mod windows_printing;

use escpos::driver::*;
use escpos::errors::Result;
use escpos::printer::Printer;
use escpos::utils::*;

// Conditionally import the windows driver and printer types
#[cfg(feature = "windows-support")]
use windows_printing::{WindowsDriver, WindowsPrinter};

/// All printing logic is moved into this generic function.
/// It can accept any type `D` as long as it implements the `Driver` trait.
fn print_receipt<D: Driver>(driver: D) -> Result<()> {
    let mut printer = Printer::new(driver, Protocol::default(), None);

    printer
        .init()?
        .debug_mode(Some(DebugMode::Dec))
        .justify(JustifyMode::CENTER)?
        .bold(true)?
        .writeln("Company Name")?
        .writeln("Byline")?
        .feed()?
        .writeln("TAX_ID: ABC123")?
        .writeln("ADDRESS LINE1")?
        .writeln("ADDRESS LINE2")?
        .writeln("")?
        .bold(true)?
        .write("Ticket ID: ")?
        .bold(false)?
        .writeln("123")?
        .bold(true)?
        .write("Date: ")?
        .bold(false)?
        .writeln("07/JUN/2025")?
        .writeln("")?
        .writeln("")?
        .print_cut()?;

    Ok(())
}

fn main() -> Result<()> {
    // This `main` function is now only responsible for creating the correct driver
    // and passing it to our generic printing function.

    #[cfg(feature = "windows-support")]
    {
        // --- Windows Driver ---
        // IMPORTANT: Replace "XP-80C" with the name of your printer on Windows.
        const PRINTER_NAME: &str = "XP-80C";

        let windows_printer = WindowsPrinter::from_str(PRINTER_NAME).map_err(|e| {
            eprintln!("Error: Could not find printer '{}'.", PRINTER_NAME);
            eprintln!("Listing available printers...");
            match WindowsPrinter::list_printers() {
                Ok(printers) if printers.is_empty() => eprintln!(" -> No printers found."),
                Ok(printers) => {
                    for p in printers {
                        eprintln!(" -> {}", p.get_name());
                    }
                }
                Err(list_err) => eprintln!(" -> Could not list printers: {}", list_err),
            }
            e
        })?;
        let driver = WindowsDriver::open(&windows_printer)?;
        print_receipt(driver)
    }

    #[cfg(not(feature = "windows-support"))]
    {
        // --- Linux/USB Driver ---
        let driver = UsbDriver::open(0x1504, 0x006e, None)?;
        print_receipt(driver)
    }
}
