# ImNES = NES Emulator + Rust + ImGui

![cover](images/cover.png)

**ImNES** is NES emulator implemented in Rust. It includes a debugging UI made using ImGui for desktop. The desktop UI has various features such as:
- Inspect the CPU, PPU, and APU state, which includes registers, pattern tables and code disassembly
- View, and enable/disable individual audio channels
- View iNES cartridge details 
- Pause, stop, or restart the emulation, as well as adjust the game speed
- Change key bindings for the joypad

This emulator currently supports the following mappers from iNES 1.0: 
| Mapper | Other Names(s) | Example Games |
|--------|------------|---------------|
| 000    | NROM       | Super Mario Bros., Donkey Kong |
| 001    | SxROM/MMC1 | The Legend of Zelda, Metroid |
| 002    | UxROM      | Mega Man, Duck Tales |
| 003    | CNROM      | Arkanoid, Gradius |
| 004    | TxROM/MMC3 | Super Mario Bros. 3, Kirby's Adventure |
| 007    | AxROM      | Battletoads, Marble Madness |
| 066    | GxROM      | Doraemon, Dragon Power |

This repository also includes a binary in `nes-emulator-sdl2` which is a standalone emulator that does not contain any UI. Running it will require [SDL](https://www.libsdl.org/) to be installed and linked on your local machine.


## Desktop Application Setup 
Before starting, make sure you have [Rust](https://www.rust-lang.org/tools/install) installed and make sure the version is at least **1.79.0-nightly**. 

- Add ROMs to the `/roms` folder. Save data will be automatically placed in the `/saves` folder with the same name as its ROM file, with a `.sav` extension.

- To run the application:
```
cargo run -p imnes-desktop --release
```

By default, the joypad bindings are set to:
- **Left, Right, Up, Down** - Arrow Keys
- **Start** - Enter Key
- **Select** - Left Shift Key
- **A Button** - X Key
- **B Button** - Z Key

## Screenshots

![smb3](images/smb3.png)

![dk](images/dk.png)

![mm3](images/mm3.png)

![nestest](images/nestest.png)  

![smb](images/smb.png)

## Credits
This project wouldn't have been possible without reference from these amazing sources. Thank you!!

- [the NESdev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki)
- [6502 Reference Guide](http://www.6502.org/users/obelisk/6502/reference.html#JSR) for the CPU implementation
- javidx9's [NES Emulator Series on YouTube](https://www.youtube.com/watch?v=nViZg02IMQo&list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf) (Big shoutout to [OneLoneCoder](https://github.com/OneLoneCoder) for his amazing series!)
- [rust-nes-emulator](https://github.com/kamiyaowl/rust-nes-emulator/tree/master)
- and many more...

## Future TODOs
- [ ] 2 Player Joypad Support
- [ ] Game Save States
- [ ] Controller Input Support
- Implementations for more mappers



