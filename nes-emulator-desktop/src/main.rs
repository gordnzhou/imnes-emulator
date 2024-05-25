mod gui;
mod emulator;

use gui::App;

const TITLE: &str = "NES EMULATOR";


/*
TWO MAIN THREADS
GUI thread, emulator thread
- the emulator's entire state should be visible to the GUI thread
- emulator contains an SDL2 audio callback thread to send audio samples (SAME IMPLEMENTATION)
- GUI can update state of emulator (game speed, game input, disable/enable parts, ...)

EMULATOR's STATE:
- public game speed 
- audio sample rate + buffer size (CONSTANT FOR NOW) 
- controller state(s) (changable via a function)
- emulation cycles elapsed

- a reset function for Emulator, which does NOT reset 
window scale, speed, sample rate, nor buffer size
- loading a cartridge also calls the reset function after it is loaded


GUI STATE
- game display FPS
- game display window scale 
- buttons
- all window-related state
- load and export save button. Load will be buffered by the
emulator until the time reset is called (assuming the save file name matches game file name)
*/

fn main() {
    let mut app = match App::new() {
        Ok(app) => app,
        Err(e) => panic!("Error Starting App: {}", e.to_string())
    };
    
    app.run_app(TITLE)
}