use rcc::{drive, DriverError};

fn main() -> Result<(), DriverError> {
        drive()?;
        std::process::exit(0);
}
