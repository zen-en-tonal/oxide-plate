use crate::instruments::*;
use core::{fmt::Debug, marker::PhantomData};

pub struct PlateBuffers<T, V> {
    pub predelay: T,
    pub prefilter: T,

    pub input_diffusion_1_1: T,
    pub input_diffusion_1_2: T,
    pub input_diffusion_2_1: T,
    pub input_diffusion_2_2: T,

    pub decay_diffusion_1_1: T,
    pub decay_diffusion_1_2: T,
    pub decay_diffusion_2_1: T,
    pub decay_diffusion_2_2: T,

    pub dumping_1: T,
    pub dumping_2: T,

    pub delay_1: T,
    pub delay_2: T,
    pub delay_3: T,
    pub delay_4: T,

    pub tank: T,

    pub _t: PhantomData<V>,
}

impl<T, V> PlateBuffers<T, V>
where
    T: AsMut<[V]>,
    V: num_traits::Num + num_traits::Signed + Clone,
{
    pub fn build(&mut self) -> Plate<'_, V> {
        Plate {
            predelay: Delay::new(self.predelay.as_mut()),
            predelay_length: 1,
            prefilter: IIR::new(self.prefilter.as_mut()),
            input_diffusion_1_1: APF::new(self.input_diffusion_1_1.as_mut()),
            input_diffusion_1_2: APF::new(self.input_diffusion_1_2.as_mut()),
            input_diffusion_2_1: APF::new(self.input_diffusion_2_1.as_mut()),
            input_diffusion_2_2: APF::new(self.input_diffusion_2_2.as_mut()),
            tank: self.tank.as_mut(),
            decay_diffusion_1_1: APF::new(self.decay_diffusion_1_1.as_mut()),
            decay_diffusion_1_2: APF::new(self.decay_diffusion_1_2.as_mut()),
            decay_diffusion_2_1: APF::new(self.decay_diffusion_2_1.as_mut()),
            decay_diffusion_2_2: APF::new(self.decay_diffusion_2_2.as_mut()),
            damping_1: IIR::new(self.dumping_1.as_mut()),
            damping_2: IIR::new(self.dumping_2.as_mut()),
            delay_1: Delay::new(self.delay_1.as_mut()),
            delay_2: Delay::new(self.delay_2.as_mut()),
            delay_3: Delay::new(self.delay_3.as_mut()),
            delay_4: Delay::new(self.delay_4.as_mut()),
            decay: V::zero(),
        }
    }
}

pub struct Plate<'a, T> {
    predelay: Delay<'a, T>,
    predelay_length: usize,
    prefilter: IIR<'a, T, 1>,

    input_diffusion_1_1: APF<'a, T>,
    input_diffusion_1_2: APF<'a, T>,
    input_diffusion_2_1: APF<'a, T>,
    input_diffusion_2_2: APF<'a, T>,

    tank: &'a mut [T],

    decay_diffusion_1_1: APF<'a, T>,
    decay_diffusion_1_2: APF<'a, T>,
    decay_diffusion_2_1: APF<'a, T>,
    decay_diffusion_2_2: APF<'a, T>,

    damping_1: IIR<'a, T, 1>,
    damping_2: IIR<'a, T, 1>,

    delay_1: Delay<'a, T>,
    delay_2: Delay<'a, T>,
    delay_3: Delay<'a, T>,
    delay_4: Delay<'a, T>,

    decay: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlateParams<T> {
    pub predelay: usize,

    pub bandwidth: T,

    pub input_diffusion_1: T,
    pub input_diffusion_2: T,

    pub decay_diffusion_1: T,
    pub decay_diffusion_2: T,

    pub decay_modulation: isize,

    pub damping: T,

    pub decay: T,
}

impl Default for PlateParams<f32> {
    fn default() -> Self {
        Self {
            predelay: 1,
            bandwidth: 0.9995,
            input_diffusion_1: 0.750,
            input_diffusion_2: 0.625,
            decay_diffusion_1: 0.70,
            decay_diffusion_2: 0.50,
            damping: 0.0005,
            decay: 0.50,
            decay_modulation: 0,
        }
    }
}

impl Default for PlateParams<f64> {
    fn default() -> Self {
        Self {
            predelay: 1,
            bandwidth: 0.9995,
            input_diffusion_1: 0.750,
            input_diffusion_2: 0.625,
            decay_diffusion_1: 0.70,
            decay_diffusion_2: 0.50,
            damping: 0.0005,
            decay: 0.50,
            decay_modulation: 0,
        }
    }
}

