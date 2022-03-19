use esp_idf_hal;
use esp_idf_hal::i2c;

use math::*;

use anyhow;

use std::io::{Read, Write};

const PCA_PRESCALER: u8 = 0x7A;
const PCA_ADDR: u8 = 0x40;

pub fn deg_to_int(deg: u8) -> [u8; 2] {
    let deg = deg % 180;
    let deg: i32 = deg as i32 - 90 as i32;

    let ms = 1.5 - deg * 0.5 / 90;
    // PCA values
    ((ms / 20 * 4096) as u16).to_be_bytes()
}

pub struct Leg {
    dirs: u16,
    servos: [u8; 3],
    rot: mut[u32; 3], // front to back
}

impl Leg {
    #[inline]
    pub fn fw_kinematic(&mut self, deg1: u32, deg2: u32, deg3: u32) -> (u32, u32) {
        // zero cost
        let degs = deg[deg1, deg2, deg3];
        for (i, ii) in self.servos.enumerate() {
            pca_setup_servo(i2c, i, deg_to_int(degs[ii]));
        }
        unimplemented!();
    }

    pub fn inv_kinematic(&mut self, x_mm: u32, y_mm: u32) {
        for i in self.servo {
            let base_deg = asin(xy.0 / xy.1);
            pca_change_servo(i2c, self.servo, deg_to_int(base_deg) * (acos(xy.0 / xy.1 / 2))); // cheat
        }
    }

    pub fn forward(&mut self, deg1: u32, deg2: u32, deg3: u32, i2c: &mut I2c) {

    }

    #[inline(never)]
    pub fn backward(&mut self, xy: (u32, u32), i2c: &mut I2c) {
        // TODO: Jackobian or CCD impl
        unimplemented!();
    }
}

pub fn pca_setup_servo<I: i2c::I2c>(i2c: &mut I) -> Result<(), anyhow::Error> {
    // 50Hz operation
    (*i2c).write(PCA_ADDR, [0xFE, PCA_PRESCALER])?;
    Ok(())
}

pub fn pca_change_servo<I: i2c::I2c>(
    i2c: &mut I,
    channel: u8,
    pwm: [u8; 2],
) -> Result<(), anyhow::Error> {
    assert!(channel <= 16);
    (*i2c).write(PCA_ADDR, [(channel * 4) - 4, 0, 0, pwm[0], pwm[1]])?;

    log::info!("servo{} to {}",channel, pwm.0);
    Ok(())
}
