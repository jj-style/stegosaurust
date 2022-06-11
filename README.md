<h1 align="center">
  <img src=".github/logo.png" alt="Stegosaurust" width="256" />
  <br />
  Stegosaurust
</h1>

<p align="center"><b>A simple image steganography tool, written in rust.</b></p>

[![Publish](https://github.com/jj-style/stegosaurust/actions/workflows/publish.yml/badge.svg?branch=v0.3.0)](https://github.com/jj-style/stegosaurust/actions/workflows/publish.yml)
[![CI](https://github.com/jj-style/stegosaurust/actions/workflows/ci.yml/badge.svg)](https://github.com/jj-style/stegosaurust/actions/workflows/ci.yml)

# Disclaimer
:warning: **This is a program I made for fun. There is no guarantee of cryptographic security or data confidentiality. Please do not use this for sensitive information. If you do, you are doing so at your own risk.** :warning:

# Introduction
Easily encode messages in images:
```bash
echo "text to hide" | stegosaurust --output encoded_image.png image.png
stegosaurust --decode encoded_image.png 
```
See the [examples](#examples) below for more usage. 

# Usage
```
🦕 stegosaurust 0.3.0
Hide text in images, using rust.

USAGE:
    stegosaurust [FLAGS] [OPTIONS] <image>

FLAGS:
    -b, --base64              Encode/decode with base64
    -C, --check-max-length    Check max message size that can be encoded with options given. Does not perform the
                              encoding, acts like a dry-run
    -c, --compress            Compress/decompress data
    -d, --decode              Decode a message from the image
    -h, --help                Prints help information
    -V, --version             Prints version information

OPTIONS:
    -i, --input <input>        Input file to encode, stdin if not present
    -k, --key <key>            Encrypt the text before encoding it with AES-256-CBC
    -N, --max-bit <max-bit>    Maximum bit to possible modify (1-4)
    -m, --method <method>      Method to use for encoding (lsb,rsb) [default: lsb]
    -o, --output <output>      Output file, stdout if not present
    -s, --seed <seed>          Seed for random significant bit encoding

ARGS:
    <image>    Input image
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
stegosaurust --decode examples/example-2.png | mpv -

# is there something hidden in the logo on the README?
stegosaurust --decode .github/logo.png | xargs python -c "import webbrowser,sys; webbrowser.open(sys.argv[1])"
```