use std::{collections::VecDeque, time::Instant};

pub trait Graphable {
    fn draw(&self);
}

#[derive(Debug, Default)]
pub struct Graph {
    cache: iced::canvas::Cache,
}

pub struct LineTimeGraph<'a> {
    frame: &'a iced::canvas::Frame,
    points: VecDeque<(u128, f32)>,
    y_bounds: (f32, f32), // Min, Max
    x_label: String,
    y_label: String,
    max_time_ms: u128,
}

impl<'a> LineTimeGraph<'a> {
    pub fn new(frame: &'a iced::canvas::Frame, x_label: String, y_label: String) -> Self {
        Self {
            frame,
            points: VecDeque::new(),
            y_bounds: (0.0, 1.0),
            x_label,
            y_label,
            max_time_ms: 30000, // 30 seconds
        }
    }

    pub fn add_data_point(&mut self, y: f32) {
        // Adjust our bounds
        if y > self.y_bounds.1 {
            self.y_bounds.1 = y
        } else if y < self.y_bounds.0 {
            self.y_bounds.0 = y
        }
        // Push the data point
        let time = Instant::now().elapsed().as_millis();
        self.points.push_back((time, y))
    }
}

impl<'a> Graphable for LineTimeGraph<'a> {
    fn draw(&self) {
        let max_height = self.frame.height();
        let resolution_y = max_height / (self.y_bounds.1 + self.y_bounds.0); // y per px

        let curr_time = Instant::now().elapsed().as_millis(); // Time at x=0

        let resolution_x = self.max_time_ms as f32 / self.frame.width(); // ms per px

        todo!()
    }
}

#[cfg_attr(not(feature = "graph_tests"), ignore)]
pub mod graph_test {
    use iced::{
        canvas::{self, Cursor, Frame, Geometry},
        executor, Application, Canvas, Column, Rectangle, Settings,
    };

    use crate::themes::elements::Container;

    use super::*;

    #[test]
    fn test_plot() {
        let mut settings = Settings::default();
        settings.window.resizable = false;
        settings.window.size = (1366, 768);
        settings.window.min_size = Some((1366, 768));
        TestApp::run(settings);
    }

    #[derive(Debug, Clone, Copy)]
    enum TestMsg {}

    struct TestApp<'a> {
        frame: iced::canvas::Frame,
        graph: LineTimeGraph<'a>,
    }

    impl<'a> iced::Application for TestApp<'a> {
        type Executor = executor::Default;
        type Message = TestMsg;
        type Flags = ();

        fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
            todo!()
            /*
            let f = Frame::new(iced::Size::new(1366.0, 768.0));
            (Self {
                frame: f,
                graph: LineTimeGraph::new(&f, "X axis".into(), "Y axis".into())
            }, iced::Command::none())
            */
        }

        fn title(&self) -> String {
            "Test graph library".into()
        }

        fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
            todo!()
        }

        fn view(&mut self) -> iced::Element<'_, Self::Message> {
            todo!()
            //Container::new(Canvas::new(&mut self.graph).into()
        }
    }

    impl<LineTimeGraph> canvas::Program<LineTimeGraph> for Graph {
        fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
            vec![]
        }
    }
}
