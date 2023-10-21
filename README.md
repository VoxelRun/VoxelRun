<h1 align="center">
  VoxelRun
</h1>

## About

A federated voxel-game inspired by Minecraft. 

## Index
- [Usage](#usage)
  - [System Requirements](#system-requirements)
  - [Build Requirements](#build-requirements)
- [Contributing](#contributing)
- [License](#license)

## Usage

### System Requirements
- Vulkan
- SDL
- libc

### Build requirements
- cargo
- glsl
- vulkan

### Installing

TBA

## Contributing

### Code Structure
- vr-core: Basic code shared between client and server
- vr-server: Server binary
- vr-client: Client binary
- vr-protocol-specs: RFCs for the network protocols
- vr-networking-core: Core transport layers defined in spec
- vr-networking-transport: Implementation of the spec
- vr-networking-federation: Federation implementation
- vr-renderer: Rendering engine
- vr-threading: Utilities for multithreaded tasks

## License

This repository is licensed under GPLv3, further information in [LICENSE](./LICENSE).
