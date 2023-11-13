use super::*;
use instance::*;
use std::collections::VecDeque;

/// シーン機能のスタック
pub struct SceneStack<S: Scene> {
    ident: SceneIdentMaster, 
    scenes: VecDeque<SceneHolder<S>>, 
    ops: Vec<Option<(SceneIdent, SceneStackCtrlOp<S>)>>, 
}
impl<S: Scene> SceneStack<S> {
    pub fn new(
        default_scene: impl IntoIterator<Item = S>, 
    ) -> Self {
        let mut ident = SceneIdentMaster::default();
        let scenes = default_scene.into_iter()
            .map(|s| SceneHolder {
                ident: ident.issue(),
                scene: s,
            })
            .collect::<VecDeque<SceneHolder<S>>>();
        let ops = Vec::new();
        Self {
            ident,
            scenes,
            ops,
        }
    }

    /// 処理
    pub fn process(
        &mut self, 
        frame_param: &mut S::Fpr, 
        gfx: &GfxCtx<S::Rdr>, 
        sfx: &SfxCtx, 
    ) -> Result<SceneFrameCtrlParam, Box<dyn std::error::Error>> {
        let top = self.scenes.len() - 1;
        self.scenes.iter_mut()
            .enumerate()
            .map(|(depth, s)| 
                s.process(
                    top - depth, 
                    depth == top, 
                    frame_param, 
                    gfx, 
                    sfx, 
                )
                    .map(|sccp| if let SceneProcOp::StkCtl(
                        sc
                    ) = sccp.1 {
                        Some((sccp.0, sc))
                    } else { None })
            )
            .filter_map(|sc| match sc {
                Ok(None) => None, 
                v @ _ => Some(v)
            })
            .fold(
                Ok::<(), Box<dyn std::error::Error>>(()), 
                |
                    init, 
                    spr, 
                | {
                    init?;
                    self.ops.push(spr?);
                    Ok(())
                }
            )?;

        // 命令の処理
        let cp = self.ops.iter_mut()
            .filter_map(|op| op.take())
            .fold(
                SceneFrameCtrlParam::Continue, 
                |
                    init, 
                    (_ident, op), 
                | if let SceneFrameCtrlParam::Continue = init { match op {
                    SceneStackCtrlOp::Push(scene) => {
                        self.scenes.push_back(SceneHolder {
                            scene, 
                            ident: self.ident.issue(), 
                        });
                        SceneFrameCtrlParam::Continue
                    },
                    SceneStackCtrlOp::Pop => {
                        self.scenes.pop_back().map(|s| (
                            self.scenes.back_mut().map(|fg| (s.scene.pop(), fg))
                        ))
                            .flatten()
                            .map(|(
                                popv, 
                                fg
                            )| fg.scene.return_foreground(popv));
                        SceneFrameCtrlParam::Continue
                    },
                    SceneStackCtrlOp::PopAll(scene) => {
                        self.scenes.clear();
                        self.scenes.push_back(SceneHolder {
                            scene, 
                            ident: self.ident.issue(), 
                        });
                        SceneFrameCtrlParam::Continue
                    },
                    SceneStackCtrlOp::Exit => SceneFrameCtrlParam::Exit(0),
                } 
            } else { SceneFrameCtrlParam::Exit(0) });

        // 命令キューの削除
        self.ops.clear();

        // 終了処理
        Ok(match cp {
            SceneFrameCtrlParam::Continue if self.scenes.is_empty() => {
                SceneFrameCtrlParam::Exit(0)
            }, 
            v @ _ => v, 
        })
    }

    /// 描画処理
    pub fn rendering<'a>(
        &mut self, 
        render_chain: RenderingChain<'a, S::Rdr>, 
        frame_param: &S::Fpr, 
    ) -> RenderingChain<'a, S::Rdr> {
        let top = self.scenes.len() - 1;
        let mut render_chain = Some(render_chain);
        for (idx, scene) in self.scenes.iter_mut()
            .enumerate()
            .filter(|(idx, s)| s.scene.require_rendering(
                top - *idx, 
                top == *idx
            ))
        { render_chain = Some(scene.scene.rendering(
            render_chain.take().unwrap(), 
            top - idx, 
            top == idx, 
            frame_param
        ))}
        render_chain.unwrap()
    }

    /// キー入力
    pub fn input_key(
        &mut self, 
        keycode: VirtualKeyCode, 
        state: ElementState, 
    ) {
        self.scenes.back_mut()
            .map(|s| s.scene.input_key(keycode, state));
    }

    /// マウス入力
    pub fn input_mouse_button(
        &mut self, 
        button: MouseButton, 
        state: ElementState, 
    ) {
        self.scenes.back_mut()
            .map(|s| s.scene.input_mouse_button(button, state));
    }

    /// マウス動作入力
    pub fn input_mouse_motion(
        &mut self, 
        delta: (f64, f64)
    ) {
        self.scenes.back_mut()
            .map(|s| s.scene.input_mouse_motion(delta));
    }

    /// マウススクロール入力
    pub fn input_mouse_scroll(
        &mut self, 
        delta: MouseScrollDelta, 
    ) {
        self.scenes.back_mut()
            .map(|s| s.scene.input_mouse_scroll(delta));
    }

    /// ウィンドウのリサイズ
    pub fn window_resizing(
        &mut self, 
        size: winit::dpi::PhysicalSize<u32>, 
    ) {
        self.scenes.iter_mut()
            .for_each(|s| s.scene.window_resizing(size));
    }
}