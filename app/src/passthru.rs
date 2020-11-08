
use J2534Common::*;
use libc;
use libloading::{Library, Symbol};
use std::ffi::*;
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};

lazy_static! {
    pub static ref DRIVER: Arc<RwLock<Option<PassthruDrv>>> = Arc::new(RwLock::new(None));
}

#[cfg(unix)]
use serde_json;

#[cfg(windows)]
use winreg::enums::*;

#[cfg(windows)]
use winreg::{RegKey, RegValue};

type Result<T> = std::result::Result<T, J2534Common::PassthruError>;

type PassThruOpenFn = unsafe extern fn(name: *const libc::c_void, device_id: *mut u32) -> u32;

type PassThruCloseFn = unsafe extern fn(device_id: u32) -> i32;

type PassThruConnectFn = unsafe extern fn(device_id: u32, protocol_id: u32, flags: u32, baudrate: u32, channel_id: *mut u32) -> i32;

type PassThruDisconnectFn = unsafe extern fn(channel_id: u32) -> i32;

type PassThruReadMsgsFn = unsafe extern fn(channel_id: u32, msgs: *mut PASSTHRU_MSG, num_msgs: *mut u32, timeout: u32) -> i32;

type PassThruWriteMsgsFn = unsafe extern fn(channel_id: u32, msgs: *mut PASSTHRU_MSG, num_msgs: *mut u32, timeout: u32) -> i32;

type PassThruStartPeriodicMsgFn = unsafe extern fn(channel_id: u32, msg: *const PASSTHRU_MSG, msg_id: *mut u32, time_interval: u32) -> i32;

type PassThruStopPeriodicMsgFn = unsafe extern fn(channel_id: u32, msg_id: u32) -> i32;

type PassThruStartMsgFilterFn = unsafe extern fn(channel_id: u32, filter_type: u32, m_msg: *const PASSTHRU_MSG, p_msg: *const PASSTHRU_MSG, fc_msg: *const PASSTHRU_MSG, filter_id: *mut u32) -> i32;

type PassThruStopMsgFilterFn = unsafe extern fn(channel_id: u32, filter_id: u32) -> i32;

type PassThruSetProgrammingVoltageFn = unsafe extern fn(device_id: u32, pin_number: u32, voltage: u32) -> i32;

type PassThruReadVersionFn = unsafe extern fn(device_id: u32, firmware_version: *mut libc::c_char, dll_version: *mut libc::c_char, api_version: *mut libc::c_char) -> i32;

type PassThruGetLastErrorFn = unsafe extern fn(error_description: *mut libc::c_char) -> i32;

type PassThruIoctlFn = unsafe extern fn(handle_id: u32, ioctl_id: u32, input: *mut libc::c_void, output: *mut libc::c_void) -> i32;

#[derive(Debug, Serialize, Deserialize)]
pub struct DrvVersion {
    /// Library (DLL) Version
    pub dll_version: String,
    /// Passthru API Version (Only V04.04 is supported currently!)
    pub api_version: String,
    /// Device Firmware version
    pub fw_version: String
}

#[derive(Debug, Clone)]
pub struct PassthruDrv {
    /// Loaded library to interface with the device
    lib: Arc<libloading::Library>,
    open_fn: PassThruOpenFn,
    close_fn: PassThruCloseFn,
    connect_fn: PassThruConnectFn,
    disconnect_fn: PassThruDisconnectFn,
    read_msg_fn: PassThruReadMsgsFn,
    write_msg_fn: PassThruWriteMsgsFn,
    start_periodic_fn: PassThruStartPeriodicMsgFn,
    stop_periodic_fn: PassThruStopPeriodicMsgFn,
    start_filter_fn: PassThruStartMsgFilterFn,
    stop_filter_fn: PassThruStopMsgFilterFn,
    set_prog_v_fn: PassThruSetProgrammingVoltageFn,
    get_last_err_fn: PassThruGetLastErrorFn,
    ioctl_fn: PassThruIoctlFn,
    read_version_fn: PassThruReadVersionFn,
}

