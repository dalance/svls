# svls

SystemVerilog language server

[![Actions Status](https://github.com/dalance/svls/workflows/Regression/badge.svg)](https://github.com/dalance/svls/actions)

[![Crates.io](https://img.shields.io/crates/v/svls.svg)](https://crates.io/crates/svls)
[![svlint](https://snapcraft.io/svls/badge.svg)](https://snapcraft.io/svls)
[![AUR version](https://img.shields.io/aur/version/svls?logo=Arch-Linux)](https://aur.archlinux.org/packages/svls/)

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
defines = ["DEBUG", "VAR=1"]
plugins = ["path/to/libfoo.so", "path/to/libbar.so"]

[option]
linter = true
```

#### `[verilog]` section

`include_paths` is include paths from the root of repository.
`defines` is define strings.
`plugins` is paths to svlint plugins from the working directory.

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

### Vim/Neovim with [coc.nvim](https://github.com/neoclide/coc.nvim)

In configuration file
```json
"languageserver": {
    "svls": {
        "command": "svls",
        "filetypes": ["systemverilog"]
    }
}
```

### Emacs with [lsp-mode](https://github.com/emacs-lsp/lsp-mode)

```emacs-lisp
(use-package flycheck
  :ensure t
  :defer t
  :init (global-flycheck-mode t))

(use-package company
  :ensure t
  :defer t
  :init (global-company-mode t)
  :config
  ;; Company Flx adds fuzzy matching to company, powered by the sophisticated
  ;; sorting heuristics  in =flx=
  (use-package company-flx
    :ensure t
    :after company
    :init (company-flx-mode t))
  ;; Company Quickhelp
  ;; When idling on a completion candidate the documentation for the
  ;; candidate will pop up after `company-quickhelp-delay' seconds.
  (use-package company-quickhelp
    :after company
    :ensure t
    ;; :init (company-quickhelp-mode t)
    :hook (prog-mode . (lambda ()
                         (when (window-system)
                           (company-quickhelp-local-mode))))
    :config
    (setq company-quickhelp-delay 0.2
          company-quickhelp-max-lines nil)))

(use-package lsp-mode
  :defer t
  :ensure t
  :commands lsp
  :config
  (setq lsp-log-io nil
        lsp-auto-configure t
        lsp-auto-guess-root t
        lsp-enable-completion-at-point t
        lsp-enable-xref t
        lsp-prefer-flymake nil
        lsp-use-native-json t
        lsp-enable-indentation t
        lsp-response-timeout 10
        lsp-restart 'auto-restart
        lsp-keep-workspace-alive t
        lsp-eldoc-render-all nil
        lsp-enable-snippet nil
        lsp-enable-folding t)
   ;;; lsp-ui gives us the blue documentation boxes and the sidebar info
  (use-package lsp-ui
    :defer t
    :ensure t
    :after lsp
    :commands lsp-ui-mode
    :config
    (setq lsp-ui-sideline-ignore-duplicate t
          lsp-ui-sideline-delay 0.5
          lsp-ui-sideline-show-symbol t
          lsp-ui-sideline-show-hover t
          lsp-ui-sideline-show-diagnostics t
          lsp-ui-sideline-show-code-actions t
          lsp-ui-peek-always-show t
          lsp-ui-doc-use-childframe t)
    :bind
    (:map lsp-ui-mode-map
          ([remap xref-find-definitions] . lsp-ui-peek-find-definitions)
          ([remap xref-find-references] . lsp-ui-peek-find-references))
    :hook
    ((lsp-mode . lsp-ui-mode)
     (lsp-after-open . (lambda ()
                         (lsp-ui-flycheck-enable t)
                         (lsp-ui-sideline-enable t)
                         (lsp-ui-imenu-enable t)
                         (lsp-lens-mode t)
                         (lsp-ui-peek-enable t)
                         (lsp-ui-doc-enable t)))))
  ;;; company lsp
  ;; install LSP company backend for LSP-driven completion
  (use-package company-lsp
    :defer t
    :ensure t
    :after company
    :commands company-lsp
    :config
    (setq company-lsp-cache-candidates t
          company-lsp-enable-recompletion t
          company-lsp-enable-snippet t
          company-lsp-async t)
    ;; avoid, as this changes it globally do it in the major mode instead (push
    ;; 'company-lsp company-backends) better set it locally
    :hook (lsp-after-open . (lambda()
                              (add-to-list (make-local-variable 'company-backends)
                                           'company-lsp)))))

(use-package verilog-mode
  :defer t
  :config
  (require 'lsp)
  (lsp-register-client
   (make-lsp-client :new-connection (lsp-stdio-connection '("svls"))
   :major-modes '(verilog-mode)
   :priority -1
   ))
  :hook (verilog-mode . (lambda()
      (lsp)
      (flycheck-mode t)
      (add-to-list 'lsp-language-id-configuration '(verilog-mode . "verilog")))))
```
