use chrono::{DateTime, Local};
use imgui::Ui;

/// Keeps a record of logged events and displays them in Ui
pub struct Logger {
    event_log: Vec<Event>
}

impl Logger {
    pub fn new() -> Self {
        Self {
            event_log: Vec::new(),
        }
    }

    pub fn log_event(&mut self, message: &str) {
        self.event_log.push(Event { 
            timestamp: Local::now(), 
            event_type: EventType::Event, 
            message: String::from(message)
        });
    }
 
    pub fn log_error(&mut self, message: &str) {
        self.event_log.push(Event { 
            timestamp: Local::now(), 
            event_type: EventType::Error, 
            message: String::from(message)
        });
    }

    pub fn display_event_log(&self, ui: &Ui) {
        ui.window("Event Log")
            .size([800.0, 200.0], imgui::Condition::FirstUseEver)
            .position([300.0, 600.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.child_window("Event Log Child")
                    .always_vertical_scrollbar(true)
                    .build(|| {
                        for event in &self.event_log {
                            let timestamp = event.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

                            match event.event_type {
                                EventType::Event => {
                                    let text = format!("[{}] {}", timestamp, event.message);
                                    ui.text_colored([1.0, 1.0, 1.0, 1.0], text);
                                },
                                EventType::Error => {
                                    let text = format!("[{}] (error) {}", timestamp, event.message);
                                    ui.text_colored([1.0, 0.7, 0.7, 1.0], text)
                                },
                            }
                        }
                    })
            });
    }
}

struct Event {
    timestamp: DateTime<Local>,
    event_type: EventType,
    message: String
}

enum EventType {
    Event,
    Error,
}