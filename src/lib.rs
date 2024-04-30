//! This a pure rust crate to read out sensor data from the [BME680](https://www.bosch-sensortec.com/products/environmental-sensors/gas-sensors/bme680/) environmental sensor from bosch.
//!
//! Notes:
//! This library only supports reading out data with I²C but not SPI and
//! only works for the BME680 and NOT for the BME688 though this could be implemented.
//! The [official](https://github.com/BoschSensortec/BME68x-Sensor-API/) c implementation from Bosch was used as a reference.
//!
//! For further information about the sensors capabilities and settings refer to the official [product page](https://www.bosch-sensortec.com/products/environmental-sensors/gas-sensors/bme680/).

// TODO add example here
#![no_std]
#![forbid(unsafe_code)]

use self::config::{SensorMode, Variant};
use bitfields::RawConfig;
use constants::{
    CYCLE_DURATION, GAS_MEAS_DURATION, LEN_CONFIG, TPH_SWITCHING_DURATION, WAKEUP_DURATION,
};
use data::CalibrationData;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::{I2c, SevenBitAddress};
use i2c_helper::I2CHelper;

pub use self::config::{Configuration, DeviceAddress, GasConfig, IIRFilter, Oversampling};
use crate::data::{calculate_humidity, calculate_pressure, calculate_temperature};
pub use data::MeasurementData;
pub use error::BmeError;

mod bitfields;
mod calculations;
mod config;
mod constants;
mod data;
mod error;
mod i2c_helper;

/// Sensor driver
pub struct Bme680<I2C, D> {
    // actually communicates with sensor
    i2c: I2CHelper<I2C, D>,
    // calibration data that was saved on the sensor
    calibration_data: CalibrationData,
    // used to calculate measurement delay period
    sensor_config: RawConfig<[u8; LEN_CONFIG]>,
    // needed to calculate the gas resistance since it differs between bme680 and bme688
    variant: Variant,
}
impl<I2C, D> Bme680<I2C, D>
where
    I2C: I2c<SevenBitAddress>,
    I2C::Error: defmt::Format,
    D: DelayNs,
{
    /// Creates a new instance of the Sensor
    ///
    /// # Arguments
    /// * `delayer` - Used to wait for the triggered measurement to finish
    /// * `ambient_temperature` - Needed to calculate the heater target temperature
    pub async fn new(
        i2c_interface: I2C,
        device_address: DeviceAddress,
        delayer: D,
        sensor_config: &Configuration,
        ambient_temperature: i32,
    ) -> Result<Self, BmeError<I2C::Error>> {
        let mut i2c =
            I2CHelper::new(i2c_interface, device_address, delayer, ambient_temperature).await?;

        let calibration_data = i2c.get_calibration_data().await?;
        let sensor_config = i2c.set_config(sensor_config, &calibration_data).await?;
        let variant = i2c.get_variant_id().await?;
        let bme = Self {
            i2c,
            calibration_data,
            sensor_config,
            variant,
        };

        Ok(bme)
    }
    /// Returns the wrapped i2c interface
    pub fn into_inner(self) -> I2C {
        self.i2c.into_inner()
    }

    async fn put_to_sleep(&mut self) -> Result<(), BmeError<I2C::Error>> {
        self.i2c.set_mode(SensorMode::Sleep).await
    }
    pub async fn set_configuration(&mut self, config: &Configuration) -> Result<(), BmeError<I2C::Error>> {
        self.put_to_sleep().await?;
        let new_config = self.i2c.set_config(config, &self.calibration_data).await?;
        // current conf is used to calculate measurement delay period
        self.sensor_config = new_config;
        Ok(())
    }
    /// Trigger a new measurement.
    /// # Errors
    /// If no new data is generated in 5 tries a Timeout error is returned.
    // Sets the sensor mode to forced
    // Tries to wait 5 times for new data with a delay calculated based on the set sensor config
    // If no new data could be read in those 5 attempts a Timeout error is returned
    pub async fn measure(&mut self) -> Result<MeasurementData, BmeError<I2C::Error>> {
        self.i2c.set_mode(SensorMode::Forced).await?;
        let delay_period = self.calculate_delay_period_us();
        self.i2c.delay(delay_period).await;
        // try read new values 5 times and delay if no new data is available or the sensor is still measuring
        for _i in 0..5 {
            let raw_data = self.i2c.get_field_data().await?;
            if !raw_data.measuring() && raw_data.new_data() {
                let (temperature, t_fine) =
                    calculate_temperature(raw_data.temperature_adc().0, &self.calibration_data);
                // update the current ambient temperature which is needed to calculate the target heater temp
                self.i2c.ambient_temperature = temperature as i32;
                let pressure =
                    calculate_pressure(raw_data.pressure_adc().0, &self.calibration_data, t_fine);
                let humidity =
                    calculate_humidity(raw_data.humidity_adc().0, &self.calibration_data, t_fine);
                let gas_resistance = if raw_data.gas_valid() && !raw_data.gas_measuring() {
                    let gas_resistance = self.variant.calc_gas_resistance(
                        raw_data.gas_adc().0,
                        self.calibration_data.range_sw_err,
                        raw_data.gas_range() as usize,
                    );
                    Some(gas_resistance)
                } else {
                    None
                };

                let data = MeasurementData {
                    temperature,
                    gas_resistance,
                    humidity,
                    pressure,
                };
                return Ok(data);
            } else {
                self.i2c.delay(delay_period).await;
            }
        }
        // Shouldn't happen
        Err(BmeError::MeasuringTimeOut)
    }
    // calculates the delay period needed for a measurement in microseconds.
    fn calculate_delay_period_us(&self) -> u32 {
        let mut measurement_cycles: u32 = 0;
        measurement_cycles += self.sensor_config.temperature_oversampling().cycles();
        measurement_cycles += self.sensor_config.humidity_oversampling().cycles();
        measurement_cycles += self.sensor_config.pressure_oversampling().cycles();

        let mut measurement_duration = measurement_cycles * CYCLE_DURATION;
        measurement_duration += TPH_SWITCHING_DURATION;
        measurement_duration += GAS_MEAS_DURATION;

        measurement_duration += WAKEUP_DURATION;

        measurement_duration
    }

    pub fn get_calibration_data(&self) -> &CalibrationData {
        &self.calibration_data
    }
}
