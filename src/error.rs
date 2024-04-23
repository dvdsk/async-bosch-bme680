use core::fmt::Formatter;
use embedded_hal_async::i2c::{I2c, SevenBitAddress};


/// All possible errors
pub enum BmeError<I2C>
where
    I2C: I2c<SevenBitAddress>
{
    /// Error during I2C write operation.
    WriteError(I2C::Error),
    /// Error during I2C WriteRead operation.
    WriteReadError(I2C::Error),
    /// Got an unexpected ChipId during sensor initialization.
    UnexpectedChipId(u8),
    /// After running the measurement the sensor blocks until the 'new data bit' of the sensor is set.
    /// Should this take more than 5 tries an error is returned instead of incorrect data.
    MeasuringTimeOut,
}

impl<I2C> core::fmt::Debug for BmeError<I2C>
where
    I2C: I2c<SevenBitAddress>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
        match self {
            BmeError::WriteReadError(e) => f.debug_tuple("WriteReadError").field(e).finish(),
            BmeError::WriteError(e) => f.debug_tuple("WriteError").field(e).finish(),
            BmeError::UnexpectedChipId(chip_id) => f
                .debug_tuple("Got unimplemented chip id: ")
                .field(chip_id)
                .finish(),
            BmeError::MeasuringTimeOut => f
                .debug_tuple("Timed out while waiting for new measurement values. Either no new data or the sensor took unexpectedly long to finish measuring.").finish()
        }
    }
}
