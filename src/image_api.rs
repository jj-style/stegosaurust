use reqwest;
use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use std::error::Error;

pub trait ImageApi {
    fn get_square_image(&self, width: usize) -> Result<Vec<u8>, Box<dyn Error>>;
}

pub struct PicsumClient {
    http_client: Client,
    base_url: String,
}

impl PicsumClient {
    pub fn new() -> Self {
        PicsumClient {
            http_client: Client::new(),
            base_url: String::from("https://picsum.photos"),
        }
    }
}

impl std::default::Default for PicsumClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageApi for PicsumClient {
    fn get_square_image(&self, width: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let request_url = format!("{}/{}", self.base_url, width);
        let response = self
            .http_client
            .get(request_url)
            .header(
                USER_AGENT,
                "Mozilla/5.0 (Windows NT 10.0; rv:91.0) Gecko/20100101 Firefox/91.0",
            )
            .send()?;
        if response.status().is_success() {
            let bytes = response.bytes()?.to_vec();
            Ok(bytes)
        } else {
            Err(Box::from(format!(
                "{}:{}",
                response.status().as_str(),
                response.text()?
            )))
        }
    }
}

pub fn get_square_image_width_from_bytes(length: usize) -> usize {
    let min = 200;
    let max = 5000;
    let width = ((length * 8) as f64 / 3_f64).sqrt() as usize * 2;
    match width.cmp(&min) {
        std::cmp::Ordering::Less | std::cmp::Ordering::Equal => min,
        std::cmp::Ordering::Greater => {
            if width <= max {
                width
            } else {
                max
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_picsum_api_client_creation() {
        let _ = PicsumClient::new();
    }

    #[test]
    fn test_picsum_api_get_square_image() {
        let client = PicsumClient::new();
        let result = client.get_square_image(100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_square_image_width_from_bytes() {
        assert_eq!(get_square_image_width_from_bytes(100), 200)
    }
}
