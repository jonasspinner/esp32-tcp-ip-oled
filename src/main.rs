use embedded_graphics::image::Image;
use embedded_graphics::prelude::Point;
use embedded_graphics::Drawable;
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::modem::WifiModemPeripheral;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::FromValueType;
use esp_idf_svc::sys::EspError;
use shared::{apply, BitImage, Command, Deserialize};
use ssd1306::mode::DisplayConfig;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use std::env;
use std::net::TcpListener;
use rand::thread_rng;

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

fn main() -> Result<(), anyhow::Error> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let _wifi = wifi_create(peripherals.modem)?;

    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;

    let config = I2cConfig::new().baudrate(100_u32.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    display.clear_buffer();
    display.flush().unwrap();

    display.set_display_on(true).unwrap();

    let mut image = BitImage::new(128, 64);

    let listener = TcpListener::bind("0.0.0.0:8080")?;

    let mut rng = thread_rng();

    for stream in listener.incoming() {
        let mut stream = stream?;
        while let Ok(command) = Command::deserialize(&mut stream) {
            println!("> {:?}", command);
            apply(&mut image, command, &mut rng);

            Image::new(&image.as_image_raw(), Point::zero())
                .draw(&mut display)
                .unwrap();
            display.flush().unwrap();
        }
    }

    Ok(())
}

fn wifi_create<M: WifiModemPeripheral>(
    modem: impl Peripheral<P = M> + 'static,
) -> Result<esp_idf_svc::wifi::EspWifi<'static>, EspError> {
    use esp_idf_svc::eventloop::*;
    use esp_idf_svc::nvs::*;
    use esp_idf_svc::wifi::*;

    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut esp_wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs.clone()))?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sys_loop.clone())?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        ..Default::default()
    }))?;
    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;
    Ok(esp_wifi)
}
