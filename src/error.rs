use core::fmt;

/// All possible errors
#[derive(Debug)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum BmeError<E: fmt::Debug> {
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

impl<E> Clone for BmeError<E>
where
    E: defmt::Format + fmt::Debug + Clone,
{
    fn clone(&self) -> Self {
        match self {
            BmeError::WriteError(e) => BmeError::WriteError(e.clone()),
            BmeError::WriteReadError(e) => BmeError::WriteReadError(e.clone()),
            BmeError::UnexpectedChipId(id) => BmeError::UnexpectedChipId(*id),
            BmeError::MeasuringTimeOut => BmeError::MeasuringTimeOut,
        }
    }
}

#[cfg(feature = "postcard")]
impl<E> postcard::experimental::max_size::MaxSize for BmeError<E>
where
    E: postcard::experimental::max_size::MaxSize + fmt::Debug,
{
    const POSTCARD_MAX_SIZE: usize = 1 + E::POSTCARD_MAX_SIZE;
}

impl<E> defmt::Format for BmeError<E>
where
    E: defmt::Format + fmt::Debug,
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

impl<E> Eq for BmeError<E> where E: Eq + defmt::Format + fmt::Debug {}

impl<E> PartialEq for BmeError<E>
where
    E: PartialEq + defmt::Format + fmt::Debug,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::WriteReadError(e), Self::WriteReadError(e2)) => e == e2,
            (Self::WriteError(e), Self::WriteError(e2)) => e == e2,
            (Self::UnexpectedChipId(chip_id), Self::UnexpectedChipId(chip_id2)) => {
                chip_id == chip_id2
            }
            (Self::MeasuringTimeOut, Self::MeasuringTimeOut) => true,
            (_, _) => false,
        }
    }
}
