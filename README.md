sudo apt-get install libudev-dev pkg-config
rustup target add thumbv7m-none-eabi
cargo install cargo-binutils

rustup component add llvm-tools-preview
apt-get install openocd

cargo watch -cx 'flash --chip stm32f103C8 --release'