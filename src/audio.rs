use std::time::Duration;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use rodio::source::SineWave;

pub struct SoundPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl SoundPlayer {
    pub fn new() -> Option<Self> {
        match OutputStream::try_default() {
            Ok((stream, handle)) => Some(SoundPlayer {
                _stream: stream,
                stream_handle: handle,
            }),
            Err(e) => {
                eprintln!("Failed to initialize audio: {}", e);
                None
            }
        }
    }

    /// Play a short high beep when eating food (880Hz, 50ms)
    pub fn play_eat(&self) {
        let source = SineWave::new(880.0)
            .take_duration(Duration::from_millis(50))
            .amplify(0.3);
        self.play_source(source);
    }

    /// Play a descending tone on game over (440Hz -> 220Hz effect via two tones)
    pub fn play_death(&self) {
        let tone1 = SineWave::new(440.0)
            .take_duration(Duration::from_millis(150))
            .amplify(0.3);
        let tone2 = SineWave::new(220.0)
            .take_duration(Duration::from_millis(200))
            .amplify(0.3);

        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            sink.append(tone1);
            sink.append(tone2);
            sink.detach();
        }
    }

    /// Play a rising tone on game start (330Hz -> 660Hz effect via two tones)
    pub fn play_start(&self) {
        let tone1 = SineWave::new(330.0)
            .take_duration(Duration::from_millis(100))
            .amplify(0.3);
        let tone2 = SineWave::new(660.0)
            .take_duration(Duration::from_millis(150))
            .amplify(0.3);

        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            sink.append(tone1);
            sink.append(tone2);
            sink.detach();
        }
    }

    fn play_source<S>(&self, source: S)
    where
        S: Source<Item = f32> + Send + 'static,
    {
        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            sink.append(source);
            sink.detach();
        }
    }
}
