#!/usr/bin/env bash
set -euo pipefail
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
cargo +1.97.1 build --release --locked --all-features --bin ruminate
python3 .forgejo/package.py binary linux amd64 target/release/ruminate
rustup target add --toolchain 1.97.1 x86_64-pc-windows-gnu
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar
cargo +1.97.1 build --release --locked --all-features --target x86_64-pc-windows-gnu --bin ruminate
python3 .forgejo/package.py binary windows x86_64 target/x86_64-pc-windows-gnu/release/ruminate.exe
