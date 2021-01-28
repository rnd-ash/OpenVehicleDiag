use custom_session::{CustomDiagSession, CustomDiagSessionMsg};
use iced::Element;
use kwp2000_session::KWP2000DiagSession;
use uds_session::{UDSDiagSession, UDSDiagSessionMsg};

use crate::commapi::comm_api::{ComServer, ISO15765Config};

use self::kwp2000_session::KWP2000DiagSessionMsg;

use super::diag_manual::DiagManualMessage;

pub mod custom_session;
pub mod json_session;
pub mod kwp2000_session;
pub mod uds_session;


pub enum SessionType {
    UDS,
    KWP,
    Custom
}

#[derive(Debug, Clone)]
pub enum SessionMsg {
    KWP(KWP2000DiagSessionMsg),
    UDS(UDSDiagSessionMsg),
    Custom(CustomDiagSessionMsg),
    ExitSession,
}

#[derive(Debug, Clone)]
pub enum DiagSession {
    UDS(UDSDiagSession),
    KWP(KWP2000DiagSession),
    Custom(CustomDiagSession)
}

impl DiagSession {

    pub fn new(session_type: &SessionType, comm_server: Box<dyn ComServer>, ecu: ISO15765Config) -> Self {
        match session_type {
            SessionType::UDS => Self::UDS(UDSDiagSession::new(comm_server, ecu)),
            SessionType::KWP => Self::KWP(KWP2000DiagSession::new(comm_server, ecu)),
            SessionType::Custom => Self::Custom(CustomDiagSession::new(comm_server, ecu)),
        }
    }

    pub fn view(&mut self) -> Element<SessionMsg> {
        match self {
            DiagSession::UDS(s) => s.view(),
            DiagSession::KWP(s) => s.view(),
            DiagSession::Custom(s) => s.view()
        }
    }

    pub fn update(&mut self, msg: &SessionMsg) -> Option<SessionMsg> {
        match self {
            DiagSession::UDS(s) => if let SessionMsg::UDS(m) = msg { s.update(m) } else { None },
            DiagSession::KWP(s) => if let SessionMsg::KWP(m) = msg { s.update(m) } else { None },
            DiagSession::Custom(s) => if let SessionMsg::Custom(m) = msg { s.update(m) } else { None },
        }
    }
}

pub trait DiagMessageTrait : std::fmt::Debug {

}


pub trait SessionTrait : std::fmt::Debug {
    type msg: DiagMessageTrait;

    fn view(&mut self) -> Element<SessionMsg>;

    fn update(&mut self, msg: &Self::msg) -> Option<SessionMsg>;
}