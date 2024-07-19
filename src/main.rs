use core::fmt::Write;
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::FromValueType;
use ssd1306::{mode::TerminalMode, prelude::*, I2CDisplayInterface, Ssd1306};


fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;

    log::info!("Starting I2C SSD1306 test");

    let config = I2cConfig::new().baudrate(100_u32.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_terminal_mode();
    display.init().unwrap();
    display.clear().unwrap();

    // Spam some characters to the display
    for c in 97..123 {
        let _ = display.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) })?;
    }
    for c in 65..91 {
        let _ = display.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) })?;
    }

    // The `write!()` macro is also supported
    write!(display, "Hello, {}", "world")?;

    Ok(())
}
