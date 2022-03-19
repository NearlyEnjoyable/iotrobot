//use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Condvar, Mutex};
use std::{cell::RefCell, env, sync::atomic::*, sync::Arc, thread, time::*};

use anyhow::bail;

use log::*;

use esp_idf_hal;
use esp_idf_svc;
use smol;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported  ^..^

use embedded_hal::adc::OneShot;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;

use embedded_svc::eth;
use embedded_svc::eth::{Eth, TransitionalState};
use embedded_svc::httpd::registry::*;
use embedded_svc::httpd::*;
use embedded_svc::io;
use embedded_svc::ipv4;
use embedded_svc::mqtt::client::{Publish, QoS};
use embedded_svc::ping::Ping;
use embedded_svc::sys_time::SystemTime;
use embedded_svc::timer::TimerService;
use embedded_svc::timer::*;
use embedded_svc::wifi::*;

use esp_idf_svc::eth::*;
use esp_idf_svc::eventloop::*;
use esp_idf_svc::eventloop::*;
use esp_idf_svc::httpd;
use esp_idf_svc::httpd::ServerRegistry;
use esp_idf_svc::mqtt::client::*;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sntp;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::systime::EspSystemTime;
use esp_idf_svc::timer::*;
use esp_idf_svc::wifi::*;

use esp_idf_hal::adc;
use esp_idf_hal::delay;
use esp_idf_hal::gpio;
use esp_idf_hal::i2c;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;

use esp_idf_sys::esp;
use esp_idf_sys::{self, c_types};

use pwm_pca9685::{Channel, Pca9685};

mod api;
mod sens;
mod servo;

// I2C handler with multithread guards
static I2C: Option<Arc<Mutex<I2c>>> = None;

//const SSID: &str = "robot";
//const PASS: &str = "esp32";


fn main() -> Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Servo offset and rotation
    legs = Leg[
    {0b011001101111, [1,2,3], [0,0,0]},
    {0b011001101111, [5,6,7], [0,0,0]},
    {0b10011000000, [9,10,11], [0,0,0]},
    {0b10011000000, [13,14,15], [0,0,0]},
    ];
    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new()?);
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new()?);

    let i2c = i2c::Master::new(
        peripherals.i2c0,
        i2c::MasterPins {
            sda: pins.gpio18,
            scl: pins.gpio19,
        },
        i2c::config::MasterConfig::new()
            .baudrate(400.kHz().into())
            .timeout(None)
            .sda_enable_pullup(false)
            .scl_enable_pullup(false),
    )?;

    log::info!("running");

    pca_setup_servo(&mut i2c);

    // FreeRTOS task scheduling
    loop {
        xTaskCreatePinnedToCore(servo, "servo", 19, None, 0)?;

        xTaskPinnedToCore(imu, "imu", 19, None, 0);

        //xTaskPinnedToCore(nw, "nw", 18, None, 1);
    }

    Ok(())
}

// Only goes forward
fn servo() {
    // if nw_buf == 0 {
    for leg in legs[0,3] {
        leg.forward();
    }
    // delay with kernel
    xTaskDelay(200.ms()?);

    for leg in legs[1,2] {
        leg.forward();
    }
// TODO: dinamic control

}

fn imu() {
    let buf = Vec::new();
    unsafe {
        // 0x20 hex
        I2C.read(MPU9250_ADDR, &mut buf);
        log::info!("acc: {} grav: {}", buf[0..2], buf[3..4]);
    }
}
