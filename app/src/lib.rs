#[macro_use]
extern crate napi;
#[macro_use]
extern crate napi_derive;

use napi::{CallContext, Result, Status, Error, JsUnknown, JsFunction, JsUndefined, Module, JsNumber, JsNull, JsString};

use serde_json;
use serde::*;
use std::ffi::*;
use std::convert::{TryInto, TryFrom};

use J2534Common::*;
mod passthru;
use passthru::*;
use J2534Common::PassthruError::ERR_FAILED;
use crate::passthru::LoadDeviceError::LibLoadError;

#[derive(Debug, Serialize, Deserialize)]
struct LibError {
  err: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Device {
  dev_id: u32
}

#[derive(Debug, Serialize, Deserialize)]
struct Ok{}

#[derive(Debug, Serialize, Deserialize)]
struct Voltage {
  mv: u32
}

#[js_function]
pub fn get_device_list(mut ctx: CallContext) -> Result<JsUnknown> {
  Ok(match passthru::PassthruDevice::find_all() {
    Ok(dev) => { ctx.env.to_js_value(&dev)? },
    Err(e) =>  ctx.env.to_js_value(&LibError{ err: e.get_err_desc() })?,
  })
}

//type PassThruOpenFn = unsafe extern "stdcall" fn(name: *const libc::c_void, device_id: *mut u32) -> i32;
//type PassThruCloseFn = unsafe extern "stdcall" fn(device_id: u32) -> i32;
//type PassThruConnectFn = unsafe extern "stdcall" fn(device_id: u32, protocol_id: u32, flags: u32, baudrate: u32, channel_id: *mut u32) -> i32;
//type PassThruDisconnectFn = unsafe extern "stdcall" fn(channel_id: u32) -> i32;
//type PassThruReadMsgsFn = unsafe extern "stdcall" fn(channel_id: u32, msgs: *mut PASSTHRU_MSG, num_msgs: *mut u32, timeout: u32) -> i32;
//type PassThruWriteMsgsFn = unsafe extern "stdcall" fn(channel_id: u32, msgs: *mut PASSTHRU_MSG, num_msgs: *mut u32, timeout: u32) -> i32;
//type PassThruStartPeriodicMsgFn = unsafe extern "stdcall" fn(channel_id: u32, msg: *const PASSTHRU_MSG, msg_id: *mut u32, time_interval: u32) -> i32;
//type PassThruStopPeriodicMsgFn = unsafe extern "stdcall" fn(channel_id: u32, msg_id: u32) -> i32;
//type PassThruStartMsgFilterFn = unsafe extern "stdcall" fn(channel_id: u32, filter_type: u32, m_msg: *const PASSTHRU_MSG, p_msg: *const PASSTHRU_MSG, fc_msg: *const PASSTHRU_MSG, filter_id: *mut u32) -> i32;
//type PassThruStopMsgFilterFn = unsafe extern "stdcall" fn(channel_id: u32, filter_id: u32) -> i32;
//type PassThruSetProgrammingVoltageFn = unsafe extern "stdcall" fn(device_id: u32, pin_number: u32, voltage: u32) -> i32;
//type PassThruReadVersionFn = unsafe extern "stdcall" fn(device_id: u32, firmware_version: *mut libc::c_char, dll_version: *mut libc::c_char, api_version: *mut libc::c_char) -> i32;
//type PassThruGetLastErrorFn = unsafe extern "stdcall" fn(error_description: *mut libc::c_char) -> i32;
//type PassThruIoctlFn = unsafe extern "stdcall" fn(handle_id: u32, ioctl_id: u32, input: *mut libc::c_void, output: *mut libc::c_void) -> i32;



#[js_function(1)]
pub fn get_version(mut ctx: CallContext) -> Result<JsUnknown> {
  let idx: u32 = u32::try_from(ctx.get::<JsNumber>(0)?)?;
  if passthru::DRIVER.read().unwrap().is_none() {
    return ctx.env.to_js_value(&LibError{ err: "No driver!".to_string() });
  }
  Ok(match &passthru::DRIVER.write().unwrap().as_ref().unwrap().get_version(idx) {
    Ok(v) => ctx.env.to_js_value(v)?,
    Err(e) => ctx.env.to_js_value(&0)?,
  })
}

#[js_function(1)]
pub fn open(mut ctx: CallContext) -> Result<JsUnknown> {
  let v = ctx.get::<JsUnknown>(0)?;
  let deser: PassthruDevice = ctx.env.from_js_value(v)?;

  if passthru::DRIVER.read().unwrap().is_some() {
    return ctx.env.to_js_value(&LibError{ err: "Driver in use!".to_string() });
  }

  match PassthruDrv::load_lib(deser.drv_path) {
    Ok (d) => { // Library load OK?
      match d.open() { // Was the device able to be opened?
        Ok(idx) => { // Yes - Now we keep the library in static ref and return the idx
          DRIVER.write().unwrap().replace(d);
          ctx.env.to_js_value(&Device{ dev_id: idx })
        },
        Err(e) => {
          if e == ERR_FAILED { // Try to get last error
            let err_str = d.get_last_error();
            match err_str {
              Ok(str) => ctx.env.to_js_value(&LibError { err: format!("Operation failed. '{}'", str) }),
              Err(_) => ctx.env.to_js_value(&LibError { err: "Unknown error".to_string() }) // ERR_FAILED but no string??
            }
          } else {
            ctx.env.to_js_value(&LibError { err: e.to_string().to_string() })
          }
        }
      }
    }
    Err(x) => return ctx.env.to_js_value(&LibError{ err: x.to_string() })
  }
}

#[js_function(1)]
pub fn close(mut ctx: CallContext) -> Result<JsUnknown> {
  let idx: u32 = u32::try_from(ctx.get::<JsNumber>(0)?)?;
  if passthru::DRIVER.read().unwrap().is_none() {
    return ctx.env.to_js_value(&LibError{ err: format!("No library loaded") });
  }
  let res = passthru::DRIVER.write().unwrap().as_ref().unwrap().close(idx);
  match res {
    Ok(_) => {
      passthru::DRIVER.write().unwrap().take();
      ctx.env.to_js_value(&Ok {})
    },
    Err(e) => {
      ctx.env.to_js_value(&LibError { err: e.to_string().to_string() })
    }
  }
}

#[js_function(1)]
pub fn get_vbatt(mut ctx: CallContext) -> Result<JsUnknown> {
  let idx: u32 = u32::try_from(ctx.get::<JsNumber>(0)?)?;

  let mut voltage = 0;
  if passthru::DRIVER.read().unwrap().is_none() {
    return ctx.env.to_js_value(&LibError{ err: "No driver loaded!".to_string() });
  }
  match &passthru::DRIVER.write().unwrap().as_ref().unwrap().ioctl(idx, IoctlID::READ_VBATT, std::ptr::null_mut::<c_void>(), (&mut voltage) as *mut _ as *mut c_void) {
    Ok(_) => ctx.env.to_js_value(&Voltage{mv: voltage}),
    Err(e) => ctx.env.to_js_value(&LibError{ err: e.to_string().to_string() })
  }
}

#[js_function(1)]
pub fn load_file(mut ctx: CallContext) -> Result<JsUnknown> {
  let p = ctx.get::<JsString>(0)?;
  ctx.env.to_js_value(&LibError { err: "Native library - not implemented".to_string() })
}

//type PassThruConnectFn = unsafe extern "stdcall" fn(device_id: u32, protocol_id: u32, flags: u32, baudrate: u32, channel_id: *mut u32) -> i32;

#[derive(Debug, Serialize, Deserialize)]
struct Channel {
  channel_id: i32
}

#[js_function(4)]
pub fn connect(mut ctx: CallContext) -> Result<JsUnknown> {
  let dev_idx: u32 = u32::try_from(ctx.get::<JsNumber>(0)?)?;
  let protocol_id: u32 = u32::try_from(ctx.get::<JsNumber>(1)?)?;
  let protocol = match Protocol::from_raw(protocol_id) {
    Some(p) => p,
    None => return ctx.env.to_js_value(&LibError{ err: format!("Invalid protocol {}", protocol_id) })
  };

  let flags: u32 = u32::try_from(ctx.get::<JsNumber>(2)?)?;
  let baud: u32 = u32::try_from(ctx.get::<JsNumber>(3)?)?;
  if passthru::DRIVER.read().unwrap().is_none() {
    return ctx.env.to_js_value(&LibError{ err: "No driver loaded!".to_string() });
  }
  match &passthru::DRIVER.write().unwrap().as_ref().unwrap().connect(dev_idx, protocol, flags, baud) {
    Ok(channel_id) => ctx.env.to_js_value(&Channel{channel_id: *channel_id as i32}),
    Err(e) => ctx.env.to_js_value(&LibError{ err: e.to_string().to_string() })
  }
}

#[js_function(1)]
pub fn disconnect(mut ctx: CallContext) -> Result<JsUnknown> {
  let p = ctx.get::<JsString>(0)?;
  ctx.env.to_js_value(&LibError { err: "Native library - not implemented".to_string() })
}

#[js_function]
pub fn get_last_err(mut ctx: CallContext) -> Result<JsString> {
  if passthru::DRIVER.read().unwrap().is_none() {
    return ctx.env.create_string_from_std("No library loaded".to_string())
  }
  match &passthru::DRIVER.write().unwrap().as_ref().unwrap().get_last_error() {
    Ok(msg) => ctx.env.create_string_from_std(msg.clone()), // No error
    Err(_) => ctx.env.create_string_from_std("".to_string()) // No error
  }
}


// Register node functions

register_module!(ovd, init);

fn init(module: &mut Module) -> Result<()> {
  module.create_named_method("get_device_list", get_device_list)?;
  module.create_named_method("open", open)?;
  module.create_named_method("close", close)?;
  module.create_named_method("connect", connect)?;
  module.create_named_method("disconnect", disconnect)?;
  module.create_named_method("get_vbatt", get_vbatt)?;
  module.create_named_method("get_version", get_version)?;
  module.create_named_method("load_file", load_file)?;
  module.create_named_method("get_last_err", get_last_err)?;
  Ok(())
}