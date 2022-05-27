<h1 align="center">
  <img src=".github/logo.png" alt="Stegosaurust" width="256" />
  <br />
  Stegosaurust
</h1>

<p align="center"><b>A simple image steganography tool, written in rust.</b></p>

# Disclaimer
:warning: **This is a program I made for fun. There is no guarantee of cryptographic security or data confidentiality. Please do not use this for sensitive information. If you do, you are doing so at your own risk.** :warning:

# Introduction
Easily encode messages in images:
```bash
echo "text to hide" | stegosaurust --output encoded_image.png image.png
stegosaurust --decode encoded_image.png 
```

# Usage
```
🦕 Stegosaurust 0.1.0
Hide text in images, using rust.

USAGE:
    stegosaurust [FLAGS] [OPTIONS] <image>

FLAGS:
    -b, --base64     Encode/decode with base64
    -d, --decode     Decode a message from the image
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input <input>      Input file to encode, stdin if not present
    -k, --key <key>          Encrypt the text before encoding it with AES-128-CBC
    -o, --output <output>    Output file, stdout if not present

ARGS:
    <image>    Input image
```

# Installation
**TODO**