use crate::commapi::protocols::uds::UDSNegativeCode::RequestOutOfRange;
use crate::commapi::comm_api::{ComServer, ISO15765Config, ComServerError, ISO15765Data};
use std::time::Instant;

pub type Result<T> = std::result::Result<T, UDSProcessError>;

/// Represents a UDS Request that is sent to an ECU using ISO-TP protocol
pub struct UDSRequest {
    /// The command to execute on the ECU
    cmd: UDSCommand,
    /// The args provided to the ECU for the command
    args: Vec<u8>
}

impl UDSRequest {
    /// Creates a new UDSRequest using provided command and args
    pub fn new(command: UDSCommand, args: &[u8]) -> Self {
        Self {
            cmd: command,
            args: Vec::from(args)
        }
    }

    pub fn run_cmd_can(&self, server: &mut Box<dyn ComServer>, tp_config: &ISO15765Config) -> Result<UDSResponse> {
        server.open_iso15765_interface(500_000, false).map_err(|e| UDSProcessError::CommError(e))?;
        let res = {
            server.add_iso15765_filter(tp_config.recv_id, 0xFFFF, tp_config.send_id)?;
            server.set_iso15765_params(tp_config.sep_time, tp_config.block_size)?;

            let mut packet: Vec<u8> = vec![self.cmd as u8];
            packet.extend_from_slice(&self.args);

            let payload = ISO15765Data {
                id: tp_config.send_id,
                data: packet,
                pad_frame: false // Todo do we need to pad flow control frame?
            };
            server.send_iso15765_data(&[payload], 0)?;

            let clock = Instant::now();
            let mut p: Option<UDSResponse> = None;
            let mut timeout = 500; // ms
            while clock.elapsed().as_millis() < timeout {
                if let Ok(msgs) = server.read_iso15765_packets(0, 10) {
                    for m in msgs {
                        if m.data.len() > 0 {
                            let d = UDSResponse::from_data(&m.data)?;
                            if let UDSResponse::NegativeResponse(_, x) = d {
                                if x == UDSNegativeCode::ResponsePending {
                                    timeout += 500; // Response is coming, keep the clock ticking
                                    break;
                                }
                            }
                            p = Some(d);
                            timeout = 0; // Break
                            break;
                        }
                    }
                }
            }
            match p {
                Some(x) => Ok(x),
                None => Err(UDSProcessError::NoResponse)
            }
            // Now we can send the payload with the adapter being configured
        };
        server.close_iso15765_interface();
        return res
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialOrd, PartialEq)]
pub enum UDSCommand {
    DiagnosticSessionControl,
    ECUReset,
    ClearDTCInformation,
    ReadDTCInformation,
    ReadDataByID,
    ReadMemoryByAddress,
    ReadScalingDataById,
    SecurityAccess,
    CommunicationControl,
    Authentication,
    ReadDataByPeriodicID,
    WriteDataByID,
    IOCTLById,
    RoutineControl,
    RequestDownload,
    RequestUpload,
    TransferData,
    TransferExit,
    WriteMemoryByAddress,
    TesterPresent,
    ControlDTCSetting
}

impl UDSCommand {
    /// This function attempts to decode a given byte into a UDS Command
    /// This works for either the request byte of a UDS Payload,
    /// or on the response byte (Which has the highest bit set)
    pub (crate) fn from_byte(resp: &u8) -> Result<Self> {
        match *resp {
            0x10 | 0x50 => Ok(Self::DiagnosticSessionControl),
            0x11 | 0x51 => Ok(Self::ECUReset),
            0x14 | 0x54 => Ok(Self::ClearDTCInformation),
            0x19 | 0x59 => Ok(Self::ReadDTCInformation),
            0x22 | 0x62 => Ok(Self::ReadDataByID),
            0x23 | 0x63 => Ok(Self::ReadMemoryByAddress),
            0x24 | 0x64 => Ok(Self::ReadScalingDataById),
            0x27 | 0x67 => Ok(Self::SecurityAccess),
            0x28 | 0x68 => Ok(Self::CommunicationControl),
            0x29 | 0x69 => Ok(Self::Authentication),
            0x2A | 0x6A => Ok(Self::ReadDataByPeriodicID),
            0x2E | 0x6E => Ok(Self::WriteDataByID),
            0x2F | 0x6F => Ok(Self::IOCTLById),
            0x31 | 0x71 => Ok(Self::RoutineControl),
            0x34 | 0x74 => Ok(Self::RequestDownload),
            0x35 | 0x75 => Ok(Self::RequestUpload),
            0x36 | 0x76 => Ok(Self::TransferData),
            0x37 | 0x77 => Ok(Self::TransferExit),
            0x3D | 0x7D => Ok(Self::WriteMemoryByAddress),
            0x3E | 0x7E => Ok(Self::TesterPresent),
            0x85 | 0xC5 => Ok(Self::ControlDTCSetting),
            _ => Err(UDSProcessError::InvalidCommand)
        }
    }
}

