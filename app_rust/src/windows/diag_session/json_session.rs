use crate::commapi::{
    iface::{InterfaceConfig, InterfaceType, PayloadFlag, IFACE_CFG},
    protocols::{kwp2000::read_ecu_identification, DiagCfg},
};
use common::schema::{
    diag::{dtc::ECUDTC, service::Service},
    variant::{ECUVariantDefinition, ECUVariantPattern},
    ConType, Connection, OvdECU,
};
use core::panic;
use iced::{time, Align, Column, Length, Row, Subscription};
use std::{cell::RefCell, time::Instant, vec};

use crate::{
    commapi::{
        comm_api::ComServer,
        protocols::{DTCState, DiagProtocol, DiagServer, ProtocolResult},
    },
    themes::{
        button_coloured, button_outlined, picklist, text, text_input, title_text, ButtonType,
        TextType,
    },
    widgets::table::{Table, TableMsg},
};

use super::{
    log_view::{LogType, LogView},
    DiagMessageTrait, SessionError, SessionResult, SessionTrait,
};

const TABLE_DTC: usize = 0;
const ENV_TABLE: usize = 1;
const INFO_TABLE_ID: usize = 2;

const MAX_TABLES: usize = 3;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonDiagSessionMsg {
    ReadErrors,
    ClearErrors,
    ReadInfo,
    ExecuteService(ServiceRef, Vec<u8>),
    ClearLogs,
    Selector(SelectorMsg),
    LoopRead(Instant),
    Navigate(TargetPage),
    Select(usize, usize, usize),
}

impl From<SelectorMsg> for JsonDiagSessionMsg {
    fn from(x: SelectorMsg) -> Self {
        JsonDiagSessionMsg::Selector(x)
    }
}

