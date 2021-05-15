use j2534_rust::FilterType::FLOW_CONTROL_FILTER;
use j2534_rust::*;
use lazy_static::lazy_static;
use libloading::Library;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::{ffi::*, fmt};

lazy_static! {
    pub static ref DRIVER: Arc<RwLock<Option<PassthruDrv>>> = Arc::new(RwLock::new(None));
}

#[cfg(windows)]
use winreg::enums::*;

#[cfg(windows)]
use winreg::{RegKey, RegValue};

/// Result which contains a PASSTHRU_ERROR in it's Err() variant
pub type Result<T> = std::result::Result<T, j2534_rust::PassthruError>;

type PassThruOpenFn =
    unsafe extern "stdcall" fn(name: *const libc::c_void, device_id: *mut u32) -> i32;
type PassThruCloseFn = unsafe extern "stdcall" fn(device_id: u32) -> i32;
type PassThruConnectFn = unsafe extern "stdcall" fn(
    device_id: u32,
    protocol_id: u32,
    flags: u32,
    baudrate: u32,
    channel_id: *mut u32,
) -> i32;
type PassThruDisconnectFn = unsafe extern "stdcall" fn(channel_id: u32) -> i32;
type PassThruReadMsgsFn = unsafe extern "stdcall" fn(
    channel_id: u32,
    msgs: *mut PASSTHRU_MSG,
    num_msgs: *mut u32,
    timeout: u32,
) -> i32;
type PassThruWriteMsgsFn = unsafe extern "stdcall" fn(
    channel_id: u32,
    msgs: *mut PASSTHRU_MSG,
    num_msgs: *mut u32,
    timeout: u32,
) -> i32;
type PassThruStartPeriodicMsgFn = unsafe extern "stdcall" fn(
    channel_id: u32,
    msg: *const PASSTHRU_MSG,
    msg_id: *mut u32,
    time_interval: u32,
) -> i32;
type PassThruStopPeriodicMsgFn = unsafe extern "stdcall" fn(channel_id: u32, msg_id: u32) -> i32;
type PassThruStartMsgFilterFn = unsafe extern "stdcall" fn(
    channel_id: u32,
    filter_type: u32,
    m_msg: *const PASSTHRU_MSG,
    p_msg: *const PASSTHRU_MSG,
    fc_msg: *const PASSTHRU_MSG,
    filter_id: *mut u32,
) -> i32;
type PassThruStopMsgFilterFn = unsafe extern "stdcall" fn(channel_id: u32, filter_id: u32) -> i32;
type PassThruSetProgrammingVoltageFn =
    unsafe extern "stdcall" fn(device_id: u32, pin_number: u32, voltage: u32) -> i32;
type PassThruReadVersionFn = unsafe extern "stdcall" fn(
    device_id: u32,
    firmware_version: *mut libc::c_char,
    dll_version: *mut libc::c_char,
    api_version: *mut libc::c_char,
) -> i32;
type PassThruGetLastErrorFn =
    unsafe extern "stdcall" fn(error_description: *mut libc::c_char) -> i32;
type PassThruIoctlFn = unsafe extern "stdcall" fn(
    handle_id: u32,
    ioctl_id: u32,
    input: *mut libc::c_void,
    output: *mut libc::c_void,
) -> i32;

#[derive(Debug, Serialize, Deserialize)]
pub struct DrvVersion {
    /// Library (DLL) Version
    pub dll_version: String,
    /// Passthru API Version (Only V04.04 is supported currently!)
    pub api_version: String,
    /// Device Firmware version
    pub fw_version: String,
}

#[derive(Clone)]
pub struct PassthruDrv {
    /// Loaded library to interface with the device
    lib: Arc<libloading::Library>,
    /// Is the device currently connected?
    is_connected: bool,
    /// Open device connection
    open_fn: PassThruOpenFn,
    /// Close device connection
    close_fn: PassThruCloseFn,
    /// Connect a communication channel
    connect_fn: PassThruConnectFn,
    /// Disconnect a communication channel
    disconnect_fn: PassThruDisconnectFn,
    /// Read messages from a communication channel
    read_msg_fn: PassThruReadMsgsFn,
    /// Write messages to a communication channel
    write_msg_fn: PassThruWriteMsgsFn,
    /// Start a periodic message
    start_periodic_fn: PassThruStartPeriodicMsgFn,
    /// Stop a periodic message
    stop_periodic_fn: PassThruStopPeriodicMsgFn,
    /// Start a filter on a channel
    start_filter_fn: PassThruStartMsgFilterFn,
    /// Stop a filter on a channel
    stop_filter_fn: PassThruStopMsgFilterFn,
    /// Set programming voltage
    set_prog_v_fn: PassThruSetProgrammingVoltageFn,
    /// Get the last driver error description if ERR_FAILED
    get_last_err_fn: PassThruGetLastErrorFn,
    /// IOCTL
    ioctl_fn: PassThruIoctlFn,
    /// Get driver details
    read_version_fn: PassThruReadVersionFn,
}

