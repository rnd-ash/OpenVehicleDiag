use std::collections::VecDeque;

use iced::{scrollable, Column, Element, Length, Row, Scrollable, Space};

use crate::themes::{button_outlined, text, title_text, ButtonType, TextType};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LogType {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone)]
struct LogOperation {
    request: Option<String>,
    response: Option<String>,
    log_type: LogType,
}

impl LogOperation {
    fn create<T: ToString>(request: Option<T>, response: Option<T>, ltype: LogType) -> Self {
        Self {
            request: request.map(|x| x.to_string()),
            response: response.map(|x| x.to_string()),
            log_type: ltype,
        }
    }

    fn render<'a, T>(&self) -> Element<'a, T>
    where
        T: 'a,
    {
        let mut c = Column::new();
        let text_type = match self.log_type {
            LogType::Error => TextType::Danger,
            LogType::Warn => TextType::Warning,
            LogType::Info => TextType::Normal,
        };
        if let Some(r) = &self.request {
            c = c.push(text(&r, text_type).size(16))
        }
        if let Some(r) = &self.response {
            c = c.push(text(&r, text_type).size(16))
        }
        c.into()
    }
}

#[derive(Debug, Clone)]
pub struct LogView {
    logs: VecDeque<LogOperation>,
    scroll_state: scrollable::State,
    btn_state: iced::button::State,
}

impl LogView {
    pub fn new() -> Self {
        Self {
            logs: VecDeque::new(),
            scroll_state: Default::default(),
            btn_state: Default::default(),
        }
    }

    pub fn view<'a, T: Clone>(&'a mut self, clear_log_msg: T) -> Element<'a, T>
    where
        T: 'a,
    {
        let mut c = Column::new().spacing(5).width(Length::Fill);
        c = c.push(
            Row::new()
                .width(Length::Fill)
                .push(title_text("Log view", crate::themes::TitleSize::P3))
                .push(Space::with_width(Length::Fill))
                .push(
                    button_outlined(&mut self.btn_state, "Clear logs", ButtonType::Success)
                        .on_press(clear_log_msg),
                ),
        );
        let mut s = Scrollable::new(&mut self.scroll_state)
            .width(Length::Fill)
            .height(Length::Fill);
        for l in &self.logs {
            s = s.push(l.render())
        }
        c = c.push(s);
        c.into()
    }

    pub fn add_log<X: ToString>(&mut self, request: X, response: X, ltype: LogType) {
        self.logs
            .push_back(LogOperation::create(Some(request), Some(response), ltype))
    }

    pub fn add_msg<X: ToString>(&mut self, msg: X, ltype: LogType) {
        self.logs
            .push_back(LogOperation::create(Some(msg), None, ltype))
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear()
    }
}
