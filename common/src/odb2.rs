/// ODB-II Modes (SAE J1979)
///
/// OEM's are **NOT** Required to impliment
/// ever single mode in this table, so certain
/// modes will result in an ECU responding with error
///
/// MB (2000-2010) does not support mode 0x06
pub enum Mode {
    /// Show current data
    ShowCurrentData = 0x01,

    /// Show freeze frame data
    ShowFreezeFrame = 0x02,

    /// Show stored Diagnostic trouble codes
    ShowDTC = 0x03,

    /// Clear diagnostic trouble codes and stored values
    ClearDTC = 0x04,

    /// Test results - O2 sensor monitoring
    TestO2Monitoring = 0x05,

    /// Test results - Other components / systems
    TestOtherSystems = 0x06,

    /// Show pending Diagnostic trouble codes (During current / last drive cycle)
    ShowPendingDTC = 0x07,

    /// Control operation of on-board components / systems
    ControlSystem = 0x08,

    // Request vehicle information
    RequestInfo = 0x09,

    /// Permanent Diagnostic trouble codes
    PermanentDTC = 0x0A,
}

pub struct Mode01 {
    id: u8,        // PID
    args: Vec<u8>, // Returned Bytes
}

pub struct ParsedPid {
    pub value: f32,
    pub unit: String,
}

impl ToString for ParsedPid {
    fn to_string(&self) -> String {
        return format!("{}{}", self.value, self.unit);
    }
}

pub enum Mode01Pids {
    PidSupportRange00 = 0x00,
    MonitorStauts = 0x01,
    FreezeDTC = 0x02,
    FuelSystemStatus = 0x03,
    EngineLoad = 0x04,
    CoolantTemp = 0x05,
    ShortTermFuelTrim01 = 0x06,
    LongTermFuelTrim01 = 0x07,
    ShortTermFuelTrim02 = 0x08,
    LongTermFuelTrim02 = 0x09,
    FuelPressure = 0x0A,
    IntakeManifoldPressure = 0x0B,
    EngineSpeed = 0x0C,
    VehicleSpeed = 0x0D,
    TimingAdvance = 0x0E,
    IntakeAirTemp = 0x0F,
    MassAirflowRate = 0x10,
    ThrottlePosition = 0x11,
    CmdAirStatus2 = 0x12,
    O2SensorsPresent2 = 0x13,
    O2SensorVT01 = 0x14,
    O2SensorVT02 = 0x015,
    O2SensorVT03 = 0x16,
    O2SensorVT04 = 0x17,
    O2SensorVT05 = 0x18,
    O2SensorVT06 = 0x19,
    O2SensorVT07 = 0x1A,
    O2SensorVT08 = 0x1B,
    ODBStandard = 0x1C,
    O2SensorsPresent4 = 0x1D,
    AUXStatus = 0x1E,
    EngineRunTime = 0x1F,
    PidSUpportRange21 = 0x20,
    MILDistance = 0x21,
    FuelRailPressure = 0x22,
    FuelRailGaugePressure = 0x23,
    O2SensorRV01 = 0x24,
    O2SensorRV02 = 0x25,
    O2SensorRV03 = 0x26,
    O2SensorRV04 = 0x27,
    O2SensorRV05 = 0x28,
    O2SensorRV06 = 0x29,
    O2SensorRV07 = 0x2A,
    O2SensorRV08 = 0x2B,
    CommandedEGR,
    EGRError,
    CommandedEvap,
    FuelTankLevel,
    WarmUpsCodeCleared,
    DistanceCodeCleared,
    EvapVaporPressure,
    AbsolutePressure,
    O2SensorFAC01,
    O2SensorFAC02,
    O2SensorFAC03,
    O2SensorFAC04,
    O2SensorFAC05,
    O2SensorFAC06,
    O2SensorFAC07,
    O2SensorFAC08,
    CatTempB1S1,
    CatTempB2S1,
    CatTempB1S2,
    CatTempB2S2,
    PidSupportRange41,
    MonitorStatus,
    ECUVoltage,
    AbsoluteLoad,
    FuelAirRatio,
    RelativeThrottlePos,
    AmbientAirTemp,
    ThrottlePosB,
    ThrottlePosC,
    ThrottlePosD,
    ThrottlePosE,
    ThrottlePosF,
    CommandedThrottle,
    DTCTime,
    MaxValues,
    MaxMAF,
    FuelType,
    EthanolFuelPercent,
    AbsoluteEvapVaporPressure,
    EvapVaporPressure01,
    ShortTermO2TrimA1B3,
    LongTermO2TrimA1B3,
    ShortTermO2TrimA2B4,
    LongTermO2TrimA2B4,
    FuelRailAbsPressure,
    RelativeAccelPosition,
    HybridBatteryLifeRemain,
    EngineOilTemp,
    FuelInjectionTiming,
    EngineFuelRate,
    EmmissionRequirment,
    PidSupportRange61,
    DriverDemandTorque,
    EngineTorquePercent,
    EngineReferenceTorque,
    EnginePercentTorqueData,
}

