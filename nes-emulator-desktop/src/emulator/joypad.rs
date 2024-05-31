use imgui::Ui;
use winit::{event::ElementState, keyboard::{KeyCode, PhysicalKey}};

// extract default keys to consts
const DEFAULT_RIGHT_KEY: KeyCode = KeyCode::KeyD;
const DEFAULT_LEFT_KEY: KeyCode = KeyCode::KeyA;
const DEFAULT_DOWN_KEY: KeyCode = KeyCode::KeyS;
const DEFAULT_UP_KEY: KeyCode = KeyCode::KeyW;
const DEFAULT_START_KEY: KeyCode = KeyCode::KeyI;
const DEFAULT_SELECT_KEY: KeyCode = KeyCode::KeyJ;
const DEFAULT_A_KEY: KeyCode = KeyCode::KeyK;
const DEFAULT_B_KEY: KeyCode = KeyCode::KeyL;

pub struct Joypad {
    polling_key: Option<u8>,
    current_key: Option<KeyCode>,

    right_key: KeyCode,
    left_key: KeyCode,
    down_key: KeyCode,
    up_key: KeyCode,
    start_key: KeyCode,
    select_key: KeyCode,
    a_key: KeyCode,
    b_key: KeyCode,

    pub key_state: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            polling_key: None,
            current_key: None,

            right_key: DEFAULT_RIGHT_KEY,
            left_key: DEFAULT_LEFT_KEY,
            down_key: DEFAULT_DOWN_KEY,
            up_key: DEFAULT_UP_KEY,
            start_key: DEFAULT_START_KEY,
            select_key: DEFAULT_SELECT_KEY,
            a_key: DEFAULT_A_KEY,
            b_key: DEFAULT_B_KEY,

            key_state: 0,
        }
    }

    pub fn update_joypad(&mut self, physical_key: PhysicalKey, state: ElementState) -> bool {
        let pressed = matches!(state, ElementState::Pressed);

        // Updates the current key pressed for changing controls in settings
        self.current_key = if pressed {
            if let PhysicalKey::Code(key) = physical_key {
                
                Some(key)
            } else {
                None
            }
        } else {
            None
        };

        let mask = if let PhysicalKey::Code(key) = physical_key {
            match key {
                key if key == self.right_key  => 1 << 0,
                key if key == self.left_key   => 1 << 1,
                key if key == self.down_key   => 1 << 2,
                key if key == self.up_key     => 1 << 3,
                key if key == self.start_key  => 1 << 4,
                key if key == self.select_key => 1 << 5,
                key if key == self.a_key      => 1 << 6,
                key if key == self.b_key      => 1 << 7,
                _ => 0
            }
        } else { 0 };

        let old_joypad = self.key_state;

        if pressed {
            self.key_state |= mask;
        } else {
            self.key_state &= !mask;
        }

        self.key_state != old_joypad
    }

    pub fn show_key_settings(&mut self, ui: &Ui) {

        let key_setting = |key_name: &str, id: u8, key: KeyCode| -> bool {
            ui.text(format!("Current {} Key:\n{:?}", key_name, key));
            ui.same_line_with_spacing(10.0, 300.0);
            ui.button(if self.polling_key == Some(id) { 
                String::from("Press any Key...") 
            }  else { 
                format!("Set {} Key", key_name)
            })
        };

        self.polling_key = 
            if key_setting("Right", 0, self.right_key) { Some(0) } 
            else if key_setting("Left", 1, self.left_key) { Some(1) }
            else if key_setting("Down", 2, self.down_key) { Some(2) }
            else if key_setting("Up", 3, self.up_key) { Some(3) }
            else if key_setting("Start", 4, self.start_key) { Some(4) }
            else if key_setting("Select", 5, self.select_key) { Some(5) }
            else if key_setting("A", 6, self.a_key) { Some(6) }
            else if key_setting("B", 7, self.b_key) { Some(7) }
            else { self.polling_key };

        if let Some(current_key) = self.current_key {
            match self.polling_key {
                Some(0) => self.right_key = current_key,
                Some(1) => self.left_key = current_key,
                Some(2) => self.down_key = current_key,
                Some(3) => self.up_key = current_key,
                Some(4) => self.start_key = current_key,
                Some(5) => self.select_key = current_key,
                Some(6) => self.a_key = current_key,
                Some(7) => self.b_key = current_key,
                _ => {}
            }

            self.polling_key = None;
        }
    }


    pub fn reset_keys(&mut self) {
        self.key_state = 0;
        self.right_key = DEFAULT_RIGHT_KEY;
        self.left_key = DEFAULT_LEFT_KEY;
        self.down_key = DEFAULT_DOWN_KEY;
        self.up_key = DEFAULT_UP_KEY;
        self.start_key = DEFAULT_START_KEY;
        self.select_key = DEFAULT_SELECT_KEY;
        self.a_key = DEFAULT_A_KEY;
        self.b_key = DEFAULT_B_KEY;
    }
}