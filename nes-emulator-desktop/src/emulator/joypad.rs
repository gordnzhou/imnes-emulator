use imgui::Ui;
use winit::{event::ElementState, keyboard::{KeyCode, PhysicalKey}};

const DEFAULT_RIGHT_KEY: KeyCode = KeyCode::ArrowRight;
const DEFAULT_LEFT_KEY: KeyCode = KeyCode::ArrowLeft;
const DEFAULT_DOWN_KEY: KeyCode = KeyCode::ArrowDown;
const DEFAULT_UP_KEY: KeyCode = KeyCode::ArrowUp;
const DEFAULT_START_KEY: KeyCode = KeyCode::Enter;
const DEFAULT_SELECT_KEY: KeyCode = KeyCode::ShiftLeft;
const DEFAULT_A_KEY: KeyCode = KeyCode::KeyX;
const DEFAULT_B_KEY: KeyCode = KeyCode::Space;

pub struct Joypad {
    polling_key: Option<u8>,
    current_key: Option<KeyCode>,

    default_key_settings: [KeySetting; 8],
    key_settings: [KeySetting; 8],

    key_state: u8,
}

impl Joypad {
    pub fn new() -> Self {
        let default_key_settings = [
            KeySetting { key_code: DEFAULT_RIGHT_KEY, name: String::from("Right"), bit: 0 },
            KeySetting { key_code: DEFAULT_LEFT_KEY, name: String::from("Left"), bit: 1 },
            KeySetting { key_code: DEFAULT_DOWN_KEY, name: String::from("Down"), bit: 2 },
            KeySetting { key_code: DEFAULT_UP_KEY, name: String::from("Up"), bit: 3 },
            KeySetting { key_code: DEFAULT_START_KEY, name: String::from("Start"), bit: 4 },
            KeySetting { key_code: DEFAULT_SELECT_KEY, name: String::from("Select"), bit: 5 },
            KeySetting { key_code: DEFAULT_A_KEY, name: String::from("A"), bit: 6 },
            KeySetting { key_code: DEFAULT_B_KEY, name: String::from("B"), bit: 7 },
        ];

        Self {
            polling_key: None,
            current_key: None,

            key_settings: default_key_settings.clone(),
            default_key_settings,

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

        let mut mask = 0;
        if let PhysicalKey::Code(key) = physical_key {
            for key_setting in &self.key_settings {
                if key == key_setting.key_code {
                    mask = 1 << key_setting.bit;
                    break;
                }
            }
        }

        let old_joypad = self.key_state;

        if pressed {
            self.key_state |= mask;
        } else {
            self.key_state &= !mask;
        }

        self.key_state != old_joypad
    }

    pub fn show_key_settings(&mut self, ui: &Ui) {

        let change_key = |key_setting: &KeySetting| -> bool {
            ui.text(format!("Current {} Key:\n{:?}", key_setting.name, key_setting.key_code));
            ui.same_line_with_spacing(10.0, 300.0);
            ui.button(if self.polling_key == Some(key_setting.bit) { 
                String::from("Press any Key...") 
            }  else { 
                format!("Set {} Key", key_setting.name)
            })
        };

        for key_setting in &self.key_settings {
            if change_key(key_setting) {
                self.polling_key = Some(key_setting.bit);
                break;
            }
        }

        if let Some(current_key) = self.current_key {

            if let Some(polling_key) = self.polling_key {
                
                let mut old_key = None;
                let mut new_key = None;

                for key_setting in &mut self.key_settings {
                    if polling_key == key_setting.bit {
                        new_key = Some(key_setting);
                    } else if current_key == key_setting.key_code {
                        old_key = Some(key_setting);
                    }
                }

                match (old_key, new_key) {
                    (Some(old_key), Some(new_key)) => {
                        old_key.key_code = new_key.key_code;  
                        new_key.key_code = current_key;
                    },
                    (None, Some(new_key)) => new_key.key_code = current_key,
                    _ => {}
                }

                self.polling_key = None;
            }
        }

        ui.modal_popup("Same Key Warning", || {
            ui.text(format!("This key is already being used!"));
            
            if ui.button("OK") {
                ui.close_current_popup();
            }
        });
    }


    pub fn reset_keys(&mut self) {
        self.key_state = 0;
        self.key_settings = self.default_key_settings.clone();
    }

    pub fn get_key_state(&self) -> u8 {
        self.key_state
    }
}

#[derive(Clone)]
struct KeySetting {
    name: String,
    key_code: KeyCode,
    bit: u8
}