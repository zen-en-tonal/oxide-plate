use nih_plug::prelude::*;
use plate::*;
use std::{marker::PhantomData, ops::Deref, sync::Arc};

mod instruments;
mod plate;

struct PlatePlugin {
    params: Arc<PlatePluginParams>,
    plate: Plate<'static, f32>,
}

#[derive(Params, Debug)]
struct PlatePluginParams {
    #[id = "predelay"]
    pub predelay: IntParam,
    #[id = "bandwidth"]
    pub bandwidth: FloatParam,
    #[id = "input_diffusion_1"]
    pub input_diffusion_1: FloatParam,
    #[id = "input_diffusion_2"]
    pub input_diffusion_2: FloatParam,
    #[id = "decay_diffusion_1"]
    pub decay_diffusion_1: FloatParam,
    #[id = "decay_diffusion_2"]
    pub decay_diffusion_2: FloatParam,
    #[id = "damping"]
    pub damping: FloatParam,
    #[id = "decay"]
    pub decay: FloatParam,
    #[id = "wet"]
    pub wet: FloatParam,
    #[id = "decay_mod"]
    pub decay_mod: IntParam,
}

impl Default for PlatePlugin {
    fn default() -> Self {
        let buffers = Box::new(PlateBuffers {
            predelay: vec![0.0; 4096],
            input_diffusion_1_1: vec![0.0; INPUT_DIFFUSION_1_1 + 1],
            input_diffusion_1_2: vec![0.0; INPUT_DIFFUSION_1_2 + 1],
            input_diffusion_2_1: vec![0.0; INPUT_DIFFUSION_2_1 + 1],
            input_diffusion_2_2: vec![0.0; INPUT_DIFFUSION_2_2 + 1],
            decay_diffusion_1_1: vec![0.0; DECAY_DIFFUSION_1_1 + EXCURSION + 1],
            decay_diffusion_1_2: vec![0.0; DECAY_DIFFUSION_1_2 + EXCURSION + 1],
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
        });
        Self {
            params: Arc::new(PlatePluginParams::default()),
            plate: Box::leak(buffers).build(),
        }
    }
}

impl Default for PlatePluginParams {
    fn default() -> Self {
        Self {
            predelay: IntParam::new("Pre delay", 50, IntRange::Linear { min: 1, max: 4095 }),
            bandwidth: FloatParam::new(
                "Bandwidth",
                0.9995,
                FloatRange::Skewed {
                    min: 0.0001,
                    max: 0.9999,
                    factor: FloatRange::skew_factor(0.0001),
                },
            ),
            input_diffusion_1: FloatParam::new(
                "Input diffusion 1",
                0.750,
                FloatRange::Linear {
                    min: 0.0001,
                    max: 0.9999,
                },
            ),
            input_diffusion_2: FloatParam::new(
                "Input diffusion 2",
                0.625,
                FloatRange::Linear {
                    min: 0.0001,
                    max: 0.9999,
                },
            ),
            decay_diffusion_1: FloatParam::new(
                "Decay diffusion 1",
                0.70,
                FloatRange::Linear {
                    min: 0.0001,
                    max: 0.9999,
                },
            ),
            decay_diffusion_2: FloatParam::new(
                "Decay diffusion 2",
                0.50,
                FloatRange::Linear {
                    min: 0.0001,
                    max: 0.9999,
                },
            ),
            damping: FloatParam::new(
                "Damping",
                0.0005,
                FloatRange::Skewed {
                    min: 0.0001,
                    max: 0.9999,
                    factor: FloatRange::skew_factor(0.0001),
                },
            ),
            decay: FloatParam::new(
                "Decay",
                0.500,
                FloatRange::Linear {
                    min: 0.0001,
                    max: 0.9999,
                },
            ),
            wet: FloatParam::new("Wet", 0.500, FloatRange::Linear { min: 0.0, max: 1.0 }),
            decay_mod: IntParam::new(
                "Decay mod",
                0,
                IntRange::Linear {
                    min: -(EXCURSION as i32 - 1),
                    max: (EXCURSION as i32 - 1),
                },
            ),
        }
    }
}

impl From<&PlatePluginParams> for PlateParams<f32> {
    fn from(value: &PlatePluginParams) -> Self {
        PlateParams {
            predelay: value.predelay.value() as usize,
            bandwidth: value.bandwidth.smoothed.next(),
            input_diffusion_1: value.input_diffusion_1.smoothed.next(),
            input_diffusion_2: value.input_diffusion_2.smoothed.next(),
            decay_diffusion_1: value.decay_diffusion_1.smoothed.next(),
            decay_diffusion_2: value.decay_diffusion_2.smoothed.next(),
            damping: value.damping.smoothed.next(),
            decay: value.decay.smoothed.next(),
            decay_modulation: value.decay_mod.smoothed.next() as isize,
        }
    }
}

impl Plugin for PlatePlugin {
    const NAME: &'static str = "oxide plate";
    const VENDOR: &'static str = "zen-en-tonal";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),

            aux_input_ports: &[],
            aux_output_ports: &[],

            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.process_buffer(buffer)
    }

    fn deactivate(&mut self) {}
}

impl PlatePlugin {
    fn process_buffer(&mut self, buffer: &mut Buffer) -> ProcessStatus {
        let params: &PlatePluginParams = self.params.deref();
        let wet = params.wet.smoothed.next();
        self.plate.set_params(params.into());
        for mut samples in buffer.iter_samples() {
            // The following safe code will crash DAW.
            // let inputs: Vec<f32> = samples.iter_mut().map(|x| *x).collect();
            let inputs = unsafe { [*samples.get_unchecked_mut(0), *samples.get_unchecked_mut(1)] };
            let plate_out = self.plate.process_2ch(inputs.as_ref());
            for (out, y) in samples.iter_mut().zip(plate_out) {
                *out = (1.0 - wet) * *out + wet * y;
            }
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for PlatePlugin {
    const CLAP_ID: &'static str = "com.github.zen-en-tonal.oxide-plate";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A reverb.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for PlatePlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"OxidePlateZETPlg";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(PlatePlugin);
nih_export_vst3!(PlatePlugin);

#[cfg(test)]
mod tests {
    use nih_plug::buffer::Buffer;

    use crate::PlatePlugin;

    #[test]
    fn basic() {
        let mut plugin = PlatePlugin::default();

        let mut real_buffers = vec![vec![1.0; 44100 * 5]; 2];
        let mut buffer = Buffer::default();

        unsafe {
            buffer.set_slices(44100 * 5, |output_slices| {
                let (first_channel, other_channels) = real_buffers.split_at_mut(1);
                *output_slices = vec![&mut first_channel[0], &mut other_channels[0]];
            })
        };

        plugin.process_buffer(&mut buffer);
    }
}
