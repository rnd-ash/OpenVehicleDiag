use J2534Common::*;
use libc;
use libloading::{Library, Symbol};
use std::ffi::*;

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

#[derive(Debug)]
pub struct PassthruDrv {
    lib: libloading::Library,
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

#[derive(Debug)]
pub struct DrvVersion {
    pub dll_version: String,
    pub api_version: String,
    pub fw_version: String
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
                lib,
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

    pub fn get_version(&self) ->  Result<DrvVersion> {
        println!("PT -> CALLING GET_VERSION");
        let mut firmware_version: [u8; 80] = [0; 80];
        let mut dll_version: [u8; 80] = [0; 80];
        let mut api_version: [u8; 80] = [0; 80];
        let res = unsafe {
            (&self.read_version_fn)(
                0,
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
}