impl PassthruDrv {
    pub fn load_lib(path: String) -> std::result::Result<PassthruDrv, libloading::Error> {
        unsafe {
            let lib = Library::new(path)?;
            let open_fn = *lib.get::<PassThruOpenFn>(b"PassThruOpen\0")?.into_raw();
            let close_fn = *lib.get::<PassThruCloseFn>(b"PassThruClose\0")?.into_raw();
            let connect_fn = *lib.get::<PassThruConnectFn>(b"PassThruConnect\0")?.into_raw();
            let disconnect_fn = *lib.get::<PassThruDisconnectFn>(b"PassThruDisconnect\0")?.into_raw();
            let read_msg_fn = *lib.get::<PassThruReadMsgsFn>(b"PassThruReadMsgs\0")?.into_raw();
            let write_msg_fn = *lib.get::<PassThruWriteMsgsFn>(b"PassThruWriteMsgs\0")?.into_raw();
            let start_periodic_fn = *lib.get::<PassThruStartPeriodicMsgFn>(b"PassThruStartPeriodicMsg\0")?.into_raw();
            let stop_periodic_fn = *lib.get::<PassThruStopPeriodicMsgFn>(b"PassThruStopPeriodicMsg\0")?.into_raw();
            let start_filter_fn = *lib.get::<PassThruStartMsgFilterFn>(b"PassThruStartMsgFilter\0")?.into_raw();
            let stop_filter_fn = *lib.get::<PassThruStopMsgFilterFn>(b"PassThruStopMsgFilter\0")?.into_raw();
            let set_prog_v_fn = *lib.get::<PassThruSetProgrammingVoltageFn>(b"PassThruSetProgrammingVoltage\0")?.into_raw();
            let get_last_err_fn = *lib.get::<PassThruGetLastErrorFn>(b"PassThruGetLastError\0")?.into_raw();
            let ioctl_fn = *lib.get::<PassThruIoctlFn>(b"PassThruIoctl\0")?.into_raw();
            let read_version_fn = *lib.get::<PassThruReadVersionFn>(b"PassThruReadVersion\0")?.into_raw();

            Ok(PassthruDrv {
                lib: Arc::new(lib),
                open_fn,
                close_fn,
                connect_fn,
                disconnect_fn,
                read_msg_fn,
                write_msg_fn,
                start_periodic_fn,
                stop_periodic_fn,
                start_filter_fn,
                stop_filter_fn,
                set_prog_v_fn,
                get_last_err_fn,
                ioctl_fn,
                read_version_fn
            })
        }
    }

    pub fn open(&self) -> Result<u32> {
        let mut test = "Dev1".to_string();
        let mut v = 0;
        let mut name = unsafe { test.as_bytes_mut() };
        let res = unsafe {
            (&self.open_fn)(
                name.as_mut_ptr() as *mut libc::c_void,
                &mut v
            )
        };
        if res == PassthruError::STATUS_NOERROR as u32 {
            return Ok(v)
        } else {
            Err(PassthruError::ERR_TIMEOUT)
        }
    }

    pub fn get_version(&self, dev_id: u32) ->  Result<DrvVersion> {
        println!("PT -> CALLING GET_VERSION");
        let mut firmware_version: [u8; 80] = [0; 80];
        let mut dll_version: [u8; 80] = [0; 80];
        let mut api_version: [u8; 80] = [0; 80];
        let res = unsafe {
            (&self.read_version_fn)(
                dev_id,
                firmware_version.as_mut_ptr() as *mut libc::c_char,
                dll_version.as_mut_ptr() as *mut libc::c_char,
                api_version.as_mut_ptr() as *mut libc::c_char,
            )
        };
        if res == PassthruError::STATUS_NOERROR as i32 {
            return Ok(DrvVersion {
                dll_version: String::from_utf8(dll_version.to_vec()).unwrap(),
                api_version: String::from_utf8(api_version.to_vec()).unwrap(),
                fw_version: String::from_utf8(firmware_version.to_vec()).unwrap()
            })
        } else {
            Err(PassthruError::ERR_TIMEOUT)
        }
    }

