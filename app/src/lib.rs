use nodejs_sys::{
    napi_callback_info, napi_create_function, napi_create_string_utf8, napi_env,
    napi_set_named_property, napi_value,
};
use std::ffi::CString;
pub unsafe extern "C" fn say_hello(env: napi_env, _info: napi_callback_info) -> napi_value {
    let mut local: napi_value = std::mem::zeroed();
    let p = CString::new("Hello from rust").expect("CString::new failed");
    napi_create_string_utf8(env, p.as_ptr(), p.len(), &mut local);
    local
}
#[no_mangle]
pub unsafe extern "C" fn napi_register_module_v1(
    env: napi_env,
    exports: napi_value,
) -> nodejs_sys::napi_value {
    let p = CString::new("myFunc").expect("CString::new failed");
// creating a location where pointer to napi_value be written
    let mut local: napi_value = std::mem::zeroed();
    napi_create_function(
        env,
        p.as_ptr(),
        5,
        Some(say_hello),
        std::ptr::null_mut(),
        &mut local,
    ); 
    napi_set_named_property(env, exports, p.as_ptr(), local);
    exports
}
