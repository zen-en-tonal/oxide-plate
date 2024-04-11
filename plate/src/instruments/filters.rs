use core::num::NonZeroUsize;

use crate::instruments::Delay;

/// [Schroeder All-pass filter](https://ccrma.stanford.edu/~jos/Delay/Schroeder_Allpass_Filters.html)
#[derive(Debug, PartialEq, Eq)]
pub struct APF<'a, T> {
    delay: NonZeroUsize,
    a: T,
    b: T,
    delay_line: Delay<'a, T>,
}

impl<'a, T> APF<'a, T>
where
    T: num_traits::Num + Clone,
{
    /// Creates a new instance.
    ///
    /// ## Panics
    /// - If the buffer is empty.
    /// - If the buffer's length is less than 2.
    pub fn new(buffer: &'a mut [T]) -> APF<'a, T> {
        return Self::with_params(buffer, 1.try_into().unwrap(), T::zero(), T::zero());
    }

    pub fn with_params(buffer: &'a mut [T], delay: NonZeroUsize, a: T, b: T) -> APF<'a, T> {
        return APF {
            delay,
            a,
            b,
            delay_line: Delay::new(buffer),
        };
    }

    pub fn set_params(&mut self, a: T, b: T, delay: NonZeroUsize) {
        self.a = a;
        self.b = b;
        self.delay = delay;
    }

    pub fn sample_buffer(&self, delay: NonZeroUsize) -> &T {
        self.delay_line.read(delay)
    }

    pub fn tick(&mut self, x: T) -> T {
        let z = self.delay_line.read(self.delay);
        let x = x - self.b.clone() * z.clone();
        let y = self.a.clone() * x.clone() + z.clone();
        self.delay_line.write(x);
        y
    }
}

/// Nth order IIR filter.
#[derive(Debug, PartialEq, Eq)]
pub struct IIR<'a, T, const ORDER: usize> {
    a: [T; ORDER],
    b: T,
    z: &'a mut [T],
}

impl<'a, T, const ORDER: usize> IIR<'a, T, ORDER>
where
    T: num_traits::Num + Clone,
{
    /// Creates a new instance.
    pub fn new(buffer: &'a mut [T]) -> Self {
        Self::new_with_params(buffer, core::array::from_fn(|_| T::zero()), T::zero())
    }

    pub fn new_with_params(buffer: &'a mut [T], a: [T; ORDER], b: T) -> Self {
        if ORDER < 1 {
            panic!()
        }
        Self { a, b, z: buffer }
    }

    pub fn set_params(&mut self, a: [T; ORDER], b: T) {
        self.a = a;
        self.b = b;
    }

    pub fn tick(&mut self, x: T) -> T {
        let mut z = T::zero();
        for iota in 0..ORDER {
            z = z + self.z[iota].clone() * self.a[iota].clone();
        }
        let y = z + x * self.b.clone();
        self.z.rotate_right(1);
        self.z[0] = y.clone();
        y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apf() {
        let mut buffer = [0; 2];
        let mut apf = APF::with_params(&mut buffer, 1.try_into().unwrap(), 2, 2);
        assert_eq!(2, apf.tick(1));
        assert_eq!(-1, apf.tick(1));
    }

    #[test]
    fn iir() {
        let mut buffer = [0.0; 1];
        let mut filter = IIR::new_with_params(&mut buffer, [0.5], 1.0);
        assert_eq!(1.0, filter.tick(1.0));
        assert_eq!(1.5, filter.tick(1.0));
    }
}
