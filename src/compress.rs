use crate::StegError;
use compression::prelude::*;

/// compress a slice of bytes into a new vec of bytes
pub fn compress(data: &[u8]) -> Result<Vec<u8>, StegError> {
    if data.is_empty() {
        return Err(StegError::Custom("Input data is empty".to_string()));
    }

    data.iter()
        .cloned()
        .encode(&mut BZip2Encoder::new(9), Action::Finish)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StegError::Compression("Failed to compress data".to_string()))
}

/// decompress a slice of bytes into a new vec of bytes
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, StegError> {
    if data.is_empty() {
        return Err(StegError::Custom("Input data is empty".to_string()));
    }

    data.iter()
        .cloned()
        .decode(&mut BZip2Decoder::new())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StegError::Decompression("Failed to decompress data".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression() {
        let input = "a".repeat(100);
        let output = compress(input.as_bytes());

        match output {
            Ok(compressed_data) => assert!(compressed_data.len() < input.len()),
            Err(e) => panic!("Failed to compress data: {:?}", e),
        }
    }

    #[test]
    fn test_decompression() {
        let input = "a".repeat(100);
        let compressed_output = compress(input.as_bytes()).expect("Compression failed");
        let decompressed_output = decompress(&compressed_output);

        match decompressed_output {
            Ok(decompressed_data) => assert_eq!(decompressed_data, input.as_bytes()),
            Err(e) => panic!("Failed to decompress data: {:?}", e),
        }
    }
    
    #[test]
    fn test_empty_input_compression() {
        assert!(compress(&[]).is_err());
    }

    #[test]
    fn test_empty_input_decompression() {
        assert!(decompress(&[]).is_err());
    }
}
