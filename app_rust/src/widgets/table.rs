use std::marker::PhantomData;

use iced::{Column, Element, Length, Row, Scrollable, scrollable};
use iced_native::Widget;

use crate::themes::{ButtonType, TextType, button_coloured, button_outlined, button_table, text};

#[derive(Debug, Copy, Clone)]
pub struct TableMsg(pub usize, pub usize);

#[derive(Debug, Clone, Copy)]
struct Position {
    x: usize,
    y: usize,
    state: iced::button::State,
}

#[derive(Debug, Clone, Default)]
pub struct Table {
    header_row: Vec<String>,
    text_matrix: Vec<Vec<String>>,
    states: Vec<Position>,
    selectable: bool,
    max_height: usize,
    scroll_sate: scrollable::State,
    default_text: String,
    selected_row: usize,
    widths: Vec<u16>
}

impl Table {
    pub fn new(header: Vec<String>, matrix: Vec<Vec<String>>, widths: Vec<u16>,selectable: bool, max_height: usize) -> Self {
        let mut tmp_matrix: Vec<Vec<String>> = Vec::new();
        if matrix.len() == 0 {
            return Self {
                header_row: header,
                text_matrix: tmp_matrix,
                states: Vec::new(),
                selectable: false,
                max_height,
                scroll_sate: scrollable::State::default(),
                default_text: "No data".into(),
                selected_row: 0,
                widths
            }
        }

        assert!(header.len() == matrix[0].len());
        let mut states = Vec::new();
        for (_, row) in matrix.iter().enumerate() {
            for (x, s) in row.iter().enumerate() {
                if let Some(col) = tmp_matrix.get_mut(x) {
                    col.push(s.clone())
                } else {
                    tmp_matrix.push(vec![s.clone()])
                }
            }
        }

        for (x,s) in tmp_matrix.iter().enumerate() {
            for (y, _) in s.iter().enumerate() {
                states.push(
                    Position {
                        x,
                        y,
                        state: iced::button::State::default()
                    }
                )
            }
        }

        Self {
            header_row: header,
            text_matrix: tmp_matrix,
            states: states,
            selectable,
            max_height,
            scroll_sate: scrollable::State::default(),
            default_text: "No data".into(),
            selected_row: 0,
            widths
        }
    }

    pub fn update(&mut self, msg: &TableMsg) {
        if self.selectable {
            self.selected_row = msg.1
        }
    }

    pub fn set_default_text(&mut self, default: String) {
        self.default_text = default;
    }

    pub fn view(&mut self) -> Element<TableMsg>{

        if self.header_row.is_empty() || self.text_matrix.is_empty() {
            return text(&self.default_text, TextType::Normal).into()
        }

        let mut row = Row::new();

        let mut scroll = Scrollable::new(&mut self.scroll_sate);

        let mut column = Column::new();

        let mut last_idx = 0;
        column = column.push(text(&self.header_row[0], TextType::Normal));
        for position in self.states.iter_mut() {
            if position.x != last_idx {
                // New column!
                row = row.push(column);
                column = Column::new().push(text(&self.header_row[position.x], TextType::Normal));
                last_idx = position.x;
            }

            let selected = self.selectable && self.selected_row == position.y;
            let mut btn = button_table(&mut position.state, &self.text_matrix[position.x][position.y], ButtonType::Danger, selected)
                .width(Length::Units(self.widths[position.x]));
            
            if self.selectable {
                btn = btn.on_press(TableMsg(position.x, position.y));
            }
            column = column.push(btn);
        }
        row = row.push(column); // Last column

        scroll.push(row).padding(4).spacing(4).max_height(self.max_height as u32).into()
    }
}