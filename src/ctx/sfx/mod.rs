use std::{sync::Arc, borrow::Cow};

use parking_lot::RwLock;
use rodio::{
    OutputStream, 
    Sink, 
    dynamic_mixer::{DynamicMixerController, mixer}, 
    OutputStreamHandle, 
    source::{Buffered, Zero}, 
    Decoder, 
    Source, cpal::FromSample, 
};

/// サウンドモジュール
pub struct SfxCtx(Arc<RwLock<SfxModule>>);
impl SfxCtx {
    pub fn new(volume: f32) -> Result<
        Self, 
        Box<dyn std::error::Error>, 
    > {
        Ok(Self(Arc::new(RwLock::new(SfxModule::new(volume)?))))
    }

    pub fn set_volume(&self, volume: f32) {
        self.0.write().set_volume(volume)
    }

    /// 音の再生
    pub fn play<T: Source<Item = f32> + Send + 'static> (
        &self, 
        src: T, 
    ) { self.0.read().play(src) }

    /// リソースの追加
    pub fn add_resource(
        &self, 
        name: impl Into<Cow<'static, str>>, 
        file: Decoder<std::fs::File>, 
    ) -> Option<Buffered<Decoder<std::fs::File>>> {
        self.0.write().add_resource(name, file)
    }

    /// リソースの再生
    pub fn play_resource<
        Q: ?Sized + Eq + std::hash::Hash, 
        S: rodio::Sample, 
        T: Source<Item = S> + Send + 'static, 
    >(
        &self, 
        name: &Q, 
        f: impl FnMut(Buffered<Decoder<std::fs::File>>) -> T, 
    ) -> bool where
        Cow<'static, str>: std::borrow::Borrow<Q>, 
        f32: FromSample<S>, 
    {
        self.0.read().play_resource(name, f)
    }

}

/// サウンド関係のモジュール
struct SfxModule {
    _stream: OutputStream, 
    _stream_handle: OutputStreamHandle, 
    sink: Sink, 
    mixer_ctrl: Arc<DynamicMixerController<f32>>, 
    res_mngr: SfxResMngr, 
}
impl SfxModule {
    const MIXER_CHANNEL: u16 = 16;
    const MIXER_SAMPLE_RATE: u32 = 44100;
    fn new(volume: f32) -> Result<Self, Box<dyn std::error::Error>> {
        // ストリーム出力の初期化
        let (
            stream, 
            stream_handle, 
        ) = OutputStream::try_default()?;
        
        // ミキサの初期化
        let (
            mixer_ctrl, 
            mixer, 
        ) = mixer(Self::MIXER_CHANNEL, Self::MIXER_SAMPLE_RATE);

        // シンクの初期化
        let sink = Sink::try_new(&stream_handle)?;

        // 音量のない音を準備する
        mixer_ctrl.add(Zero::new(Self::MIXER_CHANNEL, Self::MIXER_SAMPLE_RATE));

        // 音量を初期化
        sink.set_volume(volume);

        // 音源にミキサを入力
        sink.append(mixer);

        // 再生開始
        sink.play();

        // リソースマネージャの初期化
        let res_mngr = SfxResMngr::new();

        Ok(Self {
            _stream: stream,
            _stream_handle: stream_handle,
            sink,
            mixer_ctrl,
            res_mngr,
        })
    }

    /// 音量の設定
    fn set_volume(
        &mut self, 
        volume: f32, 
    ) {
        self.sink.set_volume(volume)
    }

    /// 音の再生
    fn play<T: Source<Item = f32> + Send + 'static> (
        &self, 
        src: T, 
    ) { self.mixer_ctrl.add(src) }

    /// リソースの追加
    fn add_resource(
        &mut self, 
        name: impl Into<Cow<'static, str>>, 
        file: Decoder<std::fs::File>, 
    ) -> Option<Buffered<Decoder<std::fs::File>>> {
        self.res_mngr.add(name, file)
    }

    /// リソースの再生
    fn play_resource<
        Q: ?Sized + Eq + std::hash::Hash, 
        S: rodio::Sample, 
        T: Source<Item = S> + Send + 'static, 
    >(
        &self, 
        name: &Q, 
        f: impl FnMut(Buffered<Decoder<std::fs::File>>) -> T, 
    ) -> bool where
        Cow<'static, str>: std::borrow::Borrow<Q>, 
        f32: FromSample<S>, 
    {
        self.res_mngr.play(name, self, f)
    }
}

/// リソース管理機構
struct SfxResMngr {
    resources: hashbrown::HashMap<
        Cow<'static, str>, 
        Buffered<Decoder<std::fs::File>>
    >, 
}
impl SfxResMngr {
    fn new() -> Self { Self { resources: Default::default() } }
    fn add (
        &mut self, 
        name: impl Into<Cow<'static, str>>, 
        file: Decoder<std::fs::File>, 
    ) -> Option<Buffered<Decoder<std::fs::File>>> {
        self.resources.insert(
            name.into(), 
            file.buffered(), 
        )
    }
    fn play<
        Q: ?Sized + Eq + std::hash::Hash, 
        S: rodio::Sample, 
        T: Source<Item = S> + Send + 'static, 
    >(
        &self, 
        name: &Q, 
        sfx_ctx: &SfxModule, 
        mut f: impl FnMut(
            Buffered<Decoder<std::fs::File>>
        ) -> T
    ) -> bool where
        Cow<'static, str>: std::borrow::Borrow<Q>, 
        f32: FromSample<S>, 
    {
        if let Some(res) = self.resources.get(name)
        {
            sfx_ctx.play(f(res.clone()).convert_samples());
            true
        } else { false }
    }
}