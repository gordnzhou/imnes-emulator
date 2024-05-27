use winit::{event::ElementState, keyboard::{KeyCode, PhysicalKey}};

pub struct Joypad {
    pub key_state: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            key_state: 0,
        }
    }

    pub fn update_joypad(&mut self, physical_key: PhysicalKey, state: ElementState) -> bool {
        let pressed = matches!(state, ElementState::Pressed);

        // TODO: configurable controls
        let mask = match physical_key {
            PhysicalKey::Code(KeyCode::KeyD) => 0x01,
            PhysicalKey::Code(KeyCode::KeyA) => 0x02,
            PhysicalKey::Code(KeyCode::KeyS) => 0x04,
            PhysicalKey::Code(KeyCode::KeyW) => 0x08,
            PhysicalKey::Code(KeyCode::KeyI) => 0x10,
            PhysicalKey::Code(KeyCode::KeyJ) => 0x20,
            PhysicalKey::Code(KeyCode::KeyK) => 0x40,
            PhysicalKey::Code(KeyCode::KeyL) => 0x80,
            _ => return false
        };

        let old_joypad = self.key_state;

        if pressed {
            self.key_state |= mask;
        } else {
            self.key_state &= !mask;
        }

        self.key_state != old_joypad
    }
}