# Rotating Cube - Rust + wgpu

A simple 3D rotating cube demo using **wgpu** (WebGPU/Vulkan) in Rust.
<img width="599" height="508" alt="image" src="https://github.com/user-attachments/assets/16423d00-c0c0-43f5-b9c9-908c44ec5941" />

<img width="653" height="522" alt="image" src="https://github.com/user-attachments/assets/ba4fc7f4-3170-4ce5-90bd-e970c09b42d0" />


## Features

- 3D cube with different colored faces
- Continuous rotation on Y and X axes
- Depth buffering for correct face ordering
- Perspective projection
- Uses Vulkan backend 

## Requirements

### Ubuntu 24.04 + RTX 3070

```bash
# Install system dependencies
sudo apt-get update
sudo apt-get install -y \
    pkg-config \
    libx11-dev \
    libxkbcommon-dev \
    libwayland-dev \
    libvulkan-dev \
    vulkan-tools \
    mesa-vulkan-drivers

# For NVIDIA, ensure you have the proprietary drivers installed
# The driver should include Vulkan support
nvidia-smi  # Check if NVIDIA driver is installed
vulkaninfo --summary  # Check Vulkan support
```

### Rust Toolchain

This project works with **stable Rust** (no nightly required):

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ensure you have the latest stable
rustup update stable
rustup default stable
```

## Building & Running

```bash
# Development build and run
make run

# Or using cargo directly
RUST_LOG=info cargo run

# Release build (faster execution)
make run-release
```

## Controls

- **Close window**: Press the X button or Alt+F4


## Technical Details

### Dependencies

| Crate | Purpose |
|-------|---------|
| `wgpu` | Cross-platform GPU API (Vulkan/Metal/DX12) |
| `winit` | Cross-platform window creation |
| `glam` | Fast math library (matrices, vectors) |
| `bytemuck` | Safe transmutes for GPU data |
| `pollster` | Async runtime for wgpu initialization |

### GPU Backend

The application prioritizes **Vulkan** backend:

```rust
let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
    backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
    ..Default::default()
});
```

### Shader

The cube uses WGSL (WebGPU Shading Language) which is compiled at runtime by wgpu:

- **Vertex shader**: Transforms vertices using MVP matrix
- **Fragment shader**: Outputs interpolated vertex colors

## License

MIT
