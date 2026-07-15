
cargo build -p pdf-core

cargo check -p pdf-core


run dev:

cargo tauri dev --manifest-path src-tauri/Cargo.toml

build cli linux: 
1. Nếu Linux là Intel/AMD (phổ biến nhất) (VACOM)
cargo build \
  -p report-cli \
  --release \
  --target x86_64-unknown-linux-gnu
  
check file: file report-cli/target/x86_64-unknown-linux-gnu/release/report-cli

cargo zigbuild \
    -p report-cli \
    --release \
    --target x86_64-unknown-linux-gnu

or
2. Nếu Linux là ARM64
cargo build \
  -p report-cli \
  --release \
  --target aarch64-unknown-linux-gnu
