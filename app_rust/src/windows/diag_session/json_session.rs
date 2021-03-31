use std::{
    borrow::Borrow, cell::RefCell, cmp::min, collections::HashMap, sync::Arc, time::Instant,
};

use commapi::protocols;
use common::schema::{ConType, Connection, OvdECU, diag::service::{ParamDecodeError, Service}, variant::{ECUVariantDefinition, ECUVariantPattern}};
use iced::{time, Align, Column, Container, Length, Row, Subscription};
use protocols::{ECUCommand, Selectable};
use serde_json::de::Read;

use crate::{commapi::{self, comm_api::{ComServer, ISO15765Config}, protocols::{DTC, DiagProtocol, DiagServer, ProtocolResult, ProtocolServer, kwp2000::KWP2000ECU}}, themes::{
        button_coloured, button_outlined, elements::TextInput, picklist, text, text_input,
        title_text, ButtonType, TextType,
    }, windows::diag_manual::DiagManualMessage};

use super::{
    log_view::{LogType, LogView},
    DiagMessageTrait, SessionError, SessionMsg, SessionResult, SessionTrait,
};

type DiagService = commapi::protocols::kwp2000::Service;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonDiagSessionMsg {
    ReadErrors,
    ClearErrors,
    RunService,
    ExecuteService(ServiceRef, Vec<u8>),
    ClearLogs,
    Selector(SelectorMsg),
    LoopRead(Instant),
    Back,
}

impl From<SelectorMsg> for JsonDiagSessionMsg {
    fn from(x: SelectorMsg) -> Self {
        JsonDiagSessionMsg::Selector(x)
    }
}

impl DiagMessageTrait for JsonDiagSessionMsg {
    fn is_back(&self) -> bool {
        self == &JsonDiagSessionMsg::Back
    }
}

#[derive(Debug, Clone)]
pub struct JsonDiagSession {
    unknown_variant: bool,
    connection_settings: Connection,
    server: DiagServer,
    ecu_text: (String, String),
    ecu_data: ECUVariantDefinition,
    pattern: ECUVariantPattern,
    log_view: LogView,
    clear_errors: iced::button::State,

    service_selector: ServiceSelector,

    execute_service: iced::button::State,
    clear_log_btn: iced::button::State,
    read_errors: iced::button::State,
    looping_text: String,
    looping_service: Option<ServiceRef>, // Allow only read-only services to be loop read
    logged_dtcs: Vec<DTC>,
}

impl JsonDiagSession {
    pub fn new(
        comm_server: Box<dyn ComServer>,
        ecu_data: OvdECU,
        connection_settings: Connection
    ) -> SessionResult<Self> {
        let diag_server_type = match connection_settings.server_type {
            common::schema::ServerType::UDS => DiagProtocol::UDS,
            common::schema::ServerType::KWP2000 => DiagProtocol::KWP2000
        };

        // TODO K-Line KWP2000
        // For now, Diag server ONLY supports ISO-TP, not LIN!
        let create_server = match connection_settings.connection_type {
            ConType::ISOTP { blocksize, st_min } => {
                let cfg = ISO15765Config {
                    baud: connection_settings.baud,
                    send_id: connection_settings.send_id,
                    recv_id: connection_settings.recv_id,
                    block_size: blocksize,
                    sep_time: st_min,
                };
                DiagServer::new(comm_server, &cfg, connection_settings.global_send_id, diag_server_type)
            },
            ConType::LIN { .. } => return Err(SessionError::Other("K-Line is not implemented at this time".into()))
        };


        match create_server {
            Ok(server) => {
                println!("Server started");
                let variant = server.get_variant_id()? as u32;
                let mut unknown_variant = false;
                let mut ecu_varient = ecu_data.variants.clone().into_iter().find(|x| {
                    x.clone()
                        .patterns
                        .into_iter()
                        .any(|p| p.vendor_id == variant)
                }).unwrap_or_else(|| {
                    eprintln!("WARNING. Unknown ECU Variant!");
                    unknown_variant = true;
                    ecu_data.variants[0].clone()
                });
                let pattern = &ecu_varient
                    .patterns
                    .iter()
                    .find(|x| x.vendor_id == variant)
                    .unwrap()
                    .clone();
                println!("ECU Variant: {} (Vendor: {})", ecu_varient.name, pattern.vendor);

                let read_functions: Vec<ServiceRef> = ecu_varient.downloads
                    .iter()
                    .clone()
                    .map(|s| ServiceRef {
                        inner: RefCell::new(s.clone())
                    })
                    .collect();

                let write_functions: Vec<ServiceRef> = Vec::new();
                let actuation_functions: Vec<ServiceRef> = Vec::new();

                Ok(Self {
                    unknown_variant,
                    connection_settings: connection_settings,
                    ecu_text: (ecu_data.name, ecu_data.description),
                    server,
                    ecu_data: ecu_varient,
                    pattern: pattern.clone(),
                    service_selector: ServiceSelector::new(
                        read_functions,
                        write_functions,
                        actuation_functions,
                    ),
                    log_view: LogView::new(),
                    read_errors: Default::default(),
                    clear_errors: Default::default(),
                    execute_service: Default::default(),
                    clear_log_btn: Default::default(),
                    looping_service: None,
                    looping_text: String::new(),
                    logged_dtcs: Vec::new()
                })
            }
            Err(e) => {
                eprintln!("Could not setup diag server");
                Err(SessionError::ServerError(e))
            }
        }
    }
}