#[derive(Copy, Debug, Clone, Eq, PartialOrd, PartialEq)]
/// All possible UDS Negative responses an ECU can return
/// when trying to run a command
pub enum UDSNegativeCode {
    /// This range of values is reserved by this document for future definition.
    ISOReserved,

    /// This response code indicates that the requested action has been rejected by the server.
    /// The generalReject response code shall only be implemented in the server if none of the
    /// negative response codes defined in this document meet the needs of the implementation.
    /// At no means shall this response code be a general replacement for other response codes defined.
    GeneralReject,

    /// This response code indicates that the requested action will not be taken because the
    /// server does not support the requested service. The server shall send this response code
    /// in case the client has sent a request message with a service identifier, which is either
    /// unknown or not supported by the server. Therefore this negative response code is not shown
    /// in the list of negative response codes to be supported for a diagnostic service, because
    /// this negative response code is not applicable for supported services.
    ServiceNotSupported,

    /// This response code indicates that the requested action will not be taken because the
    /// server does not support the service specific parameters of the request message.
    /// The server shall send this response code in case the client has sent a request
    /// message with a known and supported service identifier but with "sub function“
    /// which is either unknown or not supported.
    SubFunctionNotSupported,

    /// This response code indicates that the requested action will not be taken because the
    /// length of the received request message does not match the prescribed length for the
    /// specified service or the format of the parameters do not match the prescribed format
    /// for the specified service.
    IncorrectMessageLength,

    /// This response code shall be reported by the server if the response to be generated
    /// exceeds the maximum number of bytes available by the underlying network layer.
    ResponseTooLong,

    /// This response code indicates that the server is temporarily too busy to perform the
    /// requested operation. In this circumstance the client shall perform repetition of the
    /// "identical request message" or "another request message". The repetition of the request
    /// shall be delayed by a time specified in the respective implementation documents.
    ///
    /// Example: In a multi-client environment the diagnostic request of one client might be blocked
    /// temporarily by a NRC $21 while a different client finishes a diagnostic task.NOTE If the
    /// server is able to perform the diagnostic task but needs additional time to finish the task
    /// and prepare the response, the NRC 0x78 shall be used instead of NRC 0x21.This response code
    /// is in general supported by each diagnostic service, as not otherwise stated in the data link
    /// specific implementation document, therefore it is not listed in the list of applicable
    /// response codes of the diagnostic services.
    BusyRepeatRequest,

    /// This response code indicates that the requested action will not be
    /// taken because the server prerequisite conditions are not met.
    ConditionsNotCorrect,

    /// This response code indicates that the requested action will not be taken because the
    /// server expects a different sequence of request messages or message as sent by the client.
    /// This may occur when sequence sensitive requests are issued in the wrong order.
    ///
    /// Example: A successful SecurityAccess service specifies a sequence of requestSeed and
    /// sendKey as sub-functions in the request messages. If the sequence is sent different by the
    /// client the server shall send a negative response message with the negative response code
    /// 0x24 (requestSequenceError).
    RequestSequenceError,

    /// This response code indicates that the requested action will not be taken because the
    /// server has detected that the request message contains a parameter which attempts to
    /// substitute a value beyond its range of authority (e.g. attempting to substitute a data byte
    /// of 111 when the data is only defined to 100), or which attempts to access a
    /// dataIdentifier/routineIdentifer that is not supported or not supported in active session.
    /// This response code shall be implemented for all services, which allow the client to read data,
    /// write data or adjust functions by data in the server.
    RequestOutOfRange,

