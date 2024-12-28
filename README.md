# TIFF Volume Viewer

A web-based TIFF image viewer built with Rust, WebAssembly, and Next.js that supports both 8-bit and 16-bit TIFF files.

## Features

- Load and display TIFF files directly in the browser
- Support for both 8-bit and 16-bit TIFF images
- Camera controls:
  - Left mouse button drag: Orbit/rotate the view
  - Right mouse button drag: Pan the view
  - Mouse wheel: Zoom in/out
- Automatic image orientation correction
- Memory-efficient image processing

## Project Structure

- `src/rust/` - Rust source code for image processing and rendering
  - `lib.rs` - Main WASM module and volume data handling
  - `camera.rs` - Camera controls implementation
  - `renderer.rs` - Volume rendering engine
  - `tiff_loader.rs` - TIFF file loading and processing
  - `transfer_function.rs` - Color and intensity mapping

- `src/components/` - React components
  - `viewer/` - TIFF viewer interface

## Technology Stack

- Rust - Core image processing and rendering
- WebAssembly - Browser integration
- Next.js - Web framework
- React - UI components
- Tailwind CSS - Styling

## Development

1. Install dependencies:
```bash
npm install
```

2. Build the WebAssembly module:
```bash
cd src/rust
wasm-pack build --target web --out-dir ../../public/pkg
```

3. Run the development server:
```bash
npm run dev
```

## Usage

1. Open the application in a web browser
2. Click "Load Image" to select a TIFF file
3. Use mouse controls to interact with the image:
   - Drag with left mouse button to rotate
   - Drag with right mouse button to pan
   - Use mouse wheel to zoom

## Requirements

- Node.js 16 or later
- Rust toolchain with wasm-pack
- Web browser with WebAssembly support