pub const INPUT_DIFFUSION_1_1: usize = 142;
pub const INPUT_DIFFUSION_1_2: usize = 107;
pub const INPUT_DIFFUSION_2_1: usize = 379;
pub const INPUT_DIFFUSION_2_2: usize = 277;

pub const DECAY_DIFFUSION_1_1: usize = 672;
pub const DECAY_DIFFUSION_1_2: usize = 908;
pub const DECAY_DIFFUSION_2_1: usize = 1800;
pub const DECAY_DIFFUSION_2_2: usize = 2656;

pub const EXCURSION: usize = 16;

pub const DELAY_1: usize = 4453;
pub const DELAY_2: usize = 3720;
pub const DELAY_3: usize = 4217;
pub const DELAY_4: usize = 3163;

impl<'a, T> Plate<'a, T>
where
    T: num_traits::Num + num_traits::One + num_traits::Signed + Clone,
{
    pub fn set_params(&mut self, params: PlateParams<T>) {
        self.predelay_length = params.predelay;
        self.prefilter
            .set_params([T::one() - params.bandwidth.clone()], params.bandwidth);

        self.input_diffusion_1_1.set_params(
            params.input_diffusion_1.clone(),
            params.input_diffusion_1.clone(),
            INPUT_DIFFUSION_1_1.try_into().unwrap(),
        );
        self.input_diffusion_1_2.set_params(
            params.input_diffusion_1.clone(),
            params.input_diffusion_1.clone(),
            INPUT_DIFFUSION_1_2.try_into().unwrap(),
        );
        self.input_diffusion_2_1.set_params(
            params.input_diffusion_2.clone(),
            params.input_diffusion_2.clone(),
            INPUT_DIFFUSION_2_1.try_into().unwrap(),
        );
        self.input_diffusion_2_2.set_params(
            params.input_diffusion_2.clone(),
            params.input_diffusion_2.clone(),
            INPUT_DIFFUSION_2_2.try_into().unwrap(),
        );

        self.decay_diffusion_1_1.set_params(
            -params.decay_diffusion_1.clone(),
            -params.decay_diffusion_1.clone(),
            ((DECAY_DIFFUSION_1_1 as isize + params.decay_modulation) as usize)
                .try_into()
                .unwrap(),
        );
        self.decay_diffusion_1_2.set_params(
            -params.decay_diffusion_1.clone(),
            -params.decay_diffusion_1.clone(),
            ((DECAY_DIFFUSION_1_2 as isize + params.decay_modulation) as usize)
                .try_into()
                .unwrap(),
        );
        self.decay_diffusion_2_1.set_params(
            params.decay_diffusion_2.clone(),
            params.decay_diffusion_2.clone(),
            DECAY_DIFFUSION_2_1.try_into().unwrap(),
        );
        self.decay_diffusion_2_2.set_params(
            params.decay_diffusion_2.clone(),
            params.decay_diffusion_2.clone(),
            DECAY_DIFFUSION_2_2.try_into().unwrap(),
        );

        self.damping_1
            .set_params([params.damping.clone()], T::one() - params.damping.clone());
        self.damping_2
            .set_params([params.damping.clone()], T::one() - params.damping.clone());

        self.decay = params.decay;
    }

    pub fn process(&mut self, x: &[T]) {
        let mut acc = mean(x);

        self.predelay.write(acc.clone());
        acc = self
            .predelay
            .read(self.predelay_length.try_into().unwrap())
            .clone();

        acc = self.prefilter.tick(acc.clone());

        acc = self.input_diffusion_1_1.tick(acc.clone());
        acc = self.input_diffusion_1_2.tick(acc.clone());
        acc = self.input_diffusion_2_1.tick(acc.clone());
        acc = self.input_diffusion_2_2.tick(acc.clone());

        let mut tank1 = acc.clone() + self.tank[0].clone();
        tank1 = self.decay_diffusion_1_1.tick(tank1.clone());
        self.delay_1.write(tank1.clone());
        tank1 = self.delay_1.read(DELAY_1.try_into().unwrap()).clone();
        tank1 = self.damping_1.tick(tank1.clone());
        tank1 = self.decay.clone() * tank1.clone();
        tank1 = self.decay_diffusion_2_1.tick(tank1.clone());
        self.delay_2.write(tank1.clone());
        tank1 = self.delay_2.read(DELAY_2.try_into().unwrap()).clone();
        tank1 = self.decay.clone() * tank1;

        let mut tank2 = acc.clone() + self.tank[1].clone();
        tank2 = self.decay_diffusion_1_2.tick(tank2.clone());
        self.delay_3.write(tank2.clone());
        tank2 = self.delay_3.read(DELAY_3.try_into().unwrap()).clone();
        tank2 = self.damping_2.tick(tank2.clone());
        tank2 = self.decay.clone() * tank2.clone();
        tank2 = self.decay_diffusion_2_2.tick(tank2.clone());
        self.delay_4.write(tank2.clone());
        tank2 = self.delay_4.read(DELAY_4.try_into().unwrap()).clone();
        tank2 = self.decay.clone() * tank2;

        self.tank[0] = tank2;
        self.tank[1] = tank1;
    }
}

