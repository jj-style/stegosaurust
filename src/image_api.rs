use reqwest;
use reqwest::blocking::Client;
use std::error::Error;

pub trait ImageApi {
    fn get_square_image(&self, width: usize) -> Result<Vec<u8>, Box<dyn Error>>;
}

pub struct PicsumClient {
    http_client: Client,
    base_url: String
}

impl PicsumClient {
    pub fn new() -> Self {
        PicsumClient {
            http_client: Client::new(),
            base_url: String::from("https://picsum.photos")
        }
    }
}

impl ImageApi for PicsumClient {
    fn get_square_image(&self, width: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        println!("{width}");
        let request_url = format!("{}/{}",self.base_url,width);
        let response = self.http_client.get(request_url).send()?.bytes()?;
        let bytes = response.to_vec();
        Ok(bytes)
    }
}

pub fn get_square_image_width_from_bytes(length: usize) -> usize {
    // w * 2 * 3 / 8 = max_len
    // (len * 8) / 3 / 2  = w
    // TODO - fix/implement this
    // ((length * 8) as f64 / 6_f64).ceil() as usize
    4096
    
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
    #[ignore]
    fn test_get_square_image_width_from_bytes() {
        assert_eq!(get_square_image_width_from_bytes(100), 800)
    }
}