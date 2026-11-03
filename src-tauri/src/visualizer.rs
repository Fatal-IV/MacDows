//! The music visualizer: what the speakers play, split into a few frequency bands.
//!
//! The default playback device is captured in loopback (a copy of the mix, nothing is recorded to disk and nothing
//! leaves the app), analysed with an FFT and sent to the notch as `audio-visualization` events. Capture runs only
//! while a media session is playing.

use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use serde::Serialize;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use windows::Win32::Media::Audio::{
    eMultimedia, eRender, IAudioCaptureClient, IAudioClient, IMMDeviceEnumerator, MMDeviceEnumerator, AUDCLNT_BUFFERFLAGS_SILENT,
    AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, WAVEFORMATEX,
};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED};

pub const FFT_SIZE: usize = 1024;
pub const BANDS: usize = 5;
/// Band edges in Hz: bass, low mids, mids, upper mids, treble.
const EDGES: [f32; BANDS + 1] = [50.0, 200.0, 600.0, 2000.0, 5000.0, 14000.0];
/// Quietest and loudest level shown, in dB relative to full scale.
const FLOOR_DB: f32 = -62.0;
const CEIL_DB: f32 = -14.0;

#[derive(Serialize, Clone, Debug)]
struct Frame {
    bands: Vec<f32>,
}

/// Turns `FFT_SIZE` mono samples into band levels between 0 and 1.
pub fn analyze(samples: &[f32], sample_rate: u32, planner: &mut FftPlanner<f32>) -> [f32; BANDS] {
    let mut out = [0.0; BANDS];
    if samples.len() < FFT_SIZE || sample_rate == 0 {
        return out;
    }
    let fft = planner.plan_fft_forward(FFT_SIZE);
    let mut buf: Vec<Complex<f32>> = samples[samples.len() - FFT_SIZE..]
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let hann = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE as f32 - 1.0)).cos();
            Complex::new(s * hann, 0.0)
        })
        .collect();
    fft.process(&mut buf);
    let bin_hz = sample_rate as f32 / FFT_SIZE as f32;
    for (band, level) in out.iter_mut().enumerate() {
        let lo = (EDGES[band] / bin_hz).floor().max(1.0) as usize;
        let hi = ((EDGES[band + 1] / bin_hz).ceil() as usize).min(FFT_SIZE / 2).max(lo + 1);
        // The loudest bin of the band: a single steady tone must light its band fully.
        let peak = buf[lo..hi].iter().map(|c| c.norm()).fold(0.0f32, f32::max);
        // A full-scale sine gives a bin magnitude of about FFT_SIZE / 4 with the Hann window.
        let amplitude = peak / (FFT_SIZE as f32 / 4.0);
        let db = 20.0 * (amplitude + 1e-9).log10();
        *level = ((db - FLOOR_DB) / (CEIL_DB - FLOOR_DB)).clamp(0.0, 1.0);
    }
    out
}

