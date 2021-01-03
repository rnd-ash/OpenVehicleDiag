use std::ffi::c_void;

use J2534Common::PassthruError;

use crate::passthru::{PassthruDevice, PassthruDrv};

struct ComServer {
    device_info: PassthruDevice,
    device_driver: PassthruDrv,
    device_id: u32,
}

pub struct Error {
    err_type: PassthruError,
    err_desc: Option<String>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.err_desc {
            Some(desc) => write!(f, "{:?}({})", self.err_type, desc),
            None => write!(f, "{:?}", self.err_type),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl ComServer {
    fn new(info: PassthruDevice, driver: PassthruDrv, idx: u32) -> Self {
        Self {
            device_info: info,
            device_driver: driver,
            device_id: idx,
        }
    }

    fn convert_error(&self, e: PassthruError) -> Error {
        if e == PassthruError::ERR_FAILED {
            // Try and get the description from the device as to what went wrong
            if let Ok(desc) = self.device_driver.get_last_error() {
                Error {
                    err_type: e,
                    err_desc: Some(desc),
                }
            } else {
                // Err failed was thrown, but driver does not have a description
                Error {
                    err_type: e,
                    err_desc: Some(String::from("Unknown driver error")),
                }
            }
        } else {
            Error {
                err_type: e,
                err_desc: None,
            }
        }
    }

    /// Returns the battery voltage of the vehicle
    pub fn get_batt_voltage(&self) -> Result<f32> {
        let mut output: u32 = 0;
        self.device_driver
            .ioctl(
                self.device_id,
                J2534Common::IoctlID::READ_VBATT,
                std::ptr::null_mut(),
                (&mut output) as *mut _ as *mut c_void,
            )
            .map_err(|e| self.convert_error(e))
            .map(|_| output as f32)
    }
}
