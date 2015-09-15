/// 
/// A simple dsp node for multiplying the amplitude of its inputs by some given multiplier.
///


extern crate dsp;
extern crate time_calc as time;


/// A simple dsp node for multiplying the amplitude of its inputs by the held multiplier.
#[derive(Copy, Clone, Debug)]
pub struct Volume {
    maybe_prev: Option<f32>,
    pub current: f32,
    pub interpolation_ms: time::calc::Ms,
}


impl Volume {

    /// Constructor for a new default volume. The default volume is 1.0, having no impact on the
    /// input signal. The default interpolation_ms is 10ms to avoid clipping.
    pub fn new() -> Volume {
        Volume {
            maybe_prev: None,
            current: 1.0,
            interpolation_ms: 10.0,
        }
    }

    /// Builder for constructing a Volume with some given volume.
    pub fn volume(mut self, volume: f32) -> Volume {
        self.current = volume;
        self
    }

    /// Builder for constructing a Volume with some given interpolation length in Ms.
    /// The interpolation length is how long it takes the volume to progress from some previous
    /// volume to a new current volume. It is mainly used to avoid clipping.
    ///
    /// The default interpolation_ms is 10ms - just enough to avoid clipping.
    pub fn interpolation_ms(mut self, ms: time::calc::Ms) -> Volume {
        self.interpolation_ms = ms;
        self
    }

    /// Set the new current volume.
    pub fn set(&mut self, volume: f32) {
        self.current = volume;
    }

}


impl<S> dsp::Node<S> for Volume where S: dsp::Sample {

    #[inline]
    fn audio_requested(&mut self, buffer: &mut [S], settings: dsp::Settings) {
        match self.maybe_prev {

            // If the volume used for the previous buffer is different to the volume used for the
            // current buffer, we should interpolate from it to the current volume to avoid
            // clipping.
            Some(prev) if prev != self.current && self.interpolation_ms > 0.0 => {

                // Calculate the interpolation duration in frames along with the volume increment
                // to use for interpolation.
                let interpolation_frames = ::std::cmp::min(
                    settings.frames as usize,
                    time::Ms(self.interpolation_ms).samples(settings.sample_hz as f64) as usize
                );
                let volume_diff = self.current - prev;
                let volume_increment = volume_diff * (1.0 / interpolation_frames as f32);
                let mut volume = prev;

                // Interpolated frames.
                for frame in 0..interpolation_frames {
                    volume += volume_increment;
                    for channel in 0..(settings.channels as usize) {
                        let idx = frame * (settings.channels as usize) + channel;
                        let sample = &mut buffer[idx];
                        *sample = sample.mul_amp(volume);
                    }
                }

                // Remaining frames.
                let start_of_remaining = interpolation_frames * settings.channels as usize;
                for idx in start_of_remaining..buffer.len() {
                    let sample = &mut buffer[idx];
                    *sample = sample.mul_amp(volume);
                }
            },

            // Otherwise, simply multiply every sample by the current volume.
            _ => for sample in buffer.iter_mut() {
                *sample = sample.mul_amp(self.current);
            },

        }

        // Always set the current volume as the new `maybe_prev`.
        self.maybe_prev = Some(self.current);
    }

}

