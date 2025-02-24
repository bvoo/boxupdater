# BoxUpdater

A desktop application for easily updating RP2040-based controller firmware.

https://github.com/user-attachments/assets/8b1a876b-e047-4a1d-bd07-34d37e5791a5

## Features

- 🎮 Support for multiple firmware repositories:
  - HayBox
  - HayBox-GRAM
  - GP2040-CE
  - Pico Rectangle

- 🚀 Easy firmware updates:
  - Automatic detection of BOOTSEL mode
  - One-click firmware flashing
  - Optional flash nuke before flashing

- 💻 Modern desktop application:
  - Cross-platform support (Windows, macOS, Linux)
  - Clean and intuitive user interface
  - Version selection from official releases

## Description

BoxUpdater is designed to simplify the process of updating firmware on RP2040-based controllers. It automatically fetches the latest releases from supported firmware repositories and provides a straightforward interface for flashing new firmware to your device.

## Installation

Download the latest release for your operating system from the releases page.

## Usage

1. Launch BoxUpdater
2. Select your controller's firmware from the available repositories
3. Choose the desired firmware version
4. Put your controller in BOOTSEL mode
5. Click to flash the firmware
6. Wait for the process to complete

## Development

This project uses:
- Tauri
- Vue 3
- Rust
- TailwindCSS

To run in development mode:

```bash
pnpm install
pnpm tauri dev
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

See the [LICENSE](LICENSE) file for details.
