use std::{fs, io, path::Path};

use nesemulib::{CartridgeNes, SystemBus};

use crate::logger::Logger;


const SAVE_FOLDER: &str = "saves/";
const ROMS_FOLDER: &str = "roms/";

/// Handles ROM state in emulation, and saving + loading
pub struct RomManager {
    pub auto_save: bool,

    pub bus: Option<SystemBus>,
    pub cartridge_name: Option<String>,

    pub selected_file: usize,
    pub file_names: Vec<String>,
    pub save_folder: String,
    pub roms_folder: String,
}

impl RomManager {
    pub fn new() -> Self {

        let save_folder = String::from(SAVE_FOLDER);
        let roms_folder = String::from(ROMS_FOLDER);
        
        let selected_file = 0;
        let mut file_names: Vec<String> = fs::read_dir(&roms_folder).unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().unwrap().is_file())
            .map(|e| e.file_name().into_string().unwrap())
            .collect();
        file_names.sort(); 

        Self {
            auto_save: true,
            selected_file,
            file_names,
            cartridge_name: None,
            bus: None,
            save_folder,
            roms_folder,
        }
    }

    pub fn unload_cartridge(&mut self, logger: &mut Logger) {
        if let Some(name) = &self.cartridge_name {
            logger.log_event(&format!("Unloaded ROM cartridge: {}", name))
        }

        self.write_save_to_file(logger);

        self.bus = None;
        self.cartridge_name = None;
    }

    pub fn refresh_file_names(&mut self) {
        self.file_names = fs::read_dir(&self.roms_folder).unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().unwrap().is_file())
            .map(|e| e.file_name().into_string().unwrap())
            .collect();
        
        self.file_names.sort(); 
    }

    pub fn load_ines_cartridge(&mut self, file_name: &str, logger: &mut Logger) -> Result<(), io::Error> {
        let cartridge = CartridgeNes::from_ines_file(file_name)?;
        let bus = SystemBus::new(cartridge);

        self.cartridge_name = Some(String::from(file_name));
        self.bus = Some(bus);

        self.load_save_from_file(file_name, logger);

        logger.log_event(&format!("Loaded ROM cartridge: {}", file_name));

        Ok(())
    }

    fn load_save_from_file(&mut self, file_name: &str, logger: &mut Logger) {
        let save_path = self.get_save_path(file_name);

        if let Some(bus) = &mut self.bus {

            match fs::read(&save_path) {
                Ok(save_ram) => if let Err(e) = bus.cartridge.load_save_ram(save_ram) {
                    logger.log_event(&format!("Unable to load save RAM for {}:\n{}", file_name, e));
                } else {
                    logger.log_event(&format!("Successfully loaded save RAM from: {}", save_path))
                }
                Err(e) if bus.cartridge.mapper.get_save_ram().is_some() => {
                    logger.log_error(&format!("Failed to load save RAM for {}:\n{}", file_name, e));
                }
                _ => {}
            };
        }
    }

    pub fn write_save_to_file(&mut self, logger: &mut Logger) {
        let file_name = match &self.cartridge_name {
            Some(name) => name,
            _ => return
        };

        let save_path = self.get_save_path(file_name);

        if let Some(bus) = &mut self.bus {

            if let Some(ram) = bus.cartridge.get_save_ram() {
                if let Err(e) = fs::write(save_path.clone(), ram) {
                    logger.log_error(&format!("Failed to save RAM to {}:\n{}", save_path, e));
                } else {
                    logger.log_event(&format!("Successfully saved RAM to: {}", save_path));
                }
            }
        }
    }

    pub fn do_auto_save(&mut self, logger: &mut Logger) {
        if !self.auto_save {
            return;
        }

        self.write_save_to_file(logger)
    }

    #[inline]
    fn get_save_path(&self, file_name: &str) -> String {
        let file_stem = Path::new(file_name).file_stem().unwrap().to_str().unwrap();
        format!("{}{}.sav", self.save_folder, file_stem)
    }
}