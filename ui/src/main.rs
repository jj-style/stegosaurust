// TODO - remove at some point
#![allow(unused_imports)]

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use iced::widget::{
    self, button, checkbox, column, container, image, pick_list, radio, row, svg, text, text_input,
    Image,
};
use iced::{Alignment, Color, Command, Element, Length, Theme};
use iced::{Sandbox, Settings};
use native_dialog::FileDialog;
use stegosaurust::cli::{BitDistribution, Encode, EncodeOpts, StegMethod};
use tempfile::NamedTempFile;

// TODO - make better type for this rather than just a tuple
#[derive(Debug, Clone)]
struct Data(Encode, String);

fn main() -> iced::Result {
    let mut settings = Settings::default();
    let mut window = iced::window::Settings::default();
    window.size = (300, 500);
    settings.window = window;
    Data::run(settings)
}

#[derive(Debug, Clone)]
enum Message {
    ChooseMask,
    EncMethodSelected(StegMethod),
    BitDistSelected(BitDistribution),
    LinearDistStepChanged(String),
    ToggleBase64(bool),
    ToggleCompression(bool),
    ToggleEncryption(bool),
    FunctionChanged(Function),
    RsbSeedChanged(String),
    RsbMaxBitChanged(u8),
    EncryptionKeyChanged(String),
    InputOutput(String),
    Submit,
    SaveImg,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Function {
    Encode,
    Decode,
}

impl From<bool> for Function {
    fn from(b: bool) -> Self {
        if b {
            Function::Decode
        } else {
            Function::Encode
        }
    }
}

impl Into<bool> for Function {
    fn into(self) -> bool {
        match self {
            Function::Encode => false,
            Function::Decode => true,
        }
    }
}

impl Sandbox for Data {
    type Message = Message;

    fn new() -> Self {
        Data(
            Encode {
                opts: EncodeOpts {
                    decode: false,
                    base64: false,
                    compress: false,
                    key: None,
                    method: Some(StegMethod::default()),
                    distribution: Some(BitDistribution::default()),
                    seed: None,
                    max_bit: Some(2),
                },
                check_max_length: false,
                output: None,
                input: None,
                image: PathBuf::new(),
            },
            "".to_string(),
        )
    }

