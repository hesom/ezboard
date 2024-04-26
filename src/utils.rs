use std::collections::VecDeque;

#[derive(Debug)]
pub struct RingBuffer<T> {
    buf: VecDeque<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        RingBuffer {
            buf: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add(&mut self, value: T) {
        self.buf.push_back(value);
        if self.buf.len() > self.capacity {
            self.buf.pop_front();
        }
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.buf.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::vec_deque::IterMut<'_, T> {
        self.buf.iter_mut()
    }

    pub fn peek(&self) -> Option<&T> {
        self.buf.front()
    }
}

#[cfg(test)]
mod tests {
    use super::RingBuffer;

    #[test]
    fn wrap() {
        let mut buffer = RingBuffer::new(3);
        buffer.add(1);
        buffer.add(2);
        buffer.add(3);
        buffer.add(4);

        assert_eq!(buffer.peek(), Some(&2));
    }

    #[test]
    fn iterator() {
        let mut buffer = RingBuffer::new(3);
        buffer.add(1);
        buffer.add(2);
        buffer.add(3);
        buffer.add(4);

        let v: Vec<_> = buffer.iter().collect();
        assert_eq!(v, vec![&2, &3, &4]);
    }
}
