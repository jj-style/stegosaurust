use crate::StegError;
use compression::prelude::*;

/// compress a slice of bytes into a new vec of bytes
pub fn compress(data: &[u8]) -> Result<Vec<u8>, StegError> {
    if data.is_empty() {
        return Err(StegError::Compression("Input data is empty.".to_string()));
    }
    
    data.iter()
        .cloned()
        .encode(&mut BZip2Encoder::new(9), Action::Finish)
        .collect::<Result<Vec<_>, _>>()
        .map_err(StegError::Compression)
}

/// decompress a slice of bytes into a new vec of bytes
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, StegError> {
    if data.is_empty() {
        return Err(StegError::Decompression("Input data is empty.".to_string()));
    }
    
    data.iter()
        .cloned()
        .decode(&mut BZip2Decoder::new())
        .collect::<Result<Vec<_>, _>>()
        .map_err(StegError::Decompression)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression() {
        let input = "a".repeat(100);
        let output = compress(input.as_bytes()).unwrap();
        assert!(output.len() < input.len());
    }

    #[test]
    fn test_decompression() {
        let input = "a".repeat(100);
        let output = compress(input.as_bytes()).unwrap();
        assert_eq!(decompress(&output).unwrap(), input.as_bytes());
    }
    
    #[test]
    fn test_empty_input_compression() {
        let input = "";
        let output = compress(input.as_bytes());
        assert!(output.is_err());
    }

    #[test]
    fn test_empty_input_decompression() {
        let input = "";
        let output = decompress(input.as_bytes());
        assert!(output.is_err());
    }
}