/// Enumeration for Mode 1 PID 51 (Fuel Type)
enum FuelType {
    /// Not applicable / Not avaliable
    NA = 0x00,
    /// Petrol engine
    Gasoline = 0x01,
    /// Methanol engine
    Methanol = 0x02,
    /// Ethanol engine
    Ethanol = 0x03,
    /// Diesel engine
    Diesel = 0x04,
    /// Liquefield petroleum gas engine
    LPG = 0x05,
    /// Compressed natural gas Engine
    CNG = 0x06,
    /// Propane engine
    Propane = 0x07,
    /// Electric (Pure
    Electric = 0x08,
    /// Bifuel engine with Petrol as primary fuel
    BifuelGasoline = 0x09,
    /// Bifuel engine with methanol as primary fuel
    BifuelMethanol = 0x0A,
    /// Bifuel engine with ethanol as primary fuel
    BifuelEthanol = 0x0B,
    /// Bifuel engine with Liquefield petroleum gas as primary fuel
    BifuelLPG = 0x0C,
    /// Bifuel engine with Compressed natural gas as primary fuel
    BifuelCNG = 0x0D,
    /// Bifuel engine with Propane as primary fuel
    BifuelPropane = 0x0E,
    /// Bifuel with Electric and combustion engine as primary
    BifuelElectricCombustion = 0x0F,
    /// Bifuel with a hybrid system as primary
    BifuelHybrid = 0x10,
    /// Hybrid with petrol engine
    HybridGasoline = 0x11,
    /// Hybrid with ethanol engine
    HybridEthanol = 0x12,
    /// Hybrid with diesel engine
    HybridDiesel = 0x13,
    /// Hybrid with electric engine
    HybridElectric = 0x14,
    /// Hybrid with generic combustion engine
    HybridElectricCombustion = 0x15,
    /// Hybrid with regenerative system
    HybridRegenerative = 0x16,
    /// Bifuel engine with Diesel as primary fuel
    BifuelDiesel = 0x17,
}

impl Mode01 {
    fn validate(&self, size: usize, pid: Mode01Pids) -> Option<&Self> {
        if self.args.len() == size && self.id == pid as u8 {
            Some(self)
        } else {
            None
        }
    }

    /// Returns Calcualted engine load as a float from 0-100%
    pub fn get_engine_load(&self) -> Option<ParsedPid> {
        self.validate(4, Mode01Pids::EngineLoad)
            .map(|load| ParsedPid {
                value: load.args[0] as f32 / 2.55,
                unit: "%".to_string(),
            })
    }

    fn get_coolant_temp(&self) -> Option<ParsedPid> {
        return Some(ParsedPid {
            value: (self.args[0] as f32 - 40.0),
            unit: "°C".to_string(),
        });
    }
}

#[test]
fn test_engine_load() {
    let resp = Mode01 {
        id: Mode01Pids::EngineLoad as u8,
        args: vec![0x10],
    };

    if let Some(parsed) = resp.get_engine_load() {
        println!("Engine Load: {}", parsed.to_string());
        assert!(parsed.value == 6.27451);
        assert!(parsed.unit == "%");
    }
}
