/// All possible errors
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum BmeError<E>
where
    E: defmt::Format,
{
    #[cfg_attr(feature = "thiserror", error("Error during I2C write operation: {0}"))]
    WriteError(E),
    #[cfg_attr(
        feature = "thiserror",
        error("Error during I2C WriteRead operation: {0}")
    )]
    WriteReadError(E),
    #[cfg_attr(
        feature = "thiserror",
        error("Got an unexpected ChipId during sensor initialization. Got id: {0}")
    )]
    UnexpectedChipId(u8),
    #[cfg_attr(
        feature = "thiserror",
        error("Waiting for the `new data bit` is taking to long")
    )]
    MeasuringTimeOut,
}

#[cfg(feature = "postcard")]
impl<I2C> postcard::experimental::max_size::MaxSize for BmeError<I2C::Error>
where
    I2C: I2c<SevenBitAddress>,
    I2C::Error: defmt::Format,
{
    // this is unrealistically large. It might still be too small if the
    // I2C::Error has an absurd amount of data in it
    const POSTCARD_MAX_SIZE: usize = 10;
}

impl<E> defmt::Format for BmeError<E>
where
    E: defmt::Format,
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
