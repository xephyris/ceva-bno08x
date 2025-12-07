use core::convert::Infallible;

pub enum CSPinError {
    PinError,
}

impl From<Infallible> for CSPinError {
    fn from(_: Infallible) -> Self {
        CSPinError::PinError
    }
}
