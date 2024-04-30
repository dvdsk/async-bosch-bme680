use embedded_hal_async::i2c::{I2c, SevenBitAddress};

/// All possible errors
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "postcard",
    derive(postcard::experimental::max_size::MaxSize)
)]
pub enum BmeError<I2C>
where
    I2C: I2c<SevenBitAddress>,
    I2C::Error: defmt::Format,
{
    #[cfg_attr(feature = "thiserror", error("Error during I2C write operation."))]
    WriteError(I2C::Error),
    #[cfg_attr(feature = "thiserror", error("Error during I2C WriteRead operation."))]
    WriteReadError(I2C::Error),
    #[cfg_attr(
        feature = "thiserror",
        error("Got an unexpected ChipId during sensor initialization.")
    )]
    UnexpectedChipId(u8),
    #[cfg_attr(feature = "thiserror", error("After running the measurement the sensor blocks until the 'new data bit' of the sensor is set."))]
    #[cfg_attr(
        feature = "thiserror",
        error(
            "Should this take more than 5 tries an error is returned instead of incorrect data."
        )
    )]
    MeasuringTimeOut,
}

impl<I2C> defmt::Format for BmeError<I2C>
where
    I2C: I2c<SevenBitAddress>,
    I2C::Error: defmt::Format,
{
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            BmeError::WriteReadError(e) => defmt::write!(fmt, "WriteReadError: {}", e),
            BmeError::WriteError(e) => defmt::write!(fmt, "WriteError: {}", e),
            BmeError::UnexpectedChipId(chip_id) => {
                defmt::write!(fmt, "Got unimplemented chip id: {}", chip_id)
            }
            BmeError::MeasuringTimeOut => defmt::write!(fmt, "Timed out while waiting for new measurement values. Either no new data or the sensor took unexpectedly long to finish measuring."),
        }
    }
}