impl fmt::Debug for PassthruDrv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PassthruDrv")
            .field("is_connected", &self.is_connected)
            .field("library", &self.lib)
            .finish()
    }
}

#[inline(always)]
/// Function to reduce boilerplate code with returning a Result
fn ret_res<T>(res: i32, ret: T) -> Result<T> {
    match res {
        0 => Ok(ret),
        _ => Err(PassthruError::from_raw(res as u32).unwrap()),
    }
}

impl PassthruDrv {
    pub fn load_lib(path: String) -> std::result::Result<PassthruDrv, libloading::Error> {
        let lib = unsafe { Library::new(path)? };
        unsafe {
            let open_fn = *lib.get::<PassThruOpenFn>(b"PassThruOpen\0")?.into_raw();
            let close_fn = *lib.get::<PassThruCloseFn>(b"PassThruClose\0")?.into_raw();
            let connect_fn = *lib
                .get::<PassThruConnectFn>(b"PassThruConnect\0")?
                .into_raw();
            let disconnect_fn = *lib
                .get::<PassThruDisconnectFn>(b"PassThruDisconnect\0")?
                .into_raw();
            let read_msg_fn = *lib
                .get::<PassThruReadMsgsFn>(b"PassThruReadMsgs\0")?
                .into_raw();
            let write_msg_fn = *lib
                .get::<PassThruWriteMsgsFn>(b"PassThruWriteMsgs\0")?
                .into_raw();
            let start_periodic_fn = *lib
                .get::<PassThruStartPeriodicMsgFn>(b"PassThruStartPeriodicMsg\0")?
                .into_raw();
            let stop_periodic_fn = *lib
                .get::<PassThruStopPeriodicMsgFn>(b"PassThruStopPeriodicMsg\0")?
                .into_raw();
            let start_filter_fn = *lib
                .get::<PassThruStartMsgFilterFn>(b"PassThruStartMsgFilter\0")?
                .into_raw();
            let stop_filter_fn = *lib
                .get::<PassThruStopMsgFilterFn>(b"PassThruStopMsgFilter\0")?
                .into_raw();
            let set_prog_v_fn = *lib
                .get::<PassThruSetProgrammingVoltageFn>(b"PassThruSetProgrammingVoltage\0")?
                .into_raw();
            let get_last_err_fn = *lib
                .get::<PassThruGetLastErrorFn>(b"PassThruGetLastError\0")?
                .into_raw();
            let ioctl_fn = *lib.get::<PassThruIoctlFn>(b"PassThruIoctl\0")?.into_raw();
            let read_version_fn = *lib
                .get::<PassThruReadVersionFn>(b"PassThruReadVersion\0")?
                .into_raw();

            Ok(PassthruDrv {
                lib: Arc::new(lib),
                is_connected: false,
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
                read_version_fn,
            })
        }
    }

    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    //type PassThruOpenFn = unsafe extern "stdcall" fn(name: *const libc::c_void, device_id: *mut u32) -> i32;
    pub fn open(&mut self) -> Result<u32> {
        let mut id: u32 = 0;
        let name = CString::new("test").unwrap();
        let res =
            unsafe { (&self.open_fn)(name.as_ptr() as *const libc::c_void, &mut id as *mut u32) };
        if res == 0x00 {
            self.is_connected = true;
        }
        ret_res(res, id)
    }

    //type PassThruCloseFn = unsafe extern "stdcall" fn(device_id: u32) -> i32;
    pub fn close(&mut self, dev_id: u32) -> Result<()> {
        let res = unsafe { (&self.close_fn)(dev_id) };
        if res == 0x00 {
            self.is_connected = false;
        }
        ret_res(res, ())
    }

