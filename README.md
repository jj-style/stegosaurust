<h1 align="center">
  <img src=".github/logo.png" alt="Stegosaurust" width="256" />
  <br />
  Stegosaurust
</h1>

<p align="center"><b>A simple image steganography tool, written in rust.</b></p>

[![Publish](https://github.com/jj-style/stegosaurust/actions/workflows/publish.yml/badge.svg?branch=v0.4.4)](https://github.com/jj-style/stegosaurust/actions/workflows/publish.yml)
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
🦕 stegosaurust 0.4.4
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

## Features
Encoding supports a variety of options that effect how the data is encoded, including:
- compression
- base64 encoding
- encryption using AES-256-CBC, requires `key` to be supplied
- bit distribution - how to distribute encoded bits throughout the image used for encoding
  - `sequential` - encode the data pixel by pixel starting from the top left
  - `linear` - encode the data into pixels evenly spread out from the start to the end of all pixels
- bit encoding methods:
  - least significant bit (`lsb`) - always encode the bit of data in the least significant bit of each colour value of each pixel 
  - random significant bit (`rsb`) - randomly encode each bit of data into one the least `n` significant bits of each colour value of each pixel. Choose how large `n` can be (1-4) (least significant to fourth least significant) and supply a `seed` which is used to determine the bit to encode into

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

## Docker
```bash
docker pull ghcr.io/stegosaurust
docker run --rm -it -v $PWD:/data stegosaurust stegosaurust enc --decode /data/image.png
```

# Examples
The examples below assume you have installed the program ([see here](#installation)) and are in the repository directory (if not installed use `cargo run --` instead of `stegosaurust`).

```bash
# how much data can we fit in an image...
stegosaurust enc --decode examples/example-2.png | mpv -

# is there something hidden in the logo on the README?
stegosaurust enc --decode .github/logo.png | xargs python -c "import webbrowser,sys; webbrowser.open(sys.argv[1])"
```
