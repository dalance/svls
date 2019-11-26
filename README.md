# svls

SystemVerilog language server

[![Actions Status](https://github.com/dalance/svls/workflows/Regression/badge.svg)](https://github.com/dalance/svls/actions)
[![Snap Status](https://build.snapcraft.io/badge/dalance/svls.svg)](https://build.snapcraft.io/user/dalance/svls)

[![Crates.io](https://img.shields.io/crates/v/svls.svg)](https://crates.io/crates/svls)
[![svlint](https://snapcraft.io/svls/badge.svg)](https://snapcraft.io/svls)

![test](https://user-images.githubusercontent.com/4331004/68925756-23478f00-07c7-11ea-84f3-2afd23ed2764.gif)

## Feature

* Linter based on [svlint](https://github.com/dalance/svlint).

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

### Language server

svls uses `.svls.toml` at the root of repository.
The example of `.svls.toml` is below:

```toml
[verilog]
include_paths = ["src/header"]

[option]
linter = true
```

#### `[verilog]` section

`include_paths` is include paths from the root of repository.

#### `[option]` section

`linter` shows whether linter feature is enabled.

### Linter

Linter uses `.svlint.toml` at the root of repository.
If `.svlint.toml` can't be used, all lint rules are enabled.
Please see [svlint#configuration](https://github.com/dalance/svlint#configuration) for the detailed information.

## Usage

### Visual Studio Code

Please install [svls-vscode](https://marketplace.visualstudio.com/items?itemName=dalance.svls-vscode) extension from marketplace.

### Vim/Neovim with [LanguageClient-neovim](https://github.com/autozimu/LanguageClient-neovim)

```viml
let g:LanguageClient_serverCommands = {
    \ 'systemverilog': ['svls'],
    \ }
```

### Vim/Neovim with [vim-lsp](https://github.com/prabirshrestha/vim-lsp)

```viml
if executable('svls')
    au User lsp_setup call lsp#register_server({
        \ 'name': 'svls',
        \ 'cmd': {server_info->['svls']},
        \ 'whitelist': ['systemverilog'],
        \ })
endif
```
