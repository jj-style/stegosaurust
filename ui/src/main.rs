use std::path::PathBuf;

use stegosaurust::cli::{Encode, EncodeOpts, StegMethod, BitDistribution};
use iced::{Sandbox, Settings};
use iced::widget::{self, column, container, image, row, text, button, pick_list};
use iced::{
    Alignment, Color, Command, Element, Length, Theme,
};
use native_dialog::{FileDialog};

struct Data(Encode);

fn main() -> iced::Result {
    let mut settings = Settings::default();
    let mut window = iced::window::Settings::default();
    window.size = (300,300);
    settings.window = window;
    Data::run(settings)
}


#[derive(Debug, Clone)]
enum Message {
    ChooseMask,
    EncMethodSelected(StegMethod),
    BitDistSelected(BitDistribution)
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
            Message::EncMethodSelected(method) => {
                self.0.opts.method = Some(method)
            },
            Message::BitDistSelected(dist) => {
                self.0.opts.distribution = Some(dist)
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

        let pick_bit_dist = pick_list(
            &BitDistribution::ALL[..],
            self.0.opts.distribution,
            Message::BitDistSelected,
        )
        .placeholder("Select bit distribution");

        let content = column![
            row![
                text(if self.validate_selected_img_mask() {format!("{:?}", self.0.image)} else {"Image mask".to_string()}),
                button("pick file").on_press(Message::ChooseMask),
            ],
            row![
                text("Encoding method"),
                pick_enc_method
            ],
            row![
                text("Bit distribution"),
                pick_bit_dist
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
}

