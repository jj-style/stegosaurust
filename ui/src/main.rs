use std::path::PathBuf;

use iced::widget::{
    self, button, checkbox, column, container, image, pick_list, radio, row, slider, text,
};
use iced::{Alignment, Color, Command, Element, Length, Theme};
use iced::{Sandbox, Settings};
use native_dialog::FileDialog;
use stegosaurust::cli::{BitDistribution, Encode, EncodeOpts, StegMethod};

struct Data(Encode);

fn main() -> iced::Result {
    let mut settings = Settings::default();
    let mut window = iced::window::Settings::default();
    window.size = (500, 300);
    settings.window = window;
    Data::run(settings)
}

#[derive(Debug, Clone)]
enum Message {
    ChooseMask,
    EncMethodSelected(StegMethod),
    BitDistSelected(BitDistribution),
    LinearDistStepChanged(u8),
    ToggleBase64(bool),
    ToggleCompression(bool),
    FunctionChanged(Function),
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
        Data(Encode {
            opts: EncodeOpts {
                decode: false,
                base64: false,
                compress: false,
                key: None,
                method: Some(StegMethod::default()),
                distribution: Some(BitDistribution::default()),
                seed: None,
                max_bit: None,
            },
            check_max_length: false,
            output: None,
            input: None,
            image: PathBuf::new(),
        })
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
                self.0.opts.distribution = Some(BitDistribution::Linear {
                    length: step as usize,
                })
            }
            Message::ToggleBase64(status) => self.0.opts.base64 = status,
            Message::ToggleCompression(status) => self.0.opts.compress = status,
            Message::FunctionChanged(function) => self.0.opts.decode = function.into(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let pick_enc_method = pick_list(
            &StegMethod::ALL[..],
            self.0.opts.method,
            Message::EncMethodSelected,
        )
        .placeholder("Select encoding method");

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
                    text(format!("Distribution key: {}", length)),
                    slider(1..=100, length as u8, Message::LinearDistStepChanged)
                ]
                .spacing(10);
                container(column![picker_row, bit_dist_row])
            }
            _ => container(row![text("Bit distribution"), pick_bit_dist]),
        };

        let content = column![
            row![
                text(if self.validate_selected_img_mask() {
                    format!("{:?}", self.0.image)
                } else {
                    "Image mask".to_string()
                }),
                button("pick file").on_press(Message::ChooseMask),
            ],
            row![text("Encoding method"), pick_enc_method],
            bit_dist_container,
            row![
                checkbox("base64", self.0.opts.base64, Message::ToggleBase64),
                checkbox("compress", self.0.opts.compress, Message::ToggleCompression)
            ],
            row![choose_function]
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
}
