<p align="center"><img width="200" src="doc/images/icon.png" alt="v1per logo"></p>

<h1 align="center">V1per Servicing Toolkit</h1>

<p align="center">
  <strong>Android Servicing Toolkit for MediaTek, Unisoc & Utilities</strong>
</p>

## Features

### MediaTek Tools
- **Scatter Flasher** - Flash firmware using scatter file
- **DA Flasher** - Flash via Download Agent
- **Preloader Flash** - Flash via preloader mode
- **Force Fastboot/Recovery/BROM** - Force device to different modes
- **Read Info** - Read device information
- **Partition Operations** - Read, backup, erase partitions
- **Bootloader Unlock** - Unlock device bootloader
- **Root Device** - Root with Magisk
- **FRP Bypass** - Bypass FRP lock

### Unisoc Tools
- **PAC Flasher** - Flash PAC firmware files
- **XML Flasher** - Flash via XML configuration
- **Research Download** - Use Research/Upgrade tool
- **Force Fastboot/Recovery** - Force device modes
- **Read Info** - Read device information
- **Diag Mode** - Enter diagnostic mode
- **Partition Operations** - Read, dump, erase partitions
- **Bootloader Unlock** - Unlock device bootloader
- **Root Device** - Root with Magisk
- **FRP Bypass** - Bypass FRP lock

### Utilities
- **ADB Tools** - Devices, Shell, Reboot, Sideload
- **Fastboot Tools** - Devices, Flash, Reboot, OEM commands
- **Device Info** - Root check, device info, battery, storage
- **Scrcpy** - Mirror Android screen
- **Drivers** - Download device drivers
- **Scatter Editor** - Parse scatter file
- **Anykernel** - Pack anykernel flashable zip

## Tech Stack

- Electron 30+
- Vue 3
- TypeScript

## Supported Platforms

- Windows 7+
- Linux
- macOS

## Development

```bash
# Install dependencies
npm install

# Run development
npm run dev

# Build for production
npm run pack
```

## License

MIT