impl DiagMessageTrait for JsonDiagSessionMsg {
    fn is_back(&self) -> bool {
        match self {
            JsonDiagSessionMsg::Navigate(s) => s == &TargetPage::Home,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DisplayableDTC {
    code: String, // DTC Itself
    summary: String,
    desc: String,
    state: DTCState,
    mil_on: bool,
    envs: Vec<(String, String)>, // Key, Value
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TargetPage {
    Home,
    Main,
    ECUInfo,
    Error,
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
    service_selector: ServiceSelector,
    looping_text: String,
    looping_service: Option<ServiceRef>, // Allow only read-only services to be loop read
    logged_dtcs: Vec<DisplayableDTC>,    // DTCs stored on ECU,
    btn1: iced::button::State,
    btn2: iced::button::State,
    btn3: iced::button::State,
    page_state: TargetPage,
    scroll_state1: iced::scrollable::State,
    scroll_state2: iced::scrollable::State,
    tables: Vec<Table>,
}

impl JsonDiagSession {
    pub fn new(
        comm_server: Box<dyn ComServer>,
        ecu_data: OvdECU,
        connection_settings: Connection,
    ) -> SessionResult<Self> {
        let diag_server_type = match connection_settings.server_type {
            common::schema::ServerType::UDS => DiagProtocol::UDS,
            common::schema::ServerType::KWP2000 => DiagProtocol::KWP2000,
        };
        println!("Detect. ECU uses {:?}", diag_server_type);

        // TODO K-Line KWP2000
        // For now, Diag server ONLY supports ISO-TP, not LIN!
        let create_server = match connection_settings.connection_type {
            ConType::ISOTP {
                blocksize,
                st_min,
                ext_isotp_addr,
                ext_can_addr,
            } => {
                let mut cfg = InterfaceConfig::new();
                cfg.add_param(IFACE_CFG::BAUDRATE, connection_settings.baud);
                cfg.add_param(IFACE_CFG::EXT_CAN_ADDR, ext_can_addr as u32);
                cfg.add_param(IFACE_CFG::EXT_ISOTP_ADDR, ext_isotp_addr as u32);
                cfg.add_param(IFACE_CFG::ISOTP_BS, blocksize);
                cfg.add_param(IFACE_CFG::ISOTP_ST_MIN, st_min);

                let diag_cfg = DiagCfg {
                    send_id: connection_settings.send_id,
                    recv_id: connection_settings.recv_id,
                    global_id: connection_settings.global_send_id,
                };

                let tx_flags = vec![PayloadFlag::ISOTP_PAD_FRAME];
                DiagServer::new(
                    diag_server_type,
                    &comm_server,
                    InterfaceType::IsoTp,
                    cfg,
                    Some(tx_flags),
                    diag_cfg,
                )
            }
            ConType::LIN { .. } => {
                return Err(SessionError::Other(
                    "K-Line is not implemented at this time".into(),
                ))
            }
        };

        match create_server {
            Ok(server) => {
                println!("Server started");
                let variant = server.get_variant_id()? as u32;
                let mut unknown_variant = false;
                let ecu_varient = ecu_data
                    .variants
                    .clone()
                    .into_iter()
                    .find(|x| {
                        x.clone()
                            .patterns
                            .into_iter()
                            .any(|p| p.vendor_id == variant)
                    })
                    .unwrap_or_else(|| {
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
                println!(
                    "ECU Variant: {} (Vendor: {})",
                    ecu_varient.name, pattern.vendor
                );

                let read_functions: Vec<ServiceRef> = ecu_varient
                    .downloads
                    .iter()
                    .clone()
                    .map(|s| ServiceRef {
                        inner: RefCell::new(s.clone()),
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
                    btn1: iced::button::State::default(),
                    btn2: iced::button::State::default(),
                    btn3: iced::button::State::default(),
                    looping_service: None,
                    looping_text: String::new(),
                    logged_dtcs: Vec::new(),
                    page_state: TargetPage::Main,
                    scroll_state1: iced::scrollable::State::default(),
                    scroll_state2: iced::scrollable::State::default(),
                    tables: vec![Table::default(); MAX_TABLES],
                })
            }
            Err(e) => {
                eprintln!("Could not setup diag server");
                Err(SessionError::ServerError(e))
            }
        }
    }
}

impl JsonDiagSession {
    pub fn draw_main_ui(&mut self) -> iced::Element<JsonDiagSessionMsg> {
        let mut btn_view = Column::new()
            .push(
                button_outlined(&mut self.btn1, "ECU Information", ButtonType::Primary)
                    .on_press(JsonDiagSessionMsg::ReadInfo),
            )
            .width(Length::FillPortion(1))
            .push(
                button_outlined(&mut self.btn2, "Read errors", ButtonType::Primary)
                    .on_press(JsonDiagSessionMsg::ReadErrors),
            )
            .width(Length::FillPortion(1));
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
            ).into()
    }

    pub fn draw_error_ui(&mut self) -> iced::Element<JsonDiagSessionMsg> {
        // Create a table of errors
        // Top row (Clear + back button)

        let mut content = Column::new()
            .padding(8)
            .spacing(8)
            .align_items(Align::Center);

        let header = Row::new()
            .padding(8)
            .spacing(8)
            .align_items(Align::Center)
            .push(title_text("ECU Error view", crate::themes::TitleSize::P3));

        // Clear btn
        let read_btn = button_outlined(&mut self.btn1, "Read errors", ButtonType::Primary)
            .on_press(JsonDiagSessionMsg::ReadErrors);

        let mut clear_btn = button_outlined(&mut self.btn2, "Clear errors", ButtonType::Primary);
        if self.logged_dtcs.len() > 0 {
            clear_btn = clear_btn.on_press(JsonDiagSessionMsg::ClearErrors);
        };

        let btn_row = Row::new()
            .padding(8)
            .spacing(8)
            .push(read_btn)
            .push(clear_btn)
            .push(iced::Space::with_width(Length::Fill))
            .push(
                button_outlined(&mut self.btn3, "Back", ButtonType::Primary)
                    .on_press(JsonDiagSessionMsg::Navigate(TargetPage::Main)),
            );

        // Like Vediamo. 2 tables. 1 with error list, 1 with env data for current selected DTC

        content = content.push(header);
        content = content.push(btn_row);

        if self.logged_dtcs.len() > 0 {
            content = content.align_items(Align::Start);

            for (id, table) in self.tables.iter_mut().enumerate() {
                match id {
                    TABLE_DTC => {
                        content = content.push(
                            table
                                .view()
                                .map(|x| JsonDiagSessionMsg::Select(TABLE_DTC, x.0, x.1)),
                        );
                    }
                    ENV_TABLE => {
                        content = content.push(title_text(
                            "Freeze frame data",
                            crate::themes::TitleSize::P4,
                        ));
                        content = content.push(
                            table
                                .view()
                                .map(|x| JsonDiagSessionMsg::Select(ENV_TABLE, x.0, x.1)),
                        );
                    }
                    _ => {}
                }
            }
            content.into()
        } else {
            //text("No Diagnostic trouble codes found", TextType::Normal)
            content.push(text("No DTCs", TextType::Normal)).into()
        }
    }

    pub fn draw_info_ui(&mut self) -> iced::Element<JsonDiagSessionMsg> {
        let content = Column::new()
            .padding(8)
            .spacing(8)
            .align_items(Align::Center);

        let header = Row::new()
            .padding(8)
            .spacing(8)
            .align_items(Align::Center)
            .push(title_text("ECU Info view", crate::themes::TitleSize::P3));

        let btn_row = Row::new()
            .padding(8)
            .spacing(8)
            .push(iced::Space::with_width(Length::Fill))
            .push(
                button_outlined(&mut self.btn1, "Back", ButtonType::Primary)
                    .on_press(JsonDiagSessionMsg::Navigate(TargetPage::Main)),
            );

        content
            .push(header)
            .push(btn_row)
            .push(
                self.tables[INFO_TABLE_ID]
                    .view()
                    .map(|_| JsonDiagSessionMsg::Select(INFO_TABLE_ID, 0, 0)),
            )
            .into()
    }
}

impl SessionTrait for JsonDiagSession {
    type msg = JsonDiagSessionMsg;
    fn view(&mut self) -> iced::Element<Self::msg> {
        match self.page_state {
            TargetPage::Main => self.draw_main_ui(),
            TargetPage::Error => self.draw_error_ui(),
            TargetPage::ECUInfo => self.draw_info_ui(),
            _ => panic!("????"),
        }
        .into()
    }

    fn update(&mut self, msg: &Self::msg) -> Option<Self::msg> {
        //self.log_view.clear_logs();
        match msg {
            JsonDiagSessionMsg::Navigate(target) => self.page_state = *target,
            JsonDiagSessionMsg::ReadInfo => {
                let header: Vec<String> = vec!["".into(), "".into()];
                let mut params: Vec<Vec<String>> = Vec::new();
                params.push(vec!["Name".into(), self.ecu_text.0.clone()]);
                params.push(vec!["Description".into(), self.ecu_text.1.clone()]);
                params.push(vec!["Software".into(), self.ecu_data.name.clone()]);
                params.push(vec!["Manufacture".into(), self.pattern.vendor.clone()]);
                if let Some(kwp) = self.server.into_kwp() {
                    if let Ok(res) = read_ecu_identification::read_dcx_mmc_id(kwp) {
                        params.push(vec!["Part number".into(), res.part_number.clone()]);
                        params.push(vec![
                            "Hardware version".into(),
                            res.hardware_version.clone(),
                        ]);
                        params.push(vec![
                            "Software version".into(),
                            res.software_version.clone(),
                        ]);
                    } else {
                        params.push(vec!["Part number".into(), "Unknown".into()]);
                        params.push(vec!["Hardware version".into(), "Unknown".into()]);
                        params.push(vec!["Software version".into(), "Unknown".into()]);
                    }

                    if let Ok(res) = read_ecu_identification::read_dcs_id(kwp) {
                        params.push(vec![
                            "Hardware build date (WW/YY)".into(),
                            res.hardware_build_date.clone(),
                        ]);
                        params.push(vec![
                            "Software build date (WW/YY)".into(),
                            res.software_written_date.clone(),
                        ]);
                        params.push(vec![
                            "Production date (DD/MM/YY)".into(),
                            res.production_date.clone(),
                        ]);
                    } else {
                        params.push(vec!["Hardware build date (WW/YY)".into(), "Unknown".into()]);
                        params.push(vec!["Software build date (WW/YY)".into(), "Unknown".into()]);
                        params.push(vec!["Production date (DD/MM/YY)".into(), "Unknown".into()]);
                    }
                }

                self.tables[INFO_TABLE_ID] = Table::new(header, params, vec![400, 400], false, 900);
                return Some(JsonDiagSessionMsg::Navigate(TargetPage::ECUInfo));
            }
            JsonDiagSessionMsg::ReadErrors => match self.server.read_errors() {
                Ok(res) => {
                    let dtc_list = self.ecu_data.errors.clone();
                    self.logged_dtcs = res
                        .iter()
                        .map(|raw_dtc| {
                            let ecu_dtc = dtc_list
                                .clone()
                                .into_iter()
                                .find(|x| x.error_name.ends_with(&raw_dtc.error))
                                .unwrap_or(ECUDTC {
                                    error_name: raw_dtc.error.clone(),
                                    summary: "UNKNOWN ERROR".into(),
                                    description: "UNKNOWN DTC".into(),
                                    envs: Vec::new(),
                                });
                            let mut res = DisplayableDTC {
                                code: ecu_dtc.error_name.clone(),
                                summary: ecu_dtc.summary.clone(),
                                desc: ecu_dtc.description.clone(),
                                state: raw_dtc.state,
                                mil_on: raw_dtc.check_engine_on,
                                envs: Vec::new(),
                            };

                            if ecu_dtc.envs.len() > 0 {
                                // If parsable freeze frame data exists, then read it
                                if let Ok(args) = self.server.get_dtc_env_data(raw_dtc) {
                                    for e in &ecu_dtc.envs {
                                        match e.decode_value_to_string(&args) {
                                            Ok(s) => res.envs.push((e.name.clone(), s)),
                                            Err(err) => {
                                                eprintln!(
                                                    "Warning could not decode param: {:?}",
                                                    err
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                            res
                        })
                        .collect();
                    let entries: Vec<Vec<String>> = self
                        .logged_dtcs
                        .iter()
                        .map(|dtc| {
                            vec![
                                dtc.code.clone(),
                                dtc.desc.clone(),
                                format!("{:?}", dtc.state),
                                if dtc.mil_on {
                                    "YES".into()
                                } else {
                                    "NO ".into()
                                },
                            ]
                        })
                        .collect();

                    let table = Table::new(
                        vec![
                            "Error".into(),
                            "Description".into(),
                            "State".into(),
                            "MIL on".into(),
                        ],
                        entries,
                        vec![200, 600, 150, 100],
                        true,
                        400,
                    );
                    self.tables[TABLE_DTC] = table;
                    if self.page_state == TargetPage::Main {
                        // Goto DTC View!
                        return Some(JsonDiagSessionMsg::Navigate(TargetPage::Error));
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
            JsonDiagSessionMsg::ClearErrors => match self.server.clear_errors() {
                Ok(_) => {
                    self.log_view.add_msg("Clear ECU Errors OK!", LogType::Info);
                    self.logged_dtcs.clear();
                }
                Err(e) => self.log_view.add_msg(
                    format!("Error clearing ECU Errors: {}", e.get_text()),
                    LogType::Error,
                ),
            },

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
            JsonDiagSessionMsg::Select(table_id, x, y) => {
                // update the table!
                self.tables[*table_id].update(&TableMsg(*x, *y));
                if *table_id == TABLE_DTC {
                    let header = vec!["Parameter".into(), "Value".into()];
                    let mut values: Vec<Vec<String>> = Vec::new();
                    for (name, v) in &self.logged_dtcs[*y].envs {
                        values.push(vec![name.clone(), v.clone()]);
                    }
                    self.tables[ENV_TABLE] = Table::new(header, values, vec![400, 200], false, 300);
                }
            }
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
