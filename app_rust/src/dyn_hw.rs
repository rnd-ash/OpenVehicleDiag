use std::sync::{Mutex, Arc};

use ecu_diagnostics::hardware::{passthru::PassthruDevice, Hardware};
#[cfg(unix)]
use ecu_diagnostics::hardware::{socketcan::SocketCanDevice};

#[derive(Clone)]
pub struct DynHardware {
    hw: DynHardwareEnum,
    connected: bool
}

impl DynHardware {
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

    pub fn get_capabilities(&mut self) -> ecu_diagnostics::hardware::HardwareCapabilities {
        self.hw.get_capabilities()
    }

    pub fn set_connect_state(&mut self, x: bool) {
        self.connected = x;
    }
}

#[derive(Clone)]
enum DynHardwareEnum {
    #[cfg(unix)]
    SocketCan(Arc<Mutex<SocketCanDevice>>),
    Passthru(Arc<Mutex<PassthruDevice>>)
}

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

    fn get_capabilities(&mut self) -> ecu_diagnostics::hardware::HardwareCapabilities {
        match self {
            #[cfg(unix)]
            Self::SocketCan(s) => s.lock().unwrap().get_capabilities().clone(),
            Self::Passthru(p) => p.lock().unwrap().get_capabilities().clone(),
        }
    }
}