impl SessionTrait for JsonDiagSession {
    type msg = JsonDiagSessionMsg;

    fn view(&mut self) -> iced::Element<Self::msg> {
        let mut btn_view = Column::new()
            .push(
                button_outlined(&mut self.read_errors, "Read errors", ButtonType::Primary)
                    .on_press(JsonDiagSessionMsg::ReadErrors),
            )
            .width(Length::FillPortion(1));

        if self.logged_dtcs.len() > 0 {
            btn_view = btn_view.push(
                button_outlined(&mut self.clear_errors, "Clear errors", ButtonType::Primary)
                    .on_press(JsonDiagSessionMsg::ClearErrors),
            )
        }

        btn_view = btn_view.push(
            self.service_selector
                .view()
                .map(JsonDiagSessionMsg::Selector),
        );
        if self.looping_service.is_some() {
            btn_view = btn_view.push(text(&self.looping_text, TextType::Normal).size(14));
        }
        Column::new()
            .align_items(Align::Center)
            .spacing(8)
            .padding(8)
            .push(title_text(
                format!(
                    "ECU: {} ({}). DiagVersion: {}, Vendor: {}",
                    self.ecu_text.0, self.ecu_text.1, self.ecu_data.name, self.pattern.vendor
                )
                .as_str(),
                crate::themes::TitleSize::P4,
            ))
            .push(text(
                format!("Automatic connection method!: Using {:?} at {}bps with {:?} a diagnostic server", 
                    &self.connection_settings.connection_type,
                    &self.connection_settings.baud,
                    &self.connection_settings.server_type,
                ).as_str(),
                TextType::Disabled,
            ))
            .push(
                Row::new().spacing(8).padding(8).push(btn_view).push(
                    Column::new()
                        .push(self.log_view.view(JsonDiagSessionMsg::ClearLogs))
                        .width(Length::FillPortion(1)),
                ),
            )
            .into()
    }

