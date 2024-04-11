use core::num::NonZeroUsize;

#[derive(Debug, PartialEq, Eq)]
pub struct Delay<'a, T> {
    head: usize,
    buffer: &'a mut [T],
}

impl<'a, T> Delay<'a, T> {
    /// Creates a new instance with the buffer.
    ///
    /// ## Panics
    /// - If the buffer is empty.
    ///
    /// ```rust
    /// # use plate::instruments::Delay;
    /// let mut buffer = [0.0; 8];
    /// let mut delay = Delay::new(&mut buffer);
    /// ```
    pub fn new(buffer: &mut [T]) -> Delay<'_, T> {
        if buffer.is_empty() {
            panic!("buffer is empty");
        }
        Delay { head: 0, buffer }
    }

    /// Write the value into a buffer.
    /// The buffer will be overridden.
    ///
    /// ```rust
    /// # use plate::instruments::Delay;
    /// let mut buffer = [0.0; 2];
    /// let mut delay = Delay::new(&mut buffer);
    ///
    /// delay.write(1.0);
    /// delay.write(2.0);
    /// delay.write(3.0); // overrides.
    ///
    /// assert_eq!(3.0, *delay.read(1.try_into().unwrap()));
    /// assert_eq!(2.0, *delay.read(2.try_into().unwrap()));
    /// ```
    pub fn write(&mut self, value: T) {
        self.buffer[self.head] = value;
        self.head = (self.head + 1) % self.buffer.len();
    }

    /// Read a value with delay.
    /// `delay = 1` means the time right after writing at.
    ///
    /// ```rust
    /// # use plate::instruments::Delay;
    /// let mut buffer = [0.0; 2];
    /// let mut delay = Delay::new(&mut buffer);
    ///
    /// delay.write(1.0);
    ///
    /// assert_eq!(1.0, *delay.read(1.try_into().unwrap()));
    /// // Warps when the delay underflows the buffer length.
    /// assert_eq!(0.0, *delay.read(2.try_into().unwrap()));
    /// ```
    pub fn read(&self, delay: NonZeroUsize) -> &T {
        &self.buffer[self.index(delay)]
    }

    fn index(&self, delay: NonZeroUsize) -> usize {
        let buffer_len = self.buffer.len();
        match (self.head, delay.get() % buffer_len) {
            // [ |  |  |  |  | ]
            //   <-----------^
            //      Offset   H
            (head, offset) if offset <= head => head - offset,

            // [ |  |  |  |  | ]
            // --------^     <--
            //  Offset H
            (head, offset) if offset > head => buffer_len + head - offset,

            _ => unreachable!(),
        }
    }

    /// Creates a new instance with new buffer and restores remainings.
    ///
    /// ## Panics
    /// - If the buffer is empty.
    pub fn resize(self, buffer: &mut [T]) -> Delay<'_, T> {
        for iota in 1..=buffer.len() {
            // H -6         -1
            // [ | | | | | | ]
            // <-------------x
            let new_delay = buffer.len() - iota;
            // -6       -1 H
            // [ | | | | | | ]
            // ----------x  <-
            let old_delay = self.index(NonZeroUsize::new(iota).unwrap());
            core::mem::swap(&mut buffer[new_delay], &mut self.buffer[old_delay]);
        }
        Delay::new(buffer)
    }
}

impl<'a, T> Delay<'a, T>
where
    T: Default + Clone,
{
    /// Clears the buffer with default values.
    pub fn clear(&mut self) {
        self.buffer.fill(T::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let mut binding = [0.0; 3];
        let mut buffer = Delay::new(&mut binding);
        buffer.write(1.0);
        buffer.write(2.0);
        buffer.write(3.0);
        assert_eq!(3.0, *buffer.read(1.try_into().unwrap()));
        assert_eq!(2.0, *buffer.read(2.try_into().unwrap()));
        assert_eq!(1.0, *buffer.read(3.try_into().unwrap()));
        assert_eq!(3.0, *buffer.read(4.try_into().unwrap()));

        buffer.write(4.0);
        assert_eq!(4.0, *buffer.read(1.try_into().unwrap()));
        assert_eq!(3.0, *buffer.read(2.try_into().unwrap()));
        assert_eq!(2.0, *buffer.read(3.try_into().unwrap()));
        assert_eq!(4.0, *buffer.read(4.try_into().unwrap()));
    }

    #[test]
    fn expand() {
        let mut buffer1 = [1, 2];
        let mut buffer2 = [0; 3];
        let delay = Delay::new(&mut buffer1).resize(&mut buffer2);
        assert_eq!([0, 1, 2], delay.buffer)
    }

    #[test]
    fn shrink() {
        let mut buffer1 = [1, 2, 3];
        let mut buffer2 = [0; 2];
        let delay = Delay::new(&mut buffer1).resize(&mut buffer2);
        assert_eq!([2, 3], delay.buffer)
    }

    #[test]
    fn from_vec() {
        let mut buffer = vec![0, 1, 2];
        Delay::new(&mut buffer.as_mut());
    }
}
