use super::*;

#[derive(Default)]
pub struct SceneIdentMaster(u64);
impl SceneIdentMaster {
    pub fn issue(&mut self) -> SceneIdent {
        let r = SceneIdent(self.0);
        self.0 = self.0.checked_add(1).unwrap_or(0);
        r
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SceneIdent(u64);
impl std::borrow::Borrow<u64> for SceneIdent {
    fn borrow(&self) -> &u64 { &self.0 }
}
impl PartialEq<u64> for SceneIdent {
    fn eq(&self, other: &u64) -> bool {
        self.0.eq(other)
    }
}

pub struct SceneHolder<S: Scene> {
    pub(super) ident: SceneIdent, 
    pub(super) scene: S, 
}
impl<S: Scene> SceneHolder<S> {
    pub fn process(
        &mut self, 
        depth: usize, 
        is_top: bool, 
        renderer: &S::Rdr, 
        frame_param: &mut S::Fpr, 
        window: &Window, 
        gfx: &GfxCtx, 
        sfx: &SfxCtx, 
    ) -> Result<
        (SceneIdent, SceneProcOp<S>), Box<dyn std::error::Error>
    > {
        self.scene.process(
            depth, 
            is_top, 
            renderer, 
            frame_param, 
            window, 
            gfx, 
            sfx, 
        ).map(|op| (self.ident.clone(), op))
    }
}