/// Rises quickly and falls slowly, so the bars do not flicker.
pub fn smooth(previous: [f32; BANDS], next: [f32; BANDS]) -> [f32; BANDS] {
    let mut out = [0.0; BANDS];
    for i in 0..BANDS {
        out[i] = next[i].max(previous[i] * 0.8);
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Capture
// ---------------------------------------------------------------------------------------------

/// Reads loopback packets until media stops playing or the device changes. Returns when the session should be
/// reopened.
unsafe fn capture_session(app: &AppHandle, planner: &mut FftPlanner<f32>) -> Option<()> {
    unsafe {
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia).ok()?;
        let client: IAudioClient = device.Activate(CLSCTX_ALL, None).ok()?;
        let format_ptr: *mut WAVEFORMATEX = client.GetMixFormat().ok()?;
        let format = *format_ptr;
        let (channels, bits, rate) = (format.nChannels as usize, format.wBitsPerSample, format.nSamplesPerSec);
        let initialized = client.Initialize(AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, 2_000_000, 0, format_ptr, None);
        CoTaskMemFree(Some(format_ptr as *const _));
        initialized.ok()?;
        let capture: IAudioCaptureClient = client.GetService().ok()?;
        client.Start().ok()?;

        let mut window: Vec<f32> = Vec::with_capacity(FFT_SIZE * 2);
        let mut levels = [0.0f32; BANDS];
        let mut last_emit = std::time::Instant::now();
        while crate::media::is_playing() {
            std::thread::sleep(Duration::from_millis(10));
            loop {
                let packet = capture.GetNextPacketSize().ok()?;
                if packet == 0 {
                    break;
                }
                let mut data = std::ptr::null_mut();
                let (mut frames, mut flags) = (0u32, 0u32);
                capture.GetBuffer(&mut data, &mut frames, &mut flags, None, None).ok()?;
                let count = frames as usize;
                if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 || data.is_null() {
                    window.extend(std::iter::repeat_n(0.0, count));
                } else if bits == 32 {
                    let s = std::slice::from_raw_parts(data as *const f32, count * channels);
                    window.extend(s.chunks_exact(channels).map(|f| f.iter().sum::<f32>() / channels as f32));
                } else if bits == 16 {
                    let s = std::slice::from_raw_parts(data as *const i16, count * channels);
                    window.extend(s.chunks_exact(channels).map(|f| f.iter().map(|v| *v as f32 / 32768.0).sum::<f32>() / channels as f32));
                }
                capture.ReleaseBuffer(frames).ok()?;
            }
            if window.len() > FFT_SIZE * 4 {
                window.drain(..window.len() - FFT_SIZE * 2);
            }
            // About 30 frames a second.
            if last_emit.elapsed() >= Duration::from_millis(33) && window.len() >= FFT_SIZE {
                levels = smooth(levels, analyze(&window, rate, planner));
                let _ = app.emit("audio-visualization", Frame { bands: levels.to_vec() });
                last_emit = std::time::Instant::now();
            }
        }
        let _ = client.Stop();
        let _ = app.emit("audio-visualization", Frame { bands: vec![0.0; BANDS] });
        Some(())
    }
}

/// Starts the capture thread: it waits for media to play, then analyses the output until it stops.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let mut planner = FftPlanner::<f32>::new();
        loop {
            if crate::media::is_playing() {
                if unsafe { capture_session(&app, &mut planner) }.is_none() {
                    // No device or it went away: try again in a moment.
                    std::thread::sleep(Duration::from_secs(2));
                }
            } else {
                std::thread::sleep(Duration::from_millis(300));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(hz: f32, amplitude: f32, rate: u32) -> Vec<f32> {
        (0..FFT_SIZE * 2).map(|i| amplitude * (2.0 * std::f32::consts::PI * hz * i as f32 / rate as f32).sin()).collect()
    }

    fn loudest(levels: [f32; BANDS]) -> usize {
        levels.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).map(|(i, _)| i).unwrap()
    }

    #[test]
    fn a_steady_tone_lights_its_own_band() {
        let mut planner = FftPlanner::new();
        for (hz, band) in [(100.0, 0), (400.0, 1), (1000.0, 2), (3000.0, 3), (9000.0, 4)] {
            let levels = analyze(&tone(hz, 0.5, 48_000), 48_000, &mut planner);
            assert_eq!(loudest(levels), band, "{hz} Hz gave {levels:?}");
            assert!(levels[band] > 0.6, "{hz} Hz is too quiet: {levels:?}");
        }
    }

    #[test]
    fn silence_and_too_few_samples_give_flat_bars() {
        let mut planner = FftPlanner::new();
        assert_eq!(analyze(&vec![0.0; FFT_SIZE], 48_000, &mut planner), [0.0; BANDS]);
        assert_eq!(analyze(&[0.5; 10], 48_000, &mut planner), [0.0; BANDS]);
        assert_eq!(analyze(&tone(1000.0, 0.5, 48_000), 0, &mut planner), [0.0; BANDS]);
    }

    #[test]
    fn a_louder_tone_shows_a_higher_bar() {
        let mut planner = FftPlanner::new();
        let quiet = analyze(&tone(1000.0, 0.02, 48_000), 48_000, &mut planner)[2];
        let loud = analyze(&tone(1000.0, 0.8, 48_000), 48_000, &mut planner)[2];
        assert!(loud > quiet + 0.2, "{quiet} vs {loud}");
    }

    #[test]
    fn bars_rise_at_once_and_fall_gradually() {
        let up = smooth([0.1; BANDS], [0.9; BANDS]);
        assert_eq!(up, [0.9; BANDS]);
        let down = smooth([0.9; BANDS], [0.0; BANDS]);
        assert!(down[0] > 0.6 && down[0] < 0.9);
    }
}
