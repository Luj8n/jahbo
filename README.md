# Jahbo

An application that automatically displays the stats of players in a Hypixel bedwards lobby:

- Guild
- Final kills/deaths ratio
- Win/loss ratio
- Winstreak
- Tries to detect alts and snipers
- And many more

## Running

- First create `settings.toml`

```toml
log_file = '[path to the minecraft log file]'
api_key = '[hypixel api key]'
```

- Run it by compiling it (first follow the steps in 'Compiling')

```
cargo run --release
```

or

- Run it with an executable (download from github releases)

## Compiling

### 1. Install dependencies

Note: not sure if all dependencies are necessary

- On Linux:

```
sudo apt-get install -y libclang-dev libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev cmake libfontconfig-dev
```

- On Fedora:

```
sudo dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel libxcb-devel cmake fontconfig-devel
```

### 2. Build it

- `cargo build --release`
