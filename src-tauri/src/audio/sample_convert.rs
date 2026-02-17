/// Converts raw bytes (little-endian f32 format) to f32 audio samples.
pub fn bytes_to_f32_samples(data: &[u8]) -> Vec<f32> {
    if !data.len().is_multiple_of(4) {
        return Vec::new();
    }
    data.chunks_exact(4)
        .map(|chunk| {
            let bytes: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
            f32::from_le_bytes(bytes)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        assert!(bytes_to_f32_samples(&[]).is_empty());
    }

    #[test]
    fn test_invalid_length() {
        assert!(bytes_to_f32_samples(&[0, 1, 2]).is_empty());
        assert!(bytes_to_f32_samples(&[0, 1, 2, 3, 4]).is_empty());
    }

    #[test]
    fn test_known_value_roundtrip() {
        let values = vec![0.0f32, 1.0, -1.0, 0.5, -0.5];
        let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        let result = bytes_to_f32_samples(&bytes);
        assert_eq!(result, values);
    }

    #[test]
    fn test_single_sample() {
        let value = 0.42f32;
        let bytes = value.to_le_bytes();
        let result = bytes_to_f32_samples(&bytes);
        assert_eq!(result.len(), 1);
        assert!((result[0] - 0.42).abs() < f32::EPSILON);
    }
}