    /// This response code indicates that the requested action will not be taken because the
    /// server's security strategy has not been satisfied by the client. The server shall send this
    /// response code if one of the following cases occur:
    /// * The test conditions of the server are not met,
    /// * The required message sequence e.g. DiagnosticSessionControl, securityAccess is not met,
    /// * The client has sent a request message which requires an unlocked server.
    ///
    /// Beside the mandatory use of this negative response code as specified in the applicable
    /// services within this standard, this negative response code can also be used for any case
    /// where security is required and is not yet granted to perform the required service.
    SecurityAccessDenied,

    /// This response code indicates that the server has not given security access because the
    /// key sent by the client did not match with the key in the server's memory. This counts as
    /// an attempt to gain security. The server shall remain locked and increment its
    /// internal securityAccessFailed counter.
    InvalidKey,

    /// This response code indicates that the requested action will not be taken because the
    /// client has unsuccessfully attempted to gain security access more times than the server's
    /// security strategy will allow.
    ExceedNumberOfAttempts,

    /// This response code indicates that the requested action will not be taken because the
    /// client's latest attempt to gain security access was initiated before the server's required
    /// timeout period had elapsed.
    RequiredTimeDelayNotExpired,

    /// This range of values is reserved by ISO 15764 Extended data link security.
    ExtendedDataLinkSecurity,

    /// This response code indicates that an attempt to upload/download to a
    /// server's memory cannot be accomplished due to some fault conditions.
    UploadDownloadNotAccepted,

    /// This response code indicates that a data transfer operation was halted due to some fault.
    /// The active transferData sequence shall be aborted.
    TransferDataSuspended,

    /// This response code indicates that the server detected an error when erasing or programming
    /// a memory location in the permanent memory device (e.g. Flash Memory).
    GeneralProgrammingFailure,

    /// This response code indicates that the server detected an error in the sequence of
    /// blockSequenceCounter values. Note that the repetition of a TransferData request message
    /// with a blockSequenceCounter equal to the one included in the previous TransferData request
    /// message shall be accepted by the server.
    WrongBlockSequenceCounter,

    /// This response code indicates that the request message was received correctly, and that
    /// all parameters in the request message were valid, but the action to be performed is not yet
    /// completed and the server is not yet ready to receive another request. As soon as the
    /// requested service has been completed, the server shall send a positive response message or
    /// negative response message with a response code different from this. The negative response
    /// message with this response code may be repeated by the server until the requested service is
    /// completed and the final response message is sent. This response code might impact the application
    /// layer timing parameter values. The detailed specification shall be included in the data link
    /// specific implementation document.
    ///
    /// This response code shall only be used in a negative response message if the server will not
    /// be able to receive further request messages from the client while completing the requested
    /// diagnostic service. When this response code is used, the server shall always send a final
    /// response (positive or negative) independent of the suppressPosRspMsgIndicationBit value.
    /// A typical example where this response code may be used is when the client has sent a
    /// request message, which includes data to be programmed or erased in flash memory of the
    /// server. If the programming/erasing routine (usually executed out of RAM) is not able to
    /// support serial communication while writing to the flash memory the server shall send a
    /// negative response message with this response code. This response code is in general
    /// supported by each diagnostic service, as not otherwise stated in the data link specific
    /// implementation document, therefore it is not listed in the list of applicable response codes
    /// of the diagnostic services.
    ResponsePending,

    /// This response code indicates that the requested action will not be taken because the
    /// server does not support the requested sub-function in the session currently active. Within
    /// the programmingSession negative response code 0x12 (subFunctionNotSupported) may optionally
    /// be reported instead of negative response code 0x7F (subFunctionNotSupportedInActiveSession).
    /// This response code shall only be used when the requested sub-function is known to be supported
    /// in another session, otherwise response code 0x12 (subFunctionNotSupported) shall be used.
    /// This response code shall be supported by each diagnostic service with a sub-function parameter,
    /// if not otherwise stated in the data link specific implementation document, therefore it is not
    /// listed in the list of applicable response codes of the diagnostic services.
    SubFunctionNotSupportedActiveSession,