    fn update(&mut self, msg: &Self::msg) -> Option<Self::msg> {
        //self.log_view.clear_logs();
        match msg {
            JsonDiagSessionMsg::ReadErrors => match self.server.read_errors() {
                Ok(res) => {
                    self.logged_dtcs = res.clone();
                    if res.is_empty() {
                        self.log_view.add_msg("No ECU Errors", LogType::Info);
                    } else {
                        self.log_view
                            .add_msg(format!("Found {} errors", res.len()), LogType::Warn);
                        for e in &res {
                            let desc = self
                                .ecu_data
                                .errors
                                .clone()
                                .into_iter()
                                .find(|x| x.error_name.ends_with(e.error.as_str()));

                            let envs = desc.clone().map(|x| x.envs).unwrap_or_default();
                            let mut env_string = String::from("\n");
                            if envs.len() > 0 {
                                if let Ok(args) = self.server.get_dtc_env_data(e) {
                                    for e in &envs {
                                        match e.decode_value_to_string(&args) {
                                            Ok(s) => {env_string.push_str(&format!("---{}: {}\n", e.name, s))},
                                            Err(err) => env_string.push_str(&format!("---{}: {:?}\n", e.name, err))
                                        }
                                    }
                                    println!("{} - {:02X?}", &desc.clone().unwrap().error_name ,args);
                                }
                            }


                            let mut err_txt = match &desc {
                                Some(d) => format!("{} - {}", e.error, d.description),
                                None => format!("{} - Unknown description", e.error),
                            };
                            if env_string.len() > 1 {
                                err_txt.push_str(&env_string);
                            }
                            self.log_view.add_msg(err_txt, LogType::Warn)
                        }
                    }
                }
                Err(e) => {
                    self.log_view.add_msg(
                        format!("Error reading ECU Errors: {}", e.get_text()),
                        LogType::Error,
                    );
                    self.logged_dtcs.clear();
                }
            },
            JsonDiagSessionMsg::ClearErrors => {
                match self.server.clear_errors() {
                    Ok(_) => {
                        self.log_view.add_msg("Clear ECU Errors OK!", LogType::Info);
                        self.logged_dtcs.clear();
                    },
                    Err(e) => self.log_view.add_msg(
                        format!("Error clearing ECU Errors: {}", e.get_text()),
                        LogType::Error,
                    ),
                }
            }

            JsonDiagSessionMsg::Selector(s) => match s {
                SelectorMsg::PickLoopService(l) => self.looping_service = Some(l.clone()),
                SelectorMsg::StopLoopService => {
                    self.looping_service = None;
                    return self.service_selector.update(s);
                }
                _ => return self.service_selector.update(s),
            },

            JsonDiagSessionMsg::ExecuteService(s, args) => {
                println!("Exec {}", s.inner.borrow().name);
                match s.exec(args, &mut self.server) {
                    Ok(res) => self.log_view.add_log(
                        format!(
                            "{} ({}):",
                            s.inner.borrow().name,
                            s.inner.borrow().description
                        ),
                        s.args_to_string(&res),
                        LogType::Info,
                    ),
                    Err(e) => self.log_view.add_msg(
                        format!("Error executing {}: {:?}", s.inner.borrow().name, e).as_str(),
                        LogType::Error,
                    ),
                }
            }
            JsonDiagSessionMsg::ClearLogs => self.log_view.clear_logs(),
            JsonDiagSessionMsg::LoopRead(_) => {
                if let Some(s) = &self.looping_service {
                    if let Ok(res) = s.exec(&[], &mut self.server) {
                        self.looping_text = format!(
                            "{}({})\n->{}",
                            s.inner.borrow().name,
                            s.inner.borrow().description,
                            s.args_to_string(&res)
                        )
                    }
                }
            }
            _ => todo!(),
        }
        None
    }

