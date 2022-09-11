<h1 align="center">
  <img src=".github/logo.png" alt="Stegosaurust" width="256" />
  <br />
  Stegosaurust
</h1>

<p align="center"><b>A simple image steganography tool, written in rust.</b></p>

[![Publish](https://github.com/jj-style/stegosaurust/actions/workflows/publish.yml/badge.svg?branch=v0.4.2)](https://github.com/jj-style/stegosaurust/actions/workflows/publish.yml)
[![CI](https://github.com/jj-style/stegosaurust/actions/workflows/ci.yml/badge.svg)](https://github.com/jj-style/stegosaurust/actions/workflows/ci.yml)

# Disclaimer
:warning: **This is a program I made for fun. There is no guarantee of cryptographic security or data confidentiality. Please do not use this for sensitive information. If you do, you are doing so at your own risk.** :warning:

# Introduction
Easily encode messages in images:
```bash
echo "text to hide" | stegosaurust enc --output encoded_image.png image.png
stegosaurust enc --decode encoded_image.png 
```
See the [examples](#examples) below for more usage. 

# Usage
```
🦕 stegosaurust 0.4.2
Hide text in images, using rust.

USAGE:
    stegosaurust <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    disguise    mask all files in a directory using steganography [aliases: dsg]
    encode      encode files using steganography [aliases: enc]
    help        Prints this message or the help of the given subcommand(s)
```

# Installation
## From crates.io
```bash
cargo install stegosaurust
```

## From Source
Build and install the executable from the source code.
```bash
git clone https://github.com/jj-style/stegosaurust.git
cd stegosaurust
cargo install --path .

# to uninstall :(
cargo uninstall stegosaurust
```

# Examples
The examples below assume you have installed the program ([see here](#installation)) and are in the repository directory (if not installed use `cargo run --` instead of `stegosaurust`).

```bash
# how much data can we fit in an image...
stegosaurust enc --decode examples/example-2.png | mpv -

# is there something hidden in the logo on the README?
stegosaurust enc --decode .github/logo.png | xargs python -c "import webbrowser,sys; webbrowser.open(sys.argv[1])"
```