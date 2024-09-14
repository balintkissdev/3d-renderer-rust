# Real-time 3D renderer with OpenGL 4 written in Rust

[![Build](https://github.com/balintkissdev/3d-renderer-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/balintkissdev/3d-renderer-rust/actions/workflows/ci.yml)

> A hardware-accelerated 3D renderer written in Rust. Runs using OpenGL 4.3 as
graphics API on desktop.

[Click here for C++ version of this project](https://github.com/balintkissdev/3d-renderer-cpp)

![Demo](doc/img/demo.png)

## Table of Contents

- [Try it out!](#try-it-out)
- [Motivation](#motivation)
- [Features](#features)
- [Requirements](#requirements)
- [Build Instructions](#build-instructions)
- [Usage](#usage)
- [Resources](#resources)

## Try it out!

- [Windows 64-bit download](https://github.com/balintkissdev/3d-renderer-rust/releases/download/0.1.0/3d-renderer-rust-0.1.0-win64.zip)
- [Linux 64-bit download](https://github.com/balintkissdev/3d-renderer-cpp/releases/download/0.1.0/3d-renderer-rust-0.1.0-linux-x86_64.tar.gz)

## Motivation

This project is a demonstration of my ability to write cross-platform 3D
graphical applications in Rust. I designed my application to balance the correct
level of abstractions and performance optimizations. The project showcases
confident usage of the following technologies:

- Rust
- OpenGL 4.x
- Immediate mode overlay GUI using Dear ImGui (as opposed to retained mode GUI frameworks like Qt)

Future additions will include Direct3D, Vulkan rendering backends and additional post-processing effects.

## Features

- 3D model display from `OBJ` file format
- Fly-by FPS camera movement
- Skybox display using cube-map
- Directional light with ADS (Ambient, Diffuse, Specular) lighting (Phong shading)

## Requirements

Desktop executable requires an OpenGL 4.3 compatible graphics adapter to run.
Check if your hardware supports OpenGL 4.3 and have the latest graphics driver
installed.

Required build dependencies on Debian, Ubuntu, Linux Mint:

```sh
sudo apt-get install extra-cmake-modules libglfw3-dev wayland-protocols libxkbcommon-dev xorg-dev
```

Required build dependencies on Fedora, Red Hat:

```sh
sudo dnf install extra-cmake-modules glfw-devel wayland-protocols wayland-devel libxkbcommon-devel libXcursor-devel libXi-devel libXinerama-devel libXrandr-devel
```

No additinal dependencies are required to be installed on Windows builds.

All other dependencies are either included in `vendor` folder or automatically downloaded by `cargo`.

## Build

1. Make sure you have the latest stable version of Rust and `cargo` installed, following the instructions on
https://www.rust-lang.org/tools/install

2. Clone the repository

  ```sh
  git clone https://github.com/balintkissdev/3d-renderer-rust.git
  cd 3d-renderer-rust
  ```

3. Compile and execute the release build

  ```sh
  cargo run --release
  ```

## Usage

Use keyboard and mouse to navigate the 3D environment.

- Movement: `W`, `A`, `S`, `D`
- Mouse look: `Right-click` and drag
- Ascend: `Spacebar`
- Descend: `C`

Modify UI controls to change properties of the 3D model display.

## Resources

- *Utah Teapot* and *Stanford Bunny* model meshes are from [Stanford Computer Graphics Laboratory](https://graphics.stanford.edu/)
    - High poly *Stanford Bunny* model mesh is from https://www.prinmath.com/csci5229/OBJ/index.html
- Skybox texture images are from [learnopengl.com](https://learnopengl.com/Advanced-OpenGL/Cubemaps)

