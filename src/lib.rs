/// 
/// A simple dsp node for multiplying the amplitude of its inputs by some given multiplier.
///


extern crate dsp;


/// A simple dsp node for multiplying the amplitude of its inputs by the held multiplier.
#[derive(Copy, Clone, Debug)]
pub struct Volume {
    maybe_prev: Option<f32>,
    pub current: f32,
}


impl Volume {

    /// Constructor for a new default volume. The default volume is 1.0, having no impact on the
    /// input signal.
    pub fn new() -> Volume {
        Volume {
            maybe_prev: None,
            current: 1.0,
        }
    }

    /// Builder for constructing a Volume with some given volume.
    pub fn volume(mut self, volume: f32) -> Volume {
        self.current = volume;
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
            // current buffer, we should interpolate from it to the current volume over the duration of
            // the buffer to avoid clipping.
            Some(prev) if prev != self.current => {
                let volume_diff = self.current - prev;
                let one_frame_vol_diff = volume_diff * (1.0 / settings.frames as f32);
                let mut interpolated_vol = prev;
                for frame in buffer.chunks_mut(settings.channels as usize) {
                    interpolated_vol += one_frame_vol_diff;
                    for channel in frame.iter_mut() {
                        *channel = channel.mul_amp(interpolated_vol)
                    }
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

