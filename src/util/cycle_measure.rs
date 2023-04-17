/// サイクルの長さを計測します。
pub struct CycleMeasure {
    pre_update_time: std::time::Instant, 
    pub cps: f32, 
    pub dur: f32, 
}
impl CycleMeasure {
    pub fn new() -> Self { Self {
        pre_update_time: std::time::Instant::now(), 
        dur: 1_000_000_000. / 60., 
        cps: 1_000_000_000. / (1_000_000_000. / 60.), 
    }}
    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        let dur = now - self.pre_update_time;
        self.pre_update_time = now;
        self.dur = dur.as_nanos() as f32 / 1_000_000_000.;
        self.cps = 1_000_000_000. / dur.as_nanos() as f32;
    }
}