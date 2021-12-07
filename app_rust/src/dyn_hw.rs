use std::{sync::{Mutex, Arc}, marker::PhantomData};

use ecu_diagnostics::hardware::{passthru::PassthruDevice, Hardware};
#[cfg(unix)]
use ecu_diagnostics::hardware::{socketcan::SocketCanDevice};

#[derive(Clone)]
pub struct DynHardware {
    hw: DynHardwareEnum
}

unsafe impl Send for DynHardware{}
unsafe impl Sync for DynHardware{}

impl DynHardware {
    pub fn new_from_passthru(hw: Arc<Mutex<PassthruDevice>>) -> Self {
        Self {
            hw: DynHardwareEnum::Passthru(hw),
        }
    }

    #[cfg(unix)]
    pub fn new_from_socketcan(hw: Arc<Mutex<SocketCanDevice>>) -> Self {
        Self {
            hw: DynHardwareEnum::SocketCan(hw),
        }
    }

    pub fn create_iso_tp_channel(&mut self) -> ecu_diagnostics::hardware::HardwareResult<Box<dyn ecu_diagnostics::channel::IsoTPChannel>> {
        self.hw.create_iso_tp_channel()
    }

    pub fn create_can_channel(&mut self) -> ecu_diagnostics::hardware::HardwareResult<Box<dyn ecu_diagnostics::channel::CanChannel>> {
        self.hw.create_can_channel()
    }

    pub fn read_battery_voltage(&mut self) -> Option<f32> {
        self.hw.read_battery_voltage()
    }

    pub fn read_ignition_voltage(&mut self) -> Option<f32> {
        self.hw.read_ignition_voltage()
    }

    pub fn get_info(&mut self) -> ecu_diagnostics::hardware::HardwareInfo {
        self.hw.get_info()
    }

    pub fn is_connected(&self) -> bool {
        self.hw.is_connected()
    }
}

#[derive(Clone)]
enum DynHardwareEnum {
    #[cfg(unix)]
    SocketCan(Arc<Mutex<SocketCanDevice>>),
    Passthru(Arc<Mutex<PassthruDevice>>)
}

unsafe impl Send for DynHardwareEnum{}
unsafe impl Sync for DynHardwareEnum{}

impl DynHardwareEnum {
    fn create_iso_tp_channel(&mut self) -> ecu_diagnostics::hardware::HardwareResult<Box<dyn ecu_diagnostics::channel::IsoTPChannel>> {
        match self {
            #[cfg(unix)]
            Self::SocketCan(s) => Hardware::create_iso_tp_channel(s.clone()),
            Self::Passthru(p) => Hardware::create_iso_tp_channel(p.clone()),
        }
    }

    fn create_can_channel(&mut self) -> ecu_diagnostics::hardware::HardwareResult<Box<dyn ecu_diagnostics::channel::CanChannel>> {
        match self {
            #[cfg(unix)]
            Self::SocketCan(s) => Hardware::create_can_channel(s.clone()),
            Self::Passthru(p) => Hardware::create_can_channel(p.clone()),
        }
    }

    fn read_battery_voltage(&mut self) -> Option<f32> {
        match self {
            #[cfg(unix)]
            Self::SocketCan(s) => s.lock().unwrap().read_battery_voltage(),
            Self::Passthru(p) => p.lock().unwrap().read_battery_voltage(),
        }
    }

    fn read_ignition_voltage(&mut self) -> Option<f32> {
        match self {
            #[cfg(unix)]
            Self::SocketCan(s) => s.lock().unwrap().read_battery_voltage(),
            Self::Passthru(p) => p.lock().unwrap().read_battery_voltage(),
        }
    }

    fn get_info(&mut self) -> ecu_diagnostics::hardware::HardwareInfo {
        match self {
            #[cfg(unix)]
            Self::SocketCan(s) => s.lock().unwrap().get_info().clone(),
            Self::Passthru(p) => p.lock().unwrap().get_info().clone(),
        }
    }

    fn is_connected(&self) -> bool {
        match self {
            #[cfg(unix)]
            Self::SocketCan(s) => {
                match s.lock() {
                    Ok(dev) => {
                        dev.is_can_channel_open() || dev.is_iso_tp_channel_open()
                    },
                    Err(_) => false
                } 
            }
            Self::Passthru(p) => {
                match p.lock() {
                    Ok(dev) => {
                        dev.is_can_channel_open() || dev.is_iso_tp_channel_open()
                    },
                    Err(_) => false
                }  
            }
        }
    }
}