impl<'a, T> Plate<'a, T>
where
    T: num_traits::Num + num_traits::NumAssign + num_traits::Signed + Clone,
{
    pub fn process_2ch(&mut self, x: &[T]) -> [T; 2] {
        self.process(x);

        let mut left_acc = self.delay_3.read(266.try_into().unwrap()).clone();
        left_acc += self.delay_3.read(2974.try_into().unwrap()).clone();
        left_acc -= self
            .decay_diffusion_2_2
            .sample_buffer(1913.try_into().unwrap())
            .clone();
        left_acc += self.delay_4.read(1996.try_into().unwrap()).clone();
        left_acc -= self.delay_1.read(1990.try_into().unwrap()).clone();
        left_acc -= self
            .decay_diffusion_2_1
            .sample_buffer(187.try_into().unwrap())
            .clone();
        left_acc -= self.delay_2.read(1066.try_into().unwrap()).clone();

        let mut right_acc = self.delay_1.read(353.try_into().unwrap()).clone();
        right_acc += self.delay_1.read(3627.try_into().unwrap()).clone();
        right_acc -= self
            .decay_diffusion_2_1
            .sample_buffer(1228.try_into().unwrap())
            .clone();
        right_acc += self.delay_2.read(2673.try_into().unwrap()).clone();
        right_acc -= self.delay_3.read(2111.try_into().unwrap()).clone();
        right_acc -= self
            .decay_diffusion_2_2
            .sample_buffer(335.try_into().unwrap())
            .clone();
        right_acc -= self.delay_4.read(121.try_into().unwrap()).clone();

        [left_acc, right_acc]
    }
}

fn mean<T>(xs: &[T]) -> T
where
    T: num_traits::NumOps + num_traits::One + Clone,
{
    let den = xs
        .iter()
        .map(|_| T::one())
        .reduce(|acc, x| acc + x)
        .unwrap();
    let num = xs.iter().cloned().reduce(|acc, x| acc + x).unwrap();

    num / den
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn burst() {
        let mut buffers = PlateBuffers {
            predelay: vec![0.0; 4096],
            input_diffusion_1_1: vec![0.0; INPUT_DIFFUSION_1_1 + 1],
            input_diffusion_1_2: vec![0.0; INPUT_DIFFUSION_1_2 + 1],
            input_diffusion_2_1: vec![0.0; INPUT_DIFFUSION_2_1 + 1],
            input_diffusion_2_2: vec![0.0; INPUT_DIFFUSION_2_2 + 1],
            decay_diffusion_1_1: vec![0.0; DECAY_DIFFUSION_1_1 + 1],
            decay_diffusion_1_2: vec![0.0; DECAY_DIFFUSION_1_2 + 1],
            decay_diffusion_2_1: vec![0.0; DECAY_DIFFUSION_2_1 + 1],
            decay_diffusion_2_2: vec![0.0; DECAY_DIFFUSION_2_2 + 1],
            delay_1: vec![0.0; DELAY_1 + 1],
            delay_2: vec![0.0; DELAY_2 + 1],
            delay_3: vec![0.0; DELAY_3 + 1],
            delay_4: vec![0.0; DELAY_4 + 1],
            prefilter: vec![0.0],
            dumping_1: vec![0.0],
            dumping_2: vec![0.0],
            tank: vec![0.0, 0.0],
            _t: PhantomData,
        };
        let mut plate: Plate<'_, f64> = buffers.build();

        for t in 0..500 {
            let burst = if t < 50 {
                (2.0 * 3.14 * t as f64 / 50.0).sin()
            } else {
                0.0
            };
            let y = plate.process_2ch(&[burst]);
            println!("{},{},{},{}", t, burst, y[0], y[1]);
        }
    }
}