    /// This response code indicates that the requested action will not be taken because the
    /// server does not support the requested service in the session currently active. This response
    /// code shall only be used when the requested service is known to be supported in another
    /// session, otherwise response code 0x11 (serviceNotSupported) shall be used. This response
    /// code is in general supported by each diagnostic service, as not otherwise stated in the data
    /// link specific implementation document, therefore it is not listed in the list of applicable
    /// response codes of the diagnostic services.
    ServiceNotSupportedActiveSession,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for RPM is not met (current RPM is above a pre-programmed maximum threshold).
    RpmTooHigh,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for RPM is not met (current RPM is below a pre-programmed minimum threshold).
    RpmTooLow,

    /// This is required for those actuator tests which cannot be actuated while the Engine is running.
    /// This is different from RPM too high negative response and needs to be allowed.
    EngineIsRunning,

    /// This is required for those actuator tests which cannot be actuated unless the Engine is
    /// running. This is different from RPM too low negative response, and needs to be allowed.
    EngineIsNotRunning,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for engine run time is not met (current engine run time is below a
    /// preprogrammed limit).
    EngineRunTimeTooLow,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for temperature is not met (current temperature is above a
    /// preprogrammed maximum threshold).
    TempTooHigh,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for temperature is not met (current temperature is below a
    /// preprogrammed minimum threshold).
    TempTooLow,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for vehicle speed is not met (current VS is above a pre-programmed maximum threshold).
    SpeedTooHigh,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for vehicle speed is not met (current VS is below a pre-programmed minimum threshold).
    SpeedTooLow,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for throttle/pedal position is not met (current TP/APP is above a preprogrammed maximum threshold).
    ThrottleTooHigh,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for throttle/pedal position is not met (current TP/APP is below a preprogrammed minimum threshold).
    ThrottleTooLow,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for being in neutral is not met (current transmission range is not in neutral).
    TransmissionNotInNeutral,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for being in gear is not met (current transmission range is not in gear).
    TransmissionNotInGear,

    /// For safety reasons, this is required for certain tests before it begins, and must be
    /// maintained for the entire duration of the test.
    BrakeNotApplied,

    /// For safety reasons, this is required for certain tests before it begins, and must be
    /// maintained for the entire duration of the test.
    ShifterNotInPark,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for torque converter clutch is not met (current TCC status above a
    /// preprogrammed limit or locked).
    TorqueConverterClutchLocked,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for voltage at the primary pin of the server (ECU) is not met
    /// (current voltage is above a pre-programmed maximum threshold).
    VoltageTooHigh,

    /// This response code indicates that the requested action will not be taken because the server
    /// prerequisite condition for voltage at the primary pin of the server (ECU) is not met
    /// (current voltage is below a pre-programmed maximum threshold).
    VoltageTooLow,

    /// This range of values is reserved for future definition.
    ReservedSpecificConditionsIncorrect
}

impl UDSNegativeCode {
    pub (crate) fn from_byte(byte: &u8) -> Result<Self> {
        match *byte {
            0x01..=0x0F => Ok(Self::ISOReserved),
            0x10 => Ok(Self::GeneralReject),
            0x11 => Ok(Self::ServiceNotSupported),
            0x12 => Ok(Self::SubFunctionNotSupported),
            0x13 => Ok(Self::IncorrectMessageLength),
            0x14 => Ok(Self::ResponseTooLong),
            0x15..=0x20 => Ok(Self::ISOReserved),
            0x21 => Ok(Self::BusyRepeatRequest),
            0x22 => Ok(Self::ConditionsNotCorrect),
            0x23 => Ok(Self::ISOReserved),
            0x24 => Ok(Self::RequestSequenceError),
            0x25..=0x30 => Ok(Self::ISOReserved),
            0x31 => Ok(Self::RequestOutOfRange),
            0x32 => Ok(Self::ISOReserved),
            0x33 => Ok(Self::SecurityAccessDenied),
            0x34 => Ok(Self::ISOReserved),
            0x35 => Ok(Self::InvalidKey),
            0x36 => Ok(Self::ExceedNumberOfAttempts),
            0x37 => Ok(Self::RequiredTimeDelayNotExpired),
            0x38..=0x4F => Ok(Self::ExtendedDataLinkSecurity), // Todo get the real codes from ISO 15765 Extended data link security
            0x50..=0x6F => Ok(Self::ISOReserved),
            0x70 => Ok(Self::UploadDownloadNotAccepted),
            0x71 => Ok(Self::TransferDataSuspended),
            0x72 => Ok(Self::GeneralProgrammingFailure),
            0x73 => Ok(Self::WrongBlockSequenceCounter),
            0x74..=0x77 => Ok(Self::ISOReserved),
            0x78 => Ok(Self::ResponsePending),
            0x79..=0x7D => Ok(Self::ISOReserved),
            0x7E => Ok(Self::SubFunctionNotSupportedActiveSession),
            0x7F => Ok(Self::ServiceNotSupportedActiveSession),
            0x80 => Ok(Self::ISOReserved),
            0x81 => Ok(Self::RpmTooHigh),
            0x82 => Ok(Self::RpmTooLow),
            0x83 => Ok(Self::EngineIsRunning),
            0x84 => Ok(Self::EngineIsNotRunning),
            0x85 => Ok(Self::EngineRunTimeTooLow),
            0x86 => Ok(Self::TempTooHigh),
            0x87 => Ok(Self::TempTooLow),
            0x88 => Ok(Self::SpeedTooHigh),
            0x89 => Ok(Self::SpeedTooLow),
            0x8A => Ok(Self::ThrottleTooHigh),
            0x8B => Ok(Self::ThrottleTooLow),
            0x8C => Ok(Self::TransmissionNotInNeutral),
            0x8E => Ok(Self::ISOReserved),
            0x8F => Ok(Self::BrakeNotApplied),
            0x90 => Ok(Self::ShifterNotInPark),
            0x91 => Ok(Self::TorqueConverterClutchLocked),
            0x92 => Ok(Self::VoltageTooHigh),
            0x93 => Ok(Self::VoltageTooLow),
            0x94..=0xFE => Ok(Self::ReservedSpecificConditionsIncorrect),
            0xFF => Ok(Self::ISOReserved),
            _ => Err(UDSProcessError::InvalidErrorCode)
        }
    }
}

#[derive(Clone, Debug)]
pub enum UDSResponse {
    /// Positive response - Command was executed successfully.
    /// Args returned by the ECU are provided
    PositiveResponse(UDSCommand, Vec<u8>),
    /// Negative response - ECU Failed to process the request
    /// Error code is provided
    NegativeResponse(UDSCommand, UDSNegativeCode)
}

#[derive(Clone, Debug)]
/// An error which can occur whilst processing a response from an ECU
pub enum UDSProcessError {
    /// ECU Responded with an ID that isn't known in the UDS protocol
    InvalidCommand,
    /// ECU Responded with an Error that isn't known
    InvalidErrorCode,
    /// ECU Response size was invalid
    InvalidDataLen,
    /// ECU did not respond
    NoResponse,
    /// Driver error whilst trying to communicate with the ECU
    CommError(ComServerError),
}

impl std::convert::From<ComServerError> for UDSProcessError {
    fn from(t: ComServerError) -> Self {
        Self::CommError(t)
    }
}

impl UDSResponse {
    fn from_data(args: &[u8]) -> Result<Self> {
        if args.len() == 0x00 {
            return Err(UDSProcessError::InvalidDataLen)
        }
        if args[0] == 0x7F {
            if args.len() == 3 {
                let cmd = UDSCommand::from_byte(&args[1])?;
                let err = UDSNegativeCode::from_byte(&args[2])?;
                Ok(UDSResponse::NegativeResponse(cmd, err))
            } else {
                Err(UDSProcessError::InvalidDataLen)
            }
        } else {
            let cmd = UDSCommand::from_byte(&args[0])?;
            let args = Vec::from(&args[1..]);
            Ok(UDSResponse::PositiveResponse(cmd, args))
        }
    }
}