    // type PassThruWriteMsgsFn = unsafe extern "stdcall" fn(channel_id: u32, msgs: *mut PASSTHRU_MSG, num_msgs: *mut u32, timeout: u32) -> i32;
    pub fn write_messages(
        &self,
        channel_id: u32,
        msgs: &mut [PASSTHRU_MSG],
        timeout: u32,
    ) -> Result<usize> {
        if msgs.is_empty() {
            // No messages? Just tell application everything is OK
            return Ok(0);
        }
        let mut msg_count: u32 = msgs.len() as u32;
        let res = unsafe {
            (&self.write_msg_fn)(
                channel_id,
                msgs.as_mut_ptr(),
                &mut msg_count as *mut u32,
                timeout,
            )
        };
        ret_res(res, msg_count as usize)
    }

    //type PassThruReadMsgsFn = unsafe extern "stdcall" fn(channel_id: u32, msgs: *mut PASSTHRU_MSG, num_msgs: *mut u32, timeout: u32) -> i32;
    pub fn read_messages(
        &self,
        channel_id: u32,
        max_msgs: u32,
        timeout: u32,
    ) -> Result<Vec<PASSTHRU_MSG>> {
        let mut msg_count: u32 = max_msgs;
        // Create a blank array of empty passthru messages according to the max we should read
        let mut write_array: Vec<PASSTHRU_MSG> = vec![
            PASSTHRU_MSG {
                protocol_id: 0,
                rx_status: 0,
                tx_flags: 0,
                timestamp: 0,
                data_size: 0,
                extra_data_size: 0,
                data: [0; 4128]
            };
            max_msgs as usize
        ];

        let res = unsafe {
            (&self.read_msg_fn)(
                channel_id,
                write_array.as_mut_ptr() as *mut PASSTHRU_MSG,
                &mut msg_count as *mut u32,
                timeout,
            )
        };
        if res == PassthruError::ERR_BUFFER_EMPTY as i32 && msg_count != 0 {
            write_array.truncate(msg_count as usize);
            return ret_res(0x00, write_array);
        }
        if msg_count != max_msgs {
            // Trim the output vector to size
            write_array.truncate(msg_count as usize);
        }
        ret_res(res, write_array)
    }

    //type PassThruReadVersionFn = unsafe extern "stdcall" fn(device_id: u32, firmware_version: *mut libc::c_char, dll_version: *mut libc::c_char, api_version: *mut libc::c_char) -> i32;
    pub fn get_version(&self, dev_id: u32) -> Result<DrvVersion> {
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
        unsafe {
            ret_res(
                res,
                DrvVersion {
                    api_version: CStr::from_ptr(api_version.as_ptr() as *const libc::c_char)
                        .to_str()
                        .unwrap()
                        .to_string(),
                    dll_version: CStr::from_ptr(dll_version.as_ptr() as *const libc::c_char)
                        .to_str()
                        .unwrap()
                        .to_string(),
                    fw_version: CStr::from_ptr(firmware_version.as_ptr() as *const libc::c_char)
                        .to_str()
                        .unwrap()
                        .to_string(),
                },
            )
        }
    }

    //type PassThruGetLastErrorFn = unsafe extern "stdcall" fn(error_description: *mut libc::c_char) -> i32;
    pub fn get_last_error(&self) -> Result<String> {
        let mut err: [u8; 80] = [0; 80];
        let res = unsafe { (&self.get_last_err_fn)(err.as_mut_ptr() as *mut libc::c_char) };
        ret_res(res, String::from_utf8(err.to_vec()).unwrap())
    }

    //type PassThruIoctlFn = unsafe extern "stdcall" fn(handle_id: u32, ioctl_id: u32, input: *mut libc::c_void, output: *mut libc::c_void) -> i32;
    pub fn ioctl(
        &self,
        handle_id: u32,
        ioctl_id: IoctlID,
        input: *mut c_void,
        output: *mut c_void,
    ) -> Result<()> {
        let res = unsafe { (&self.ioctl_fn)(handle_id, ioctl_id as u32, input, output) };
        ret_res(res, ())
    }

