# Vendor Directory Structure

This directory organizes all vendor-provided tools, firmware payloads, and unpacked components for the Royal Kludge RK855 (RK68) mechanical keyboard.

---

## Directory Overview

```
vendor/
├── binaries/                  # Original, unmodified vendor executables
│   ├── RK855_configurator.exe # Official configurator installer
│   ├── RK855_firmware_flasher.exe # Official firmware flasher
│   └── README.md              # File hashes and binary metadata
├── configurator/              # Unpacked configurator files
│   ├── app/                   # Standalone runnable application layout
│   │   ├── DeviceDriver.exe   # Win32 MFC configuration executable
│   │   ├── mui.dll            # Multi-language string resource DLL
│   │   ├── KeyboardLayout.xml # Physical 68-key coordinate and scan matrix map
│   │   ├── device.xml         # USB device VID/PID and feature descriptors
│   │   ├── font/              # OpenSans TrueType fonts
│   │   ├── language/          # Localization strings (.lan: 1028, 1033, 2052)
│   │   └── theme1/            # UI icons, keyboard graphics, buttons
│   └── installer/             # Raw InstallShield setup archives & engine DLLs
│       ├── data1.cab / data2.cab # Compressed cabinet payloads
│       ├── setup.exe / setup.ini # Installer bootstrap loader
│       └── support/           # InstallShield support DLLs and helper binaries
└── flasher/                   # Unpacked firmware flasher resources
    └── Settings/
        ├── UISettings.ini     # Flasher configuration & bootloader command
        ├── UISettings_248B.ini# MCU chip profiles
        ├── FWFiles/           # Original firmware hex payload
        └── Image/             # UI bitmaps
```

---

## Reconstituted App Directory (`configurator/app/`)

When installed by the official setup program on Windows, `DeviceDriver.exe` expects its fonts, localized strings, XML hardware maps, and themes in specific subdirectories relative to the executable:
- Fonts are read from `font/`
- Translations are read from `language/%d.lan`
- UI graphics are read from `theme1/`
- Hardware key coordinates are loaded from `KeyboardLayout.xml`
- Device identifiers are loaded from `device.xml`

This reconstructed structure allows testing or running `DeviceDriver.exe` (e.g. via Wine or on Windows) directly without needing to run the InstallShield installer.
