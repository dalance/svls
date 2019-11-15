# svls

SystemVerilog language server

[![Actions Status](https://github.com/dalance/svls/workflows/Regression/badge.svg)](https://github.com/dalance/svls/actions)
[![Snap Status](https://build.snapcraft.io/badge/dalance/svls.svg)](https://build.snapcraft.io/user/dalance/svls)

[![Crates.io](https://img.shields.io/crates/v/svls.svg)](https://crates.io/crates/svls)
[![svlint](https://snapcraft.io/svls/badge.svg)](https://snapcraft.io/svls)

## Feature

* Diagnostic with [svlint](https://github.com/dalance/svlint).

## Installation

### Download binary

Download from [release page](https://github.com/dalance/svls/releases/latest), and extract to the directory in PATH.

### snapcraft

You can install from [snapcraft](https://snapcraft.io/svls)

```
sudo snap install svls
```

### Cargo

You can install by [cargo](https://crates.io/crates/svls).

```
cargo install svls
```

## Configuration

### Diagnostic

Diagnostic requires `.svlint.toml` at the root of repository.
Please see [svlint#configuration](https://github.com/dalance/svlint#configuration) for the detailed information.

## Usage

### Neovim with [LanguageClient-neovim](https://github.com/autozimu/LanguageClient-neovim)

```
let g:LanguageClient_serverCommands = {
    \ 'systemverilog': ['svls'],
    \ }

```