    //type PassThruConnectFn = unsafe extern "stdcall" fn(device_id: u32, protocol_id: u32, flags: u32, baudrate: u32, channel_id: *mut u32) -> i32;
    /// Returns channel ID
    pub fn connect(&self, dev_id: u32, protocol: Protocol, flags: u32, baud: u32) -> Result<u32> {
        let mut channel_id: u32 = 0;
        let res = unsafe {
            (&self.connect_fn)(
                dev_id,
                protocol as u32,
                flags as u32,
                baud,
                &mut channel_id as *mut u32,
            )
        };
        ret_res(res, channel_id)
    }

    //type PassThruDisconnectFn = unsafe extern "stdcall" fn(channel_id: u32) -> i32;
    pub fn disconnect(&self, channel_id: u32) -> Result<()> {
        ret_res(unsafe { (&self.disconnect_fn)(channel_id) }, ())
    }

    //type PassThruStartPeriodicMsgFn = unsafe extern "stdcall" fn(channel_id: u32, msg: *const PASSTHRU_MSG, msg_id: *mut u32, time_interval: u32) -> i32;
    /// Returns message ID
    #[allow(dead_code)]
    pub fn start_periodic_msg(
        &self,
        channel_id: u32,
        msg: &PASSTHRU_MSG,
        time_interval: u32,
    ) -> Result<u32> {
        let mut msg_id: u32 = 0;
        let res = unsafe {
            (&self.start_periodic_fn)(
                channel_id,
                msg as *const PASSTHRU_MSG,
                &mut msg_id as *mut u32,
                time_interval,
            )
        };
        ret_res(res, msg_id)
    }

    //type PassThruStopPeriodicMsgFn = unsafe extern "stdcall" fn(channel_id: u32, msg_id: u32) -> i32;
    #[allow(dead_code)]
    pub fn stop_periodic_msg(&self, channel_id: u32, msg_id: u32) -> Result<()> {
        ret_res(unsafe { (&self.stop_periodic_fn)(channel_id, msg_id) }, ())
    }

    //type PassThruStartMsgFilterFn = unsafe extern "stdcall" fn(channel_id: u32, filter_type: u32, m_msg: *const PASSTHRU_MSG, p_msg: *const PASSTHRU_MSG, fc_msg: *const PASSTHRU_MSG, filter_id: *mut u32) -> i32;
    /// Returns filter ID
    pub fn start_msg_filter(
        &self,
        channel_id: u32,
        filter_type: FilterType,
        mask: &PASSTHRU_MSG,
        pattern: &PASSTHRU_MSG,
        flow_control: Option<PASSTHRU_MSG>,
    ) -> Result<u32> {
        let tmp = filter_type as u32;
        if tmp == FLOW_CONTROL_FILTER as u32 && flow_control.is_none() {
            return Err(PassthruError::ERR_INVALID_FILTER_ID);
        }

        let mut filter_id: u32 = 0;
        let res = match flow_control.as_ref() {
            None => unsafe {
                (&self.start_filter_fn)(
                    channel_id,
                    tmp,
                    mask as *const PASSTHRU_MSG,
                    pattern as *const PASSTHRU_MSG,
                    std::ptr::null() as *const PASSTHRU_MSG,
                    &mut filter_id as *mut u32,
                )
            },
            Some(fc) => unsafe {
                (&self.start_filter_fn)(
                    channel_id,
                    tmp,
                    mask as *const PASSTHRU_MSG,
                    pattern as *const PASSTHRU_MSG,
                    fc as *const PASSTHRU_MSG,
                    &mut filter_id as *mut u32,
                )
            },
        };
        ret_res(res, filter_id)
    }

    //type PassThruStopMsgFilterFn = unsafe extern "stdcall" fn(channel_id: u32, filter_id: u32) -> i32;
    pub fn stop_msg_filter(&self, channel_id: u32, filter_id: u32) -> Result<()> {
        let res = unsafe { (&self.stop_filter_fn)(channel_id, filter_id) };
        match res {
            0 => Ok(()),
            _ => Err(PassthruError::from_raw(res as u32).unwrap()),
        }
    }

