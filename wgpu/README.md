# wgpu 3D Demos - Rust + wgpu

3D graphics demos using **wgpu** (WebGPU/Vulkan) in Rust with **WGSL** shaders.

## Demos

### 🎲 Cube
Simple rotating cube with colored faces.

```bash
make cube
```

![Cube](https://github.com/user-attachments/assets/16423d00-c0c0-43f5-b9c9-908c44ec5941)

### 💎 Dodecahedron
Transparent emerald gemstone with directional lighting, subsurface scattering, and fresnel rim.

```bash
make dodecahedron
```

![Dodecahedron](https://github.com/user-attachments/assets/ba4fc7f4-3170-4ce5-90bd-e970c09b42d0)

### 💍 Ring
Golden torus with PBR metallic material and Blinn-Phong shading.

```bash
make ring
```

![Ring](https://github.com/user-attachments/assets/b777ec9c-46b1-4d61-ad56-b2287c96f0b1)

## Project Structure

```
├── src/
│   ├── main.rs              # Cube demo
│   ├── shader.wgsl          # Cube shader
│   └── bin/
│       ├── dodecahedron.rs  # Emerald dodecahedron
│       └── ring.rs          # Golden ring
├── Cargo.toml
├── Makefile
└── README.md
```

## Features

- **Directional lighting** (no ambient — proper shadows)
- **Blinn-Phong specular** highlights
- **Fresnel rim** effects
- **Transparency** with alpha blending (dodecahedron)
- **PBR-inspired** metallic materials (ring)
- **Depth buffering** for correct face ordering
- **Perspective projection**

## Requirements

### Ubuntu 24.04 + NVIDIA GPU

```bash
# Install system dependencies
make deps

# Or manually:
sudo apt-get update
sudo apt-get install -y \
    pkg-config \
    libx11-dev \
    libxkbcommon-dev \
    libwayland-dev \
    libvulkan-dev \
    vulkan-tools \
    mesa-vulkan-drivers

# Check Vulkan support
make vulkan-info
```

### Rust Toolchain

Works with **stable Rust** (no nightly required):

```bash
rustup update stable
rustup default stable
```

## Building & Running

```bash
# Run demos
make cube          # Colored cube
make dodecahedron  # Emerald gem
make ring          # Golden ring

# Release builds (optimized)
make run-cube-release
make run-dodecahedron-release
make run-ring-release

# Development
make build         # Build all
make fmt           # Format code
make lint          # Clippy
make clean         # Clean artifacts
```

## Technical Details

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `wgpu` | 28.0 | Cross-platform GPU API (Vulkan/Metal/DX12) |
| `winit` | 0.30 | Cross-platform window creation |
| `glam` | 0.30 | Fast math library (matrices, vectors) |
| `bytemuck` | 1.21 | Safe transmutes for GPU data |
| `pollster` | 0.4 | Async runtime for wgpu initialization |
| `env_logger` | 0.11 | Logging |

### Shaders (WGSL)

All shaders use **WGSL** (WebGPU Shading Language):

- **Cube**: Simple vertex colors, MVP transform
- **Dodecahedron**: Lambert diffuse, Blinn-Phong specular, subsurface scattering, fresnel
- **Ring**: PBR-inspired metallic, two-light setup, rim highlights

### GPU Backend

Prioritizes **Vulkan** (best for NVIDIA):

```rust
let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
    backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
    ..Default::default()
});
```

## Controls

- **Close window**: Click X or Alt+F4

## License

MIT