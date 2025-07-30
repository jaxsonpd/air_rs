/// Handle the terminal ui for the interactive mode
/// 
/// Author: Jack Duignan (JackpDuignan@gmail.com)

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Stylize}, text::Line, widgets::{Block, Cell, Row, Table}, DefaultTerminal, Frame
};

use std::{error::Error, sync::mpsc::Receiver};
use std::time::Duration;

use crate::adsb::packet::AdsbPacket;
use crate::adsb::aircraft::Aircraft;

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
struct App {
    /// Is the application running?
    running: bool,
    aircraft: Vec<Aircraft>,
    num_packets: u32,
}

impl App {
    pub fn new() -> Self {
        App {
            running: false,
            aircraft: Vec::new(),
            num_packets: 0,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal, rx: Receiver<AdsbPacket>) -> Result<(), Box<dyn Error>> {
        self.running = true;


        while self.running {
            while let Ok(packet) = rx.try_recv() {
                self.num_packets += 1;
                let mut handled = false;
                for plane in self.aircraft.iter_mut() {
                    if plane.get_icao() == packet.get_icao() {
                        plane.handle_packet(packet.clone());
                        handled = true;
                        break;
                    }
                }
                if !handled {
                    self.aircraft.push(Aircraft::new(packet.icao));
                    let len = self.aircraft.len();
                    self.aircraft[len-1].handle_packet(packet.clone());
                }
            }
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }

        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(0),
                // Constraint::Length(3),
            ])
            .split(frame.area());


        let title = Line::from(format!("air_rs adsb tracker {}", self.num_packets))
            .bold()
            .light_magenta()
            .centered();
        
        let rows = self.aircraft.iter().map(|plane| {
            let pos = plane.get_geo_position();
            Row::new(vec![
                Cell::from(format!("{:x}", plane.get_icao())),
                Cell::from(format!("{}", plane.get_callsign())),
                Cell::from(format!("{}", plane.get_altitude_ft())),
                Cell::from(pos.clone().map_or_else(|| "n/a".to_string(), |p| format!("{:.6}", p.latitude))),
                Cell::from(pos.map_or_else(|| "n/a".to_string(), |p| format!("{:.6}", p.longitude))),
                Cell::from("n/a"),
                Cell::from(format!("{}", plane.get_age())),
            ])
        });

        let column_widths = [
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(5),
        ];

        let table = Table::new(rows, column_widths)
            .header(Row::new(vec!["ICAO", "Callsign", "Altitude", "Latitude", "Longitude", "Velocity", "Age"]).bold())
            .block(Block::bordered().title(title));

        frame.render_widget(table, layout[0]);
        // frame.render_widget(footer, layout[1]);
    }

        /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<(), Box<dyn Error>> {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

pub fn interactive_display_thread_tui(rx: Receiver<AdsbPacket>) {
    color_eyre::install().expect("Cannot install color eye try stream display mode");
    let terminal = ratatui::init();
    App::new().run(terminal, rx).expect("Interactive mode terminal render died");
    ratatui::restore();
}


