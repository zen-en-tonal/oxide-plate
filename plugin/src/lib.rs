use nih_plug::prelude::*;
use plate::*;
use std::{ops::Deref, sync::Arc};

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
}

impl Default for PlatePlugin {
    fn default() -> Self {
        let predelay = vec![0.0; 4096].leak();
        let input_diffusion_1_1 = vec![0.0; INPUT_DIFFUSION_1_1 + 1].leak();
        let input_diffusion_1_2 = vec![0.0; INPUT_DIFFUSION_1_2 + 1].leak();
        let input_diffusion_2_1 = vec![0.0; INPUT_DIFFUSION_2_1 + 1].leak();
        let input_diffusion_2_2 = vec![0.0; INPUT_DIFFUSION_2_2 + 1].leak();
        let decay_diffusion_1_1 = vec![0.0; DECAY_DIFFUSION_1_1 + 1].leak();
        let decay_diffusion_1_2 = vec![0.0; DECAY_DIFFUSION_1_2 + 1].leak();
        let decay_diffusion_2_1 = vec![0.0; DECAY_DIFFUSION_2_1 + 1].leak();
        let decay_diffusion_2_2 = vec![0.0; DECAY_DIFFUSION_2_2 + 1].leak();
        let delay_1 = vec![0.0; DELAY_1 + 1].leak();
        let delay_2 = vec![0.0; DELAY_2 + 1].leak();
        let delay_3 = vec![0.0; DELAY_3 + 1].leak();
        let delay_4 = vec![0.0; DELAY_4 + 1].leak();
        let buffers: PlateBuffers<'_, f32> = PlateBuffers {
            predelay: predelay.as_mut(),
            input_diffusion_1_1: input_diffusion_1_1.as_mut(),
            input_diffusion_1_2: input_diffusion_1_2.as_mut(),
            input_diffusion_2_1: input_diffusion_2_1.as_mut(),
            input_diffusion_2_2: input_diffusion_2_2.as_mut(),
            decay_diffusion_1_1: decay_diffusion_1_1.as_mut(),
            decay_diffusion_1_2: decay_diffusion_1_2.as_mut(),
            decay_diffusion_2_1: decay_diffusion_2_1.as_mut(),
            decay_diffusion_2_2: decay_diffusion_2_2.as_mut(),
            delay_1: delay_1.as_mut(),
            delay_2: delay_2.as_mut(),
            delay_3: delay_3.as_mut(),
            delay_4: delay_4.as_mut(),
        };
        let params = PlatePluginParams::default();
        Self {
            plate: buffers.build((&params).into()),
            params: Arc::new(params),
        }
    }
}

impl Default for PlatePluginParams {
    fn default() -> Self {
        Self {
            predelay: IntParam::new("Pre delay", 1, IntRange::Linear { min: 1, max: 4095 })
                .with_smoother(SmoothingStyle::Logarithmic(50.0)),
            bandwidth: FloatParam::new(
                "Bandwidth",
                0.9995,
                FloatRange::Skewed {
                    min: 0.0001,
                    max: 0.9999,
                    factor: FloatRange::skew_factor(0.0001),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
            input_diffusion_1: FloatParam::new(
                "Input diffusion 1",
                0.750,
                FloatRange::Linear {
                    min: 0.0001,
                    max: 0.9999,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
            input_diffusion_2: FloatParam::new(
                "Input diffusion 2",
                0.625,
                FloatRange::Linear {
                    min: 0.0001,
                    max: 0.9999,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
            decay_diffusion_1: FloatParam::new(
                "Decay diffusion 1",
                0.70,
                FloatRange::Linear {
                    min: 0.0001,
                    max: 0.9999,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
            decay_diffusion_2: FloatParam::new(
                "Decay diffusion 2",
                0.50,
                FloatRange::Linear {
                    min: 0.0001,
                    max: 0.9999,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
            damping: FloatParam::new(
                "Damping",
                0.0005,
                FloatRange::Skewed {
                    min: 0.0001,
                    max: 0.9999,
                    factor: FloatRange::skew_factor(0.0001),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
            decay: FloatParam::new(
                "Decay",
                0.500,
                FloatRange::Linear {
                    min: 0.0001,
                    max: 0.9999,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
        }
    }
}

impl From<&PlatePluginParams> for PlateParams<f32> {
    fn from(value: &PlatePluginParams) -> Self {
        PlateParams {
            predelay: value.predelay.smoothed.next() as usize,
            bandwidth: value.bandwidth.smoothed.next(),
            input_diffusion_1: value.input_diffusion_1.smoothed.next(),
            input_diffusion_2: value.input_diffusion_2.smoothed.next(),
            decay_diffusion_1: value.decay_diffusion_1.smoothed.next(),
            decay_diffusion_2: value.decay_diffusion_2.smoothed.next(),
            damping: value.damping.smoothed.next(),
            decay: value.decay.smoothed.next(),
        }
    }
}

impl Plugin for PlatePlugin {
    const NAME: &'static str = "oxide_plate";
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
        self.plate.set_params(params.into());
        for channel_samples in buffer.iter_samples() {
            for (i, sample) in channel_samples.into_iter().enumerate() {
                *sample = self.plate.process_2ch(&[*sample])[i]
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
    const VST3_CLASS_ID: [u8; 16] = *b"OxidePlateZET   ";
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

        let mut buffer = Buffer::default();

        plugin.process_buffer(&mut buffer);
    }
}
