use crate::types::AudioDevice;

/// List available audio devices (input and output)
pub fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let mut devices = Vec::new();

    // List output devices (for system audio / loopback)
    if let Ok(output_devices) = host.output_devices() {
        let default_output = host.default_output_device()
            .and_then(|d| d.name().ok());
        for device in output_devices {
            if let Ok(name) = device.name() {
                let is_default = default_output.as_ref().map(|d| d == &name).unwrap_or(false);
                devices.push(AudioDevice {
                    name,
                    is_input: false,
                    is_default,
                });
            }
        }
    }

    // List input devices (microphones)
    if let Ok(input_devices) = host.input_devices() {
        let default_input = host.default_input_device()
            .and_then(|d| d.name().ok());
        for device in input_devices {
            if let Ok(name) = device.name() {
                let is_default = default_input.as_ref().map(|d| d == &name).unwrap_or(false);
                devices.push(AudioDevice {
                    name,
                    is_input: true,
                    is_default,
                });
            }
        }
    }

    Ok(devices)
}

/// Start system audio capture (WASAPI loopback) writing to WAV
pub fn start_system_capture(
    output_path: &std::path::Path,
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<std::thread::JoinHandle<()>, String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::atomic::Ordering;

    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or("No default output device found")?;

    let config = device.default_output_config()
        .map_err(|e| format!("Failed to get output config: {}", e))?;

    let spec = hound::WavSpec {
        channels: config.channels(),
        sample_rate: config.sample_rate().0,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let path = output_path.to_path_buf();
    let stop = stop_flag.clone();

    let handle = std::thread::spawn(move || {
        let writer = match hound::WavWriter::create(&path, spec) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[audio] Failed to create WAV writer: {}", e);
                return;
            }
        };
        let writer = std::sync::Arc::new(std::sync::Mutex::new(Some(writer)));
        let writer_clone = writer.clone();

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut guard) = writer_clone.lock() {
                    if let Some(ref mut w) = *guard {
                        for &sample in data {
                            let _ = w.write_sample(sample);
                        }
                    }
                }
            },
            |err| eprintln!("[audio] Stream error: {}", err),
            None,
        );

        match stream {
            Ok(stream) => {
                let _ = stream.play();
                while !stop.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                drop(stream);
            }
            Err(e) => {
                eprintln!("[audio] Failed to build stream: {}", e);
            }
        }

        // Finalize WAV
        if let Ok(mut guard) = writer.lock() {
            if let Some(w) = guard.take() {
                let _ = w.finalize();
            }
        };
    });

    Ok(handle)
}

/// Start microphone capture writing to WAV, optionally using a specific device
pub fn start_mic_capture(
    output_path: &std::path::Path,
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<std::thread::JoinHandle<()>, String> {
    start_mic_capture_device(output_path, stop_flag, None)
}

pub fn start_mic_capture_device(
    output_path: &std::path::Path,
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    device_name: Option<&str>,
) -> Result<std::thread::JoinHandle<()>, String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::atomic::Ordering;

    let host = cpal::default_host();
    let device = if let Some(name) = device_name {
        host.input_devices()
            .map_err(|e| format!("Failed to list input devices: {}", e))?
            .find(|d| d.name().ok().as_deref() == Some(name))
            .ok_or_else(|| format!("Microphone '{}' not found", name))?
    } else {
        host.default_input_device()
            .ok_or("No default input device found")?
    };

    let config = device.default_input_config()
        .map_err(|e| format!("Failed to get input config: {}", e))?;

    let spec = hound::WavSpec {
        channels: config.channels(),
        sample_rate: config.sample_rate().0,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let path = output_path.to_path_buf();
    let stop = stop_flag.clone();

    let handle = std::thread::spawn(move || {
        let writer = match hound::WavWriter::create(&path, spec) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[audio] Failed to create mic WAV writer: {}", e);
                return;
            }
        };
        let writer = std::sync::Arc::new(std::sync::Mutex::new(Some(writer)));
        let writer_clone = writer.clone();

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut guard) = writer_clone.lock() {
                    if let Some(ref mut w) = *guard {
                        for &sample in data {
                            let _ = w.write_sample(sample);
                        }
                    }
                }
            },
            |err| eprintln!("[audio] Mic stream error: {}", err),
            None,
        );

        match stream {
            Ok(stream) => {
                let _ = stream.play();
                while !stop.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                drop(stream);
            }
            Err(e) => {
                eprintln!("[audio] Failed to build mic stream: {}", e);
            }
        }

        if let Ok(mut guard) = writer.lock() {
            if let Some(w) = guard.take() {
                let _ = w.finalize();
            }
        };
    });

    Ok(handle)
}
