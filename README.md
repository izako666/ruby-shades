Ruby Shades

Ruby Shades is a full-stack media streaming platform built using Rust. It uses Yew for the frontend, Axum for the backend, and FFmpeg for transcoding videos to HLS format.
Key Features

    Frontend: Built with Yew, a Rust framework for creating web applications with WebAssembly.

    Backend: Powered by Axum, a web framework built on top of Tokio and Hyper.

    Video Transcoding: Uses FFmpeg to transcode videos into HLS (HTTP Live Streaming) format.

    Note: FFmpeg must be compiled with libx264 for the project to work.

Prerequisites

    Rust (version 1.88.0 or later) — https://www.rust-lang.org/tools/install

    FFmpeg compiled with libx264 — https://trac.ffmpeg.org/wiki/CompilationGuide

Setup and Installation
1. Clone the repository:

git clone https://github.com/yourusername/ruby-shades.git
cd ruby-shades

2. Set up the frontend:

The frontend is built with Yew and uses WebAssembly. To build it:

cd frontend
cargo build --target wasm32-unknown-unknown

3. Set up the backend:

The backend uses Axum. To build it:

cd backend
cargo build

4. Compile FFmpeg with libx264:

Make sure FFmpeg is compiled with libx264 support for video transcoding:

sudo apt update
sudo apt install ffmpeg libx264-dev

Alternatively, follow the official guide to compile FFmpeg with libx264.
5. Run the application:

To run the backend:
cd ruby-shades-backend
cargo run

To build the frontend:

cd ruby-shades-frontend
trunk serve

License

This project is under the MIT license. 
https://opensource.org/license/mit

