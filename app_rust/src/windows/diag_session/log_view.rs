use std::collections::VecDeque;

use iced::{Column, Element, Length, Scrollable, scrollable};

use crate::themes::{TextType, text, title_text};



#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LogType {
    Error,
    Warn,
    Info
}

#[derive(Debug, Clone)]
struct LogOperation {
    request: Option<String>,
    response: Option<String>,
    log_type: LogType
}

impl LogOperation {
    fn create<T: ToString>(request: Option<T>, response: Option<T>, ltype: LogType) -> Self {
        Self {
            request: request.map(|x| x.to_string()),
            response: response.map(|x| x.to_string()),
            log_type: ltype
        }
    }

    fn render<'a, T>(&self) -> Element<'a, T> where T: 'a {
        let mut c = Column::new();
        let text_type = match self.log_type {
            LogType::Error => TextType::Danger,
            LogType::Warn => TextType::Warning,
            LogType::Info => TextType::Normal
        };
        if let Some(r) = &self.request {
            c = c.push(text(&r, text_type).size(15))
        }
        if let Some(r) = &self.response {
            c = c.push(text(&r, text_type).size(15))
        }
        c.into()
    }
}


#[derive(Debug, Clone)]
pub struct LogView {
    logs: VecDeque<LogOperation>,
    scroll_state: scrollable::State
}

impl LogView {
    pub fn new() -> Self {
        Self {
            logs: VecDeque::new(),
            scroll_state: scrollable::State::default()
        }
    }

    pub fn view<'a, T>(&'a mut self) -> Element<'a, T> where T: 'a {
        let mut c = Column::new().spacing(5).width(Length::Fill);
        c = c.push(title_text("Log view", crate::themes::TitleSize::P3));
        let mut s = Scrollable::new(&mut self.scroll_state).width(Length::Fill).height(Length::Fill);
        for l in &self.logs {
            s = s.push(l.render())
        }
        c = c.push(s);
        c.into()
    }

    pub fn add_log<X: ToString>(&mut self, request: X, response: X, ltype: LogType) {
        self.logs.push_back(LogOperation::create(Some(request), Some(response), ltype))
    }

    pub fn add_msg<X: ToString>(&mut self, msg: X, ltype: LogType) {
        self.logs.push_back(LogOperation::create(Some(msg), None, ltype))
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear()
    }
}