    fn title(&self) -> String {
        String::from("Stegosaurust UI")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::ChooseMask => {
                let path = FileDialog::new()
                    .set_location("~/Desktop")
                    .add_filter("PNG Image", &["png"])
                    .add_filter("JPEG Image", &["jpg", "jpeg"])
                    .show_open_single_file()
                    .unwrap();

                let path = match path {
                    Some(path) => path,
                    None => self.0.image.clone(),
                };

                self.0.image = path
            }
            Message::EncMethodSelected(method) => self.0.opts.method = Some(method),
            Message::BitDistSelected(dist) => self.0.opts.distribution = Some(dist),
            Message::LinearDistStepChanged(step) => {
                if let Ok(parsed) = step.parse::<usize>() {
                    self.0.opts.distribution = Some(BitDistribution::Linear { length: parsed })
                }
            }
            Message::ToggleBase64(status) => self.0.opts.base64 = status,
            Message::ToggleCompression(status) => self.0.opts.compress = status,
            Message::ToggleEncryption(status) => {
                self.0.opts.key = if status { Some("".to_string()) } else { None }
            }
            Message::FunctionChanged(function) => self.0.opts.decode = function.into(),
            Message::RsbSeedChanged(seed) => self.0.opts.seed = Some(seed),
            Message::RsbMaxBitChanged(max_bit) => self.0.opts.max_bit = Some(max_bit),
            Message::EncryptionKeyChanged(key) => self.0.opts.key = Some(key),
            Message::Submit => {
                // TODO - popup or notify UI of any warnings
                self.validate_state().unwrap();
                let mut inputfile = NamedTempFile::new().unwrap();
                write!(inputfile, "{}", self.1).unwrap();
                self.0.input = Some(inputfile.path().to_path_buf());
                println!("input: {:?}", self.0.input.as_ref().unwrap());

                let outputfile = NamedTempFile::new().unwrap();
                self.0.output = Some(
                    outputfile
                        .into_temp_path()
                        .to_path_buf()
                        .with_extension("png"),
                );
                println!("output: {:?}", self.0.output.as_ref().unwrap());
                let cli_opts = stegosaurust::cli::Opt {
                    cmd: stegosaurust::cli::Command::Encode(self.0.clone()),
                };
                stegosaurust_cli::run(cli_opts).unwrap();
                println!("finished");

                // TODO - when decoding set input string to output:
                // self.0.output = a temp text file 
                //- self.1 = std::fs::read_to_string(self.0.output)
            }
            Message::SaveImg => {
                let selected_path = FileDialog::new()
                    .set_location("~/Downloads")
                    .add_filter("PNG Image", &["png"])
                    .show_save_single_file()
                    .unwrap();
                if let Some(path) = selected_path {
                    std::fs::copy(self.0.output.as_ref().unwrap(), path).unwrap();
                }
            }
            Message::InputOutput(s) => {
                self.1 = s;
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let pick_enc_method = pick_list(
            &StegMethod::ALL[..],
            self.0.opts.method,
            Message::EncMethodSelected,
        )
        .placeholder("Select encoding method");

        let enc_method_container = match self.0.opts.method.unwrap() {
            StegMethod::LeastSignificantBit => container(pick_enc_method),
            StegMethod::RandomSignificantBit => {
                let current_seed = match self.0.opts.seed {
                    Some(ref s) => s,
                    None => "",
                };
                container(column![
                    row![pick_enc_method],
                    row![
                        column![text("Seed")],
                        column![text_input(
                            "enter seed to random bit distribution",
                            current_seed,
                            Message::RsbSeedChanged
                        )]
                    ],
                    row![
                        column![text("Max bit")],
                        column![pick_list(
                            vec![1, 2, 3, 4],
                            self.0.opts.max_bit,
                            Message::RsbMaxBitChanged
                        )]
                    ]
                ])
            }
        };

        let pick_bit_dist = pick_list(
            &BitDistribution::ALL[..],
            self.0.opts.distribution,
            Message::BitDistSelected,
        )
        .placeholder("Select bit distribution");

        let choose_function =
            [Function::Encode, Function::Decode]
                .iter()
                .fold(column![], |column, f| {
                    column.push(radio(
                        format!("{f:?}"),
                        *f,
                        Some(Function::from(self.0.opts.decode)),
                        Message::FunctionChanged,
                    ))
                });

        let bit_dist_container = match (self.0.opts.distribution, self.0.opts.decode) {
            (Some(BitDistribution::Linear { length }), true) => {
                let picker_row = row![text("Bit distribution"), pick_bit_dist];
                let bit_dist_row = row![
                    text("Distribution key"),
                    text_input("", &length.to_string(), Message::LinearDistStepChanged)
                ]
                .spacing(10);
                container(column![picker_row, bit_dist_row])
            }
            _ => container(row![text("Bit distribution"), pick_bit_dist]),
        };

        // TODO - tidy these container creations up a bit

        let encrypt_cb = checkbox(
            "encrypt",
            self.0.opts.key.is_some(),
            Message::ToggleEncryption,
        );

        let encrypt_container = match &self.0.opts.key {
            Some(k) => {
                row![
                    encrypt_cb,
                    text_input("", k, Message::EncryptionKeyChanged).password()
                ]
            }
            None => row![encrypt_cb],
        };

        let img_container = self.0.output.clone().map_or_else(
            || column![],
            |p| {
                column![
                    row![Image::new(p)],
                    row![button("save").on_press(Message::SaveImg)],
                ]
            },
        );

        let content = row![
            column![
                row![text_input("message", &self.1, Message::InputOutput)],
                row![
                    text(if self.validate_selected_img_mask() {
                        format!("{:?}", self.0.image)
                    } else {
                        "Image mask".to_string()
                    }),
                    button("pick file").on_press(Message::ChooseMask),
                ],
                row![text("Encoding method"), enc_method_container],
                bit_dist_container,
                row![
                    checkbox("base64", self.0.opts.base64, Message::ToggleBase64),
                    checkbox("compress", self.0.opts.compress, Message::ToggleCompression),
                    encrypt_container,
                ],
                row![choose_function],
                // TODO - disable submit when not valid
                row![button("submit").on_press(Message::Submit)],
                row![img_container]
            ]
        ];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

impl Data {
    fn validate_selected_img_mask(&self) -> bool {
        let p = &self.0.image;
        p.exists()
    }

    fn validate_state(&self) -> Result<(), &'static str> {
        if let Some(BitDistribution::Linear { length }) = self.0.opts.distribution {
            if self.0.opts.decode {
                if length < 1 {
                    return Err(
                        "Linear bit distribution length must be greater than or equal to 1.",
                    );
                }
            }
        }
        if let Some(StegMethod::RandomSignificantBit) = self.0.opts.method {
            if self.0.opts.seed.as_ref().map_or(true, |s| s.is_empty()) {
                return Err(
                    "Seed cannot be empty when random significant bit encoding is selected",
                );
            }
        }
        if let Some(ref key) = self.0.opts.key {
            if key.is_empty() {
                return Err("Encyrption key cannot be empty if encryption is enabled");
            }
        }

        Ok(())
    }
}