    //type PassThruSetProgrammingVoltageFn = unsafe extern "stdcall" fn(device_id: u32, pin_number: u32, voltage: u32) -> i32;
    #[allow(dead_code)]
    pub fn set_programming_voltage(&self, dev_id: u32, pin: u32, voltage: u32) -> Result<()> {
        ret_res(unsafe { (&self.set_prog_v_fn)(dev_id, pin, voltage) }, ())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PassthruDevice {
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
    pub j1850pwm: bool,
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
    NoDeviceFound,
}

impl LoadDeviceError {
    #[allow(dead_code)]
    pub fn get_err_desc(&self) -> String {
        match &self {
            LoadDeviceError::NoName => "No device name attribute".to_string(),
            LoadDeviceError::NoVendor => "No device vendor attribute".to_string(),
            LoadDeviceError::NoFunctionLib => "No device function library attribute".to_string(),
            LoadDeviceError::NoPermission => "No permission reading device attributes".to_string(),
            LoadDeviceError::InvalidJSON => "Device JSON File malformed".to_string(),
            LoadDeviceError::IoError(e) => format!("IO Error: {}", e),
            LoadDeviceError::LibLoadError(e) => format!("Library load error: {}", e),
            LoadDeviceError::NoDeviceFound => "No devices found on machine".to_string(),
        }
    }
}

pub type DeviceError<T> = std::result::Result<T, LoadDeviceError>;

impl PassthruDevice {
    #[cfg(unix)]
    /// Finds all devices present in /usr/share/passthru/*.jsonS
    pub fn find_all() -> DeviceError<Vec<PassthruDevice>> {
        return match std::fs::read_dir(shellexpand::tilde("~/.passthru").to_string()) {
            Ok(list) => {
                // Read Dir into vector of files
                let dev_list: Vec<PassthruDevice> = list
                    .into_iter()
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
                    false => Ok(dev_list),
                }
            }
            Err(e) => Err(LoadDeviceError::IoError(e.to_string())),
        };
    }

    #[cfg(windows)]
    pub fn find_all() -> DeviceError<Vec<PassthruDevice>> {
        let reg = match RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\WOW6432Node\\PassThruSupport.04.04")
        {
            Ok(r) => r,
            Err(x) => return Err(LoadDeviceError::IoError(x.to_string())),
        };
        let dev_list: Vec<PassthruDevice> = reg
            .enum_keys()
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|key| reg.open_subkey(key))
            .map(|x| PassthruDevice::read_device(&x.unwrap()))
            .filter_map(|d| d.ok())
            .collect();

        match dev_list.is_empty() {
            true => Err(LoadDeviceError::NoDeviceFound),
            false => Ok(dev_list),
        }
    }

    #[cfg(unix)]
    #[inline]
    pub fn read_bool(j: &serde_json::Value, s: &str) -> bool {
        j[s].as_bool().unwrap_or(false)
    }

    #[cfg(unix)]
    /// Loads Unix passthru JSON into a passthru device
    pub fn read_device(p: &std::path::PathBuf) -> DeviceError<PassthruDevice> {
        return if let Ok(s) = std::fs::read_to_string(&p) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(s.as_str()) {
                let lib = match json["FUNCTION_LIB"].as_str() {
                    Some(s) => shellexpand::tilde(s),
                    None => return Err(LoadDeviceError::NoFunctionLib),
                };
                let name = match json["NAME"].as_str() {
                    Some(s) => s,
                    None => return Err(LoadDeviceError::NoName),
                };
                let vend = match json["VENDOR"].as_str() {
                    Some(s) => s,
                    None => return Err(LoadDeviceError::NoVendor),
                };
                Ok(PassthruDevice {
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
                })
            } else {
                return Err(LoadDeviceError::InvalidJSON);
            }
        } else {
            Err(LoadDeviceError::NoPermission)
        };
    }

    #[cfg(windows)]
    #[inline]
    fn read_bool(k: &RegKey, name: &str) -> bool {
        let val: u32 = match k.get_value(name.to_string()) {
            Ok(b) => b,
            Err(_) => return false,
        };
        return val != 0;
    }

    #[cfg(windows)]
    /// Reads a device entry from windows registry and loads it into a Passthru device
    pub fn read_device(r: &RegKey) -> DeviceError<PassthruDevice> {
        let lib: String = match r.get_value("FunctionLibrary") {
            Ok(s) => s,
            Err(_) => return Err(LoadDeviceError::NoFunctionLib),
        };

        let name: String = match r.get_value("Name") {
            Ok(s) => s,
            Err(_) => return Err(LoadDeviceError::NoName),
        };

        let vend: String = match r.get_value("Vendor") {
            Ok(s) => s,
            Err(_) => return Err(LoadDeviceError::NoVendor),
        };

        Ok(PassthruDevice {
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
