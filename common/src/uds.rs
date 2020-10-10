// Universal diagnostic services library

/// Contains a list of all known UDS Commands
pub enum UDSCommandID {
    SessionControl,
    ECUReset,
    SecurityAccess,
    CommunicationControl,
    Authentication,
    TesterPresent,
    AccessTimingParams,
    SecuredDataTransmition,
    ControlDTC,
    ResponseOnEvent,
    LinkControl,
    ReadDataByID,
    ReadMemByAddress,
    ReadScalingDataById,
    ReadDataByIdPeriodic,

    ///
    DynamicDefineDataId,

    /// Write data to the ECU given a Data ID (DID) specified in DynmicDefineDataId
    WriteDataById,

    /// Allows for writing to ECU memory
    /// at one or more arbitary addresses given by the host client
    WriteMemByAddress,

    /// Delete all stored Diagnostic trouble codes on the ECU
    ClearDTC,

    /// Read all Diagnostic trouble codes from the ECU
    ReadDTC,

    IOControlById,

    /// Tell the ECU to do something with a running routine.
    /// A routine control message can be of 1 of 3 types:
    ///
    /// * Start message - Initialise a service / ask the ECU if the service has started
    /// * Stop message - A running service can be interrupted at any time
    /// * Query message - Ask the ECU for the result of the service
    ///
    /// The start or stop message parameters can be specified to allow
    /// for any routine ID to be specified (Up to the OEM)
    RoutineControl,

    /// Tell the ECU to receive data from the host client,
    /// This SID must contain additional args
    /// regarding data size and location of the data
    /// to be downloaded from the client
    ReqDownload,

    /// Tell the ECU to upload data to the host client,
    /// This SID must contain additional args
    /// regarding data size and location of the data
    /// to be uploaded to the client
    ReqUpload,

    /// This is used when both uploading or downloading
    /// data to / from an ECU. Prior to using this SID,
    /// use ReqDownload or ReqUpload to notify the ECU
    /// of the data transfer direction
    TransferData,

    /// Tell ECU a data transmission has been 'completed'
    /// If the ECU responds with a Negative Response to this SID,
    /// it implies that it has not sent / received all the data
    /// needed before exiting the data transfer
    RequestTransferExit,

    /// Initiate a file download from the client / server to ECU
    /// or from ECU to client / server
    RequestFileTransfer,

    /// Negative response from an ECU
    /// This can ONLY be received from an ECU, never sent.
    /// If received, it implies the ECU Failed to process the sent command
    NegativeResponse,
}

impl UDSCommandID {
    /// Gets the SID Byte from a UDS Command.
    /// **Asking for NegativeResponse will result in a panic condition**
    pub fn get_send_id(&self) -> u8 {
        return match *self {
            UDSCommandID::SessionControl => 0x10,
            UDSCommandID::ECUReset => 0x11,
            UDSCommandID::SecurityAccess => 0x27,
            UDSCommandID::CommunicationControl => 0x28,
            UDSCommandID::Authentication => 0x29,
            UDSCommandID::TesterPresent => 0x3E,
            UDSCommandID::AccessTimingParams => 0x83,
            UDSCommandID::SecuredDataTransmition => 0x84,
            UDSCommandID::ControlDTC => 0x85,
            UDSCommandID::ResponseOnEvent => 0x86,
            UDSCommandID::LinkControl => 0x87,
            UDSCommandID::ReadDataByID => 0x22,
            UDSCommandID::ReadMemByAddress => 0x23,
            UDSCommandID::ReadScalingDataById => 0x24,
            UDSCommandID::ReadDataByIdPeriodic => 0x2A,
            UDSCommandID::DynamicDefineDataId => 0x2C,
            UDSCommandID::WriteDataById => 0x2E,
            UDSCommandID::WriteMemByAddress => 0x3D,
            UDSCommandID::ClearDTC => 0x14,
            UDSCommandID::ReadDTC => 0x19,
            UDSCommandID::IOControlById => 0x2F,
            UDSCommandID::RoutineControl => 0x31,
            UDSCommandID::ReqDownload => 0x34,
            UDSCommandID::ReqUpload => 0x35,
            UDSCommandID::TransferData => 0x36,
            UDSCommandID::RequestTransferExit => 0x37,
            UDSCommandID::RequestFileTransfer => 0x38,
            UDSCommandID::NegativeResponse => panic!("Cannot send Negative response to ECU!"),
        };
    }

    /// Processes the response SID from the ECU,
    /// and returns the necessary UDS Command ID for the response
    /// If None is returned, then the sid is not UDS compliant
    pub fn process_ecu_response(sid: u8) -> Option<UDSCommandID> {
        return match sid {
            0x50 => Some(UDSCommandID::SessionControl),
            0x51 => Some(UDSCommandID::ECUReset),
            0x67 => Some(UDSCommandID::CommunicationControl),
            0x69 => Some(UDSCommandID::Authentication),
            0x7E => Some(UDSCommandID::TesterPresent),
            0xC3 => Some(UDSCommandID::AccessTimingParams),
            0xC4 => Some(UDSCommandID::SecuredDataTransmition),
            0xC5 => Some(UDSCommandID::ControlDTC),
            0xC6 => Some(UDSCommandID::ResponseOnEvent),
            0xC7 => Some(UDSCommandID::LinkControl),
            0x62 => Some(UDSCommandID::ReadDataByID),
            0x63 => Some(UDSCommandID::ReadMemByAddress),
            0x64 => Some(UDSCommandID::ReadScalingDataById),
            0x6A => Some(UDSCommandID::ReadDataByIdPeriodic),
            0x6C => Some(UDSCommandID::DynamicDefineDataId),
            0x6E => Some(UDSCommandID::WriteDataById),
            0x7D => Some(UDSCommandID::WriteMemByAddress),
            0x54 => Some(UDSCommandID::ClearDTC),
            0x59 => Some(UDSCommandID::ReadDTC),
            0x6F => Some(UDSCommandID::IOControlById),
            0x71 => Some(UDSCommandID::RoutineControl),
            0x74 => Some(UDSCommandID::ReqDownload),
            0x75 => Some(UDSCommandID::ReqUpload),
            0x76 => Some(UDSCommandID::TransferData),
            0x77 => Some(UDSCommandID::RequestTransferExit),
            0x78 => Some(UDSCommandID::RequestFileTransfer),
            0x7F => Some(UDSCommandID::NegativeResponse),
            _ => None,
        };
    }
}
