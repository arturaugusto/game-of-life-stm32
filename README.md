# Conway's Game of life uC

![short_demo](short_demo.gif "Demo")


# Useful commands to install things

```bash
sudo apt-get install libudev-dev pkg-config libc-dev
rustup target add thumbv7m-none-eabi
cargo install cargo-binutils
rustup component add llvm-tools-preview
apt-get install openocd
```

# Watch for changes and flash stm32

```bash
cargo watch -cx 'flash --chip stm32f103C8 --release'
```
