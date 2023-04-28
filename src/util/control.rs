/// 反転可能なコントロール
/// 
/// 順行・逆行を排他的に入力することが出来ます。
pub struct RevCtrl {
    triggering: bool, 
    count: u32, 
    mode: RevMode, 
}
impl Default for RevCtrl {
    fn default() -> Self {
        Self {
            triggering: false,  
            count: 0, 
            mode: RevMode::Brake, 
        }
    }
}
impl RevCtrl {
    pub fn input(
        &mut self, 
        mode: RevMode, 
        state: winit::event::ElementState, 
    ) { match (mode, state) {
        (
            RevMode::Forward, winit::event::ElementState::Pressed
        ) => {
            self.triggering = true;
            self.mode = RevMode::Forward;
        }, 
        (
            RevMode::Forward, winit::event::ElementState::Released
        ) if self.mode == RevMode::Forward => {
            self.triggering = false;
            self.mode = RevMode::Brake;
        },
        (
            RevMode::Backward, winit::event::ElementState::Pressed
        ) => {
            self.triggering = true;
            self.mode = RevMode::Backward;
        }, 
        (
            RevMode::Backward, winit::event::ElementState::Released
        ) if self.mode == RevMode::Backward => {
            self.triggering = false;
            self.mode = RevMode::Brake;
        },
        _ => {}, 
    }}

    /// 入力カウントの更新処理
    pub fn update(&mut self) { if self.triggering {
        self.count = self.count.checked_add(1)
            .map_or_else(|| u32::MAX, |c| c);
    } else {
        self.count = 0
    }}

    /// 順行・逆行の状態の取得
    pub fn get_mode(&self) -> RevMode { self.mode }

    /// トリガされているか
    pub fn is_triggered(&self) -> bool { self.triggering }

    /// 入力カウントの取得
    pub fn get_trig_count(&self) -> u32 { self.count }
    
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevMode {
    Forward, 
    Brake, 
    Backward, 
}

/// トリガ
/// 
/// 入力した場合、入力が解除されるまで常に入力状態を維持します。
/// また、入力をカウントすることが可能です。
pub struct Trigger {
    triggering: bool, 
    count: u32, 
}
impl Default for Trigger {
    fn default() -> Self {
        Self { 
            triggering: false, 
            count: 0, 
        }
    }
}
impl Trigger {
    pub fn trigger(&mut self, state: winit::event::ElementState) { match state {
        winit::event::ElementState::Pressed => self.triggering = true, 
        winit::event::ElementState::Released => self.triggering = false, 
    }}
    pub fn update(&mut self) {if self.triggering { 
        self.count = self.count.checked_add(1)
            .map_or_else(|| u32::MAX, |c| c);
    } else { 
        self.count = 0 
    }}

    /// トリガされているか
    pub fn is_triggered(&self) -> bool { self.triggering }

    /// 入力カウントの取得
    pub fn get_trig_count(&self) -> u32 { self.count }
}

/// ラッチ
/// 一度入力したら次の入力まで常にオンです。
/// オンの場合でも、入力しても、次の入力まで常にオフです。
pub struct Latch {
    latch_on: bool, 
    latch_inputting: bool, 
    count: u32, 
    on_count: u32, 
    off_count: u32, 
}
impl Default for Latch {
    fn default() -> Self {
        Self { 
            latch_on: false, 
            latch_inputting: false, 
            count: 0, 
            on_count: 0, 
            off_count: 0,  
        }
    }
}
impl Latch {
    pub fn trigger(&mut self, state: winit::event::ElementState) { match state {
        winit::event::ElementState::Pressed => {
            self.latch_inputting = true;
            if self.count == 0 && !self.latch_on && self.on_count == 0 { 
                self.latch_on = true 
            } else if self.count == 0 && self.latch_on && self.off_count == 0 { 
                self.latch_on = false 
            }
        }, 
        winit::event::ElementState::Released => {
            self.latch_inputting = false;
        }
    } }

    /// カウンタの更新
    pub fn update(&mut self) {
        if self.latch_inputting {
            self.count = self.count.checked_add(1)
                .map_or_else(|| u32::MAX, |c| c);
        } else {
            self.count = 0;
        }
        if self.latch_on {
            self.on_count = self.on_count.checked_add(1)
                .map_or_else(|| u32::MAX, |c| c);
            self.off_count = 0;
        } else {
            self.off_count = self.off_count.checked_add(1)
                .map_or_else(|| u32::MAX, |c| c);
            self.on_count = 0;
        }
    }

    /// ラッチオンか
    pub fn is_latch_on(&self) -> bool { self.latch_on }

    /// ラッチオン状態のカウンタ取得
    pub fn latch_on_count(&self) -> u32 { self.on_count }

    /// ラッチオフ状態のカウンタ取得
    pub fn latch_off_count(&self) -> u32 { self.off_count }
}