    pub fn ioctl(&self, dev_id: u32, ioctl_id: IoctlID, input: *mut c_void, output: *mut c_void) -> i32 {
        unsafe {
            (&self.ioctl_fn)(
                dev_id,
                ioctl_id as u32,
                input,
                output
            )
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PassthruDevice {
    /// Driver struct
    //drv: PassthruDrv,
    /// Driver path
    pub drv_path: String,

    /// Name of passthru device
    pub name: String,
    /// Vendor of passthru device
    pub vendor: String,
    
    /// Device CAN support?
    pub can: bool,
    /// Device ISO15765 (Over CAN) support
    pub iso15765: bool,

    /// Device ISO9141 support
    pub iso9141: bool,
    /// Device ISO14230 support (KWP2000)
    pub iso14230: bool,

    /// Device SCI A Transmission support
    pub sci_a_trans: bool,
    /// Device SCI A Engine support
    pub sci_a_engine: bool,

    /// Device SCI B Transmission support
    pub sci_b_trans: bool,
    /// Device SCI B Engine support
    pub sci_b_engine: bool,

    /// Device J1850VPW support
    pub j1850vpw: bool,
    /// Device J1850PWM support
    pub j1850pwm: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoadDeviceError {
    /// Device entry contains no name
    NoName,
    /// Device entry contains no vendor
    NoVendor,
    /// Device entry contains no library
    NoFunctionLib,
    /// Permission error reading JSON (UNIX Only)
    NoPermission,
    /// Malformed JSON (UNIX Only)
    InvalidJSON,
    /// Unknown IO Error
    IoError(String),
    /// library load failed
    LibLoadError(String),
    /// No Passthru devices found
    NoDeviceFound
}

impl LoadDeviceError {
    pub fn get_err_desc(&self) -> String {
        match &self {
            LoadDeviceError::NoName => "No device name attribute".to_string(),
            LoadDeviceError::NoVendor => "No device vendor attribute".to_string(),
            LoadDeviceError::NoFunctionLib => "No device function library attribute".to_string(),
            LoadDeviceError::NoPermission => "No permission reading device attributes".to_string(),
            LoadDeviceError::InvalidJSON => "Device JSON File malformed".to_string(),
            LoadDeviceError::IoError(e) => format!("IO Error: {}", e),
            LoadDeviceError::LibLoadError(e) => format!("Library load error: {}", e),
            LoadDeviceError::NoDeviceFound => "No devices found on machine".to_string()
        }
    }
}

pub type DeviceError<T> = std::result::Result<T, LoadDeviceError>;

impl PassthruDevice {
    
    #[cfg(unix)]
    /// Finds all devices present in /usr/share/passthru/*.jsonS
    pub fn find_all() -> DeviceError<Vec<PassthruDevice>> {
        return match std::fs::read_dir("/usr/share/passthru") {
            Ok(list) => {
                // Read Dir into vector of files
                let dev_list: Vec<PassthruDevice> = list.into_iter()
                // Remove files that cannot be read
                .filter_map(|p| p.ok())
                // Filter any files that are not json files
                .filter(|p| p.file_name().to_str().unwrap().ends_with(".json"))
                // Attempt to read a PassthruDevice from each json file found
                .map(|p| PassthruDevice::read_device(&p.path()))
                // Keep Oks that were found, any entries that ended with errors are discarded
                .filter_map(|s| s.ok())
                // Convert result into vector
                .collect();
                
                match dev_list.is_empty() {
                    true => Err(LoadDeviceError::NoDeviceFound),
                    false => Ok(dev_list)
                }
            }
            Err(e) => Err(LoadDeviceError::IoError(e.to_string()))
        }
    }

    #[cfg(windows)]
    /// Finds all devices present in /usr/share/passthru/*.json
    pub fn find_all() -> DeviceError<Vec<PassthruDevice>> {
        let reg = match RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\WOW6432Node\\PassThruSupport.04.04") {
            Ok(r) => r,
            Err(x) => return Err(LoadDeviceError::IoError(x.to_string()))
        };

        let dev_list: Vec<PassthruDevice> = reg.enum_keys()
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|key| reg.open_subkey(key))
            .map(|x| PassthruDevice::read_device(&x.unwrap()))
            .filter_map(|d| d.ok())
            .collect();

        match dev_list.is_empty() {
            true => Err(LoadDeviceError::NoDeviceFound),
            false => Ok(dev_list)
        }
    }

    #[cfg(unix)]
    #[inline]
    pub fn read_bool(j: &serde_json::Value, s: &str) -> bool {
        match j[s].as_bool() {
            Some(x) => x,
            None => false
        }
    }

    #[cfg(unix)]
    /// Loads Unix passthru JSON into a passthru device
    pub fn read_device(p: &std::path::PathBuf) -> DeviceError<PassthruDevice> {
        return if let Ok(s) = std::fs::read_to_string(&p) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(s.as_str()) {
                let lib = match json["FUNCTION_LIB"].as_str() {
                    Some(s) => s,
                    None => return Err(LoadDeviceError::NoFunctionLib)
                };
                let name = match json["NAME"].as_str() {
                    Some(s) => s,
                    None => return Err(LoadDeviceError::NoName)
                };
                let vend = match json["VENDOR"].as_str() {
                    Some(s) => s,
                    None => return Err(LoadDeviceError::NoVendor)
                };
                // Load library to ensure it exists
                let driver = match Library::new(lib.clone()) {
                    Ok(l) => l,
                    Err(x) => {
                        return Err(LoadDeviceError::LibLoadError(x.to_string()))
                    }
                };
                // We can unload it, will re-load once OVD starts
                driver.close().unwrap();
                Ok(PassthruDevice{
                    drv_path: String::from(lib),
                    name: String::from(name),
                    vendor: String::from(vend),
                    can: PassthruDevice::read_bool(&json, "CAN"),
                    iso15765: PassthruDevice::read_bool(&json, "ISO15765"),
                    iso14230: PassthruDevice::read_bool(&json, "ISO14230"),
                    iso9141: PassthruDevice::read_bool(&json, "ISO9141"),
                    j1850pwm: PassthruDevice::read_bool(&json, "J1850PWM"),
                    j1850vpw: PassthruDevice::read_bool(&json, "J1850VPW"),
                    sci_a_engine: PassthruDevice::read_bool(&json, "SCI_A_ENGINE"),
                    sci_a_trans: PassthruDevice::read_bool(&json, "SCN_A_TRANS"),
                    sci_b_engine: PassthruDevice::read_bool(&json, "SCI_B_ENGINE"),
                    sci_b_trans: PassthruDevice::read_bool(&json, "SCI_B_TRANS"),
                    //drv: driver
                })
            } else {
                return Err(LoadDeviceError::InvalidJSON)
            }
        } else {
            Err(LoadDeviceError::NoPermission)
        }
    }

    #[cfg(windows)]
    #[inline]
    fn read_bool(k: &RegKey, name: &str) -> bool {
        let val: u32 = match k.get_value(name.to_string()) {
            Ok(b) => b,
            Err(_) => return false
        };
        return val != 0
    }

    #[cfg(windows)]
    /// Reads a device entry from windows registry and loads it into a Passthru device
    pub fn read_device(r: &RegKey) -> DeviceError<PassthruDevice> {
        let lib: String = match r.get_value("FunctionLibrary") {
            Ok (s) => s,
            Err(_) => return Err(LoadDeviceError::NoFunctionLib)
        };

        let name: String = match r.get_value("Name") {
            Ok (s) => s,
            Err(_) => return Err(LoadDeviceError::NoName)
        };

        let vend: String = match r.get_value("Vendor") {
            Ok (s) => s,
            Err(_) => return Err(LoadDeviceError::NoVendor)
        };

        //log::logDebug("Read_Device", format!("Found device {} by {}. Library: {}", name, vend, lib));

        // Load library to ensure it exists
        let driver = match Library::new(lib.clone()) {
            Ok(l) => l,
            Err(x) => {
                //log::logError("Read_Device", format!("Cannot load DLL! ({:?})", x));
                return Err(LoadDeviceError::LibLoadError(x.to_string()))
            }
        };
        // We can unload it, will re-load once OVD starts
        driver.close().unwrap();

        Ok(PassthruDevice{
            drv_path: String::from(lib),
            name: String::from(name),
            vendor: String::from(vend),
            can: PassthruDevice::read_bool(&r, "CAN"),
            iso15765: PassthruDevice::read_bool(&r, "ISO15765"),
            iso14230: PassthruDevice::read_bool(&r, "ISO14230"),
            iso9141: PassthruDevice::read_bool(&r, "ISO9141"),
            j1850pwm: PassthruDevice::read_bool(&r, "J1850PWM"),
            j1850vpw: PassthruDevice::read_bool(&r, "J1850VPW"),
            sci_a_engine: PassthruDevice::read_bool(&r, "SCI_A_ENGINE"),
            sci_a_trans: PassthruDevice::read_bool(&r, "SCN_A_TRANS"),
            sci_b_engine: PassthruDevice::read_bool(&r, "SCI_B_ENGINE"),
            sci_b_trans: PassthruDevice::read_bool(&r, "SCI_B_TRANS"),
            //drv: driver
        })
    }
}