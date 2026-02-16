use std::collections::VecDeque;

/// Ring buffer for audio samples.
pub struct RingBuffer {
    buffer: VecDeque<f32>,
    capacity: usize,
    sample_rate: u32,
}

impl RingBuffer {
    pub fn new(duration_secs: f32, sample_rate: u32) -> Self {
        let capacity = (duration_secs * sample_rate as f32) as usize;
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            sample_rate,
        }
    }

    pub fn push_samples(&mut self, samples: &[f32]) {
        for &sample in samples {
            if self.buffer.len() >= self.capacity {
                self.buffer.pop_front();
            }
            self.buffer.push_back(sample);
        }
    }

    pub fn samples(&self) -> Vec<f32> {
        self.buffer.iter().copied().collect()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_empty() {
        let buf = RingBuffer::new(1.0, 16000);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.sample_rate(), 16000);
    }

    #[test]
    fn push_and_retrieve_samples() {
        let mut buf = RingBuffer::new(1.0, 16000);
        let data = vec![0.1, 0.2, 0.3];
        buf.push_samples(&data);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.samples(), data);
    }

    #[test]
    fn ring_buffer_overwrites_old_data() {
        // 0.001 seconds at 1000 Hz = capacity of 1 sample
        let mut buf = RingBuffer::new(0.001, 1000);
        buf.push_samples(&[1.0, 2.0, 3.0]);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.samples(), vec![3.0]);
    }

    #[test]
    fn clear_empties_buffer() {
        let mut buf = RingBuffer::new(1.0, 16000);
        buf.push_samples(&[1.0; 100]);
        assert!(!buf.is_empty());
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn capacity_matches_duration() {
        let buf = RingBuffer::new(2.0, 16000);
        let mut buf = buf;
        // Fill with exactly 2 seconds of data
        let data = vec![0.5; 32000];
        buf.push_samples(&data);
        assert_eq!(buf.len(), 32000);
        // Adding one more sample should evict the oldest
        buf.push_samples(&[0.9]);
        assert_eq!(buf.len(), 32000);
        let samples = buf.samples();
        assert!((samples[samples.len() - 1] - 0.9).abs() < f32::EPSILON);
    }
}