    fn subscription(&self) -> iced::Subscription<Self::msg> {
        if self.looping_service.is_some() {
            return time::every(std::time::Duration::from_millis(500))
                .map(JsonDiagSessionMsg::LoopRead);
        }
        Subscription::none()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceRef {
    // Read data from ECU
    inner: RefCell<Service>,
}

impl Eq for ServiceRef {}

impl ServiceRef {
    pub fn match_query(&self, q: &str) -> bool {
        return self.inner.borrow().name.to_lowercase().contains(q);
    }

    pub fn require_input(&self) -> bool {
        return !self.inner.borrow().input_params.is_empty();
    }

    pub fn exec(&self, replace_args: &[u8], server: &mut DiagServer) -> ProtocolResult<Vec<u8>> {
        let p = &self.inner.borrow().payload;
        let mut args = if p.is_empty() {
            Vec::new()
        } else {
            Vec::from(&p[1..])
        };
        if !replace_args.is_empty() && replace_args.len() <= args.len() {
            for (pos, x) in replace_args.iter().enumerate() {
                args[pos] |= x;
            }
        }
        server.run_cmd(self.inner.borrow().payload[0], &args)
    }

    pub fn args_to_string(&self, args: &[u8]) -> String {
        let outputs = &self.inner.borrow().output_params;
        if outputs.is_empty() {
            "OK".into()
        } else {
            let mut res: String = String::new();
            for o in outputs {
                match o.decode_value_to_string(args) {
                    Ok(r) => res.push_str(format!("{}: {}\n", o.name, r).as_str()),
                    Err(e) => {
                        res.push_str(format!("Error decoding {}: {:?}\n", o.name, e).as_str())
                    }
                }
            }
            res.remove(res.len() - 1);
            res
        }
    }
}

impl ToString for ServiceRef {
    fn to_string(&self) -> String {
        self.inner.borrow().name.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectorMsg {
    ViewRead,
    ViewWrite,
    ViewActuation,
    PickService(ServiceRef),
    PickLoopService(ServiceRef),
    StopLoopService,
    BeginLoopService,
    ExecService,
    Search(String),
}

#[derive(Debug, Clone)]
pub struct ServiceSelector {
    read_services: Vec<ServiceRef>,
    write_services: Vec<ServiceRef>,
    actuation_services: Vec<ServiceRef>,

    shown_services: Vec<ServiceRef>,

    query_string: String,

    r_btn: iced::button::State,
    w_btn: iced::button::State,
    a_btn: iced::button::State,
    execb: iced::button::State,
    l_btn: iced::button::State,
    is_loop: bool,
    args: Vec<u8>,

    s_bar: iced::text_input::State,

    input_require: bool,
    can_execute: bool,

    selected_service: Option<ServiceRef>,
    picker: iced::pick_list::State<ServiceRef>,

    view_selection: [bool; 3], // Read, Write, Actuation
}

impl ServiceSelector {
    pub fn new(r: Vec<ServiceRef>, w: Vec<ServiceRef>, a: Vec<ServiceRef>) -> Self {
        println!(
            "{} Read services, {} Write services, {} Actuation services",
            r.len(),
            w.len(),
            a.len()
        );

        Self {
            read_services: r.clone(),
            write_services: w,
            actuation_services: a,
            query_string: String::new(),
            r_btn: Default::default(),
            w_btn: Default::default(),
            a_btn: Default::default(),
            s_bar: Default::default(),
            picker: Default::default(),
            execb: Default::default(),
            l_btn: Default::default(),
            args: Vec::new(),
            selected_service: None,
            view_selection: [true, false, false], // Read is default view
            shown_services: r,
            can_execute: false,
            input_require: false,
            is_loop: false,
        }
    }

    pub fn view(&mut self) -> iced::Element<SelectorMsg> {
        let r_btn = match self.view_selection[0] {
            false => button_outlined(&mut self.r_btn, "Read", ButtonType::Info),
            true => button_coloured(&mut self.r_btn, "Read", ButtonType::Info),
        };

        let w_btn = match self.view_selection[1] {
            false => button_outlined(&mut self.w_btn, "Write", ButtonType::Info),
            true => button_coloured(&mut self.w_btn, "Write", ButtonType::Info),
        };

        let a_btn = match self.view_selection[2] {
            false => button_outlined(&mut self.a_btn, "Actuate", ButtonType::Info),
            true => button_coloured(&mut self.a_btn, "Actuate", ButtonType::Info),
        };

        let search_bar = text_input(
            &mut self.s_bar,
            "Search for function",
            self.query_string.as_str(),
            SelectorMsg::Search,
        );

        let btn_row = Row::new()
            .spacing(5)
            .width(Length::Fill)
            .padding(5)
            .push(
                r_btn
                    .on_press(SelectorMsg::ViewRead)
                    .width(Length::FillPortion(1)),
            )
            .push(
                w_btn
                    .on_press(SelectorMsg::ViewWrite)
                    .width(Length::FillPortion(1)),
            )
            .push(
                a_btn
                    .on_press(SelectorMsg::ViewActuation)
                    .width(Length::FillPortion(1)),
            );

        let mut content_view = if self.shown_services.is_empty() {
            Column::new()
                .push(text("No functions match your query", TextType::Normal))
                .spacing(5)
                .width(Length::Fill)
                .padding(5)
        } else {
            Column::new()
                .spacing(5)
                .width(Length::Fill)
                .padding(5)
                .push(text(
                    format!("{} function(s) match your query", self.shown_services.len()).as_str(),
                    TextType::Normal,
                ))
                .push(picklist(
                    &mut self.picker,
                    &self.shown_services,
                    self.selected_service.clone(),
                    SelectorMsg::PickService,
                ))
        };

        if let Some(curr_service) = &self.selected_service {
            content_view = content_view.push(text(
                format!("Description: {}", curr_service.inner.borrow().description).as_str(),
                TextType::Normal,
            ));

            if self.input_require {
                for x in &curr_service.inner.borrow().input_params {
                    content_view = content_view.push(text(
                        format!("Input {} Required. Type: {:?}", x.name, x.data_format).as_str(),
                        TextType::Normal,
                    ))
                }
            }

            if !self.is_loop {
                if self.can_execute {
                    let text = if self.view_selection[0] {
                        "Read "
                    } else if self.view_selection[1] {
                        "Write "
                    } else {
                        "Actuate "
                    };
                    content_view = content_view.push(
                        button_coloured(
                            &mut self.execb,
                            format!("{}{}", text, curr_service.inner.borrow().name).as_str(),
                            ButtonType::Danger,
                        )
                        .on_press(SelectorMsg::ExecService),
                    )
                }
                if self.can_execute == self.view_selection[0] {
                    // Show the graph button
                    content_view = content_view.push(
                        button_coloured(&mut self.l_btn, "Begin graphing", ButtonType::Info)
                            .on_press(SelectorMsg::BeginLoopService),
                    )
                }
            } else {
                // Stop the loop
                content_view = content_view.push(
                    button_coloured(&mut self.l_btn, "Stop graphing", ButtonType::Info)
                        .on_press(SelectorMsg::StopLoopService),
                )
            }
        }

        Column::new()
            .width(Length::Fill)
            .spacing(5)
            .width(Length::Fill)
            .padding(5)
            .push(search_bar.width(Length::Fill))
            .push(btn_row)
            .push(content_view)
            .into()
    }

    pub fn get_shown_services(&self, src: &[ServiceRef]) -> Vec<ServiceRef> {
        if self.query_string.is_empty() {
            return Vec::from(src);
        }
        let lc = self.query_string.to_lowercase();
        src.iter()
            .filter(|x| x.match_query(lc.as_str()))
            .cloned()
            .collect()
    }

    pub fn on_change_items(&mut self) {
        self.selected_service = None;
        self.can_execute = false;
        self.input_require = false;
    }

    pub fn update(&mut self, msg: &SelectorMsg) -> Option<JsonDiagSessionMsg> {
        match &msg {
            SelectorMsg::ViewActuation => {
                self.view_selection = [false, false, true];
                self.shown_services = self.get_shown_services(&self.actuation_services);
                self.on_change_items();
            }
            SelectorMsg::ViewRead => {
                self.view_selection = [true, false, false];
                self.shown_services = self.get_shown_services(&self.read_services);
                self.on_change_items();
            }
            SelectorMsg::ViewWrite => {
                self.view_selection = [false, true, false];
                self.shown_services = self.get_shown_services(&self.write_services);
                self.on_change_items();
            }
            SelectorMsg::Search(s) => {
                let old_len = self.query_string.len();
                self.query_string = s.clone();
                self.shown_services = if old_len < self.query_string.len() {
                    // Adding to existing input
                    self.get_shown_services(&self.shown_services) // Reduce the current array (faster)
                } else {
                    // Reduce the source arrays
                    if self.view_selection[0] {
                        // Read
                        self.get_shown_services(&self.read_services)
                    } else if self.view_selection[1] {
                        // Write
                        self.get_shown_services(&self.write_services)
                    } else {
                        // Actuations
                        self.get_shown_services(&self.actuation_services)
                    }
                }
            }
            SelectorMsg::PickService(s) => {
                if s.require_input() {
                    self.can_execute = false;
                    self.input_require = true;
                } else {
                    self.can_execute = true;
                    self.input_require = false;
                }
                self.selected_service = Some(s.clone());
                println!("{} selected", s.inner.borrow().name);
            }
            SelectorMsg::StopLoopService => {
                self.is_loop = false;
            }
            SelectorMsg::BeginLoopService => {
                if let Some(s) = &self.selected_service {
                    self.is_loop = true;
                    return Some(JsonDiagSessionMsg::Selector(SelectorMsg::PickLoopService(
                        s.clone(),
                    )));
                }
            }
            SelectorMsg::ExecService => {
                return Some(JsonDiagSessionMsg::ExecuteService(
                    self.selected_service.clone().unwrap(),
                    self.args.clone(),
                ))
            }
            _ => {}
        }
        None
    }
}
