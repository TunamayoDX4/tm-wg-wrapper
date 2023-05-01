/// 物理演算に使うためのボディ
pub trait PhysicBody {
    fn position(&self) -> nalgebra::Point2<f32>;
    fn size(&self) -> nalgebra::Vector2<f32>;
    fn rotation(&self) -> f32;
    fn velocity(&self) -> nalgebra::Vector2<f32>;
}

/// 物理演算に使う正方形の剛体のボディ
pub trait PhysicRigidSquare {
    fn position(&self) -> nalgebra::Point2<f32>;
    fn size(&self) -> nalgebra::Vector2<f32>;
}

/// 未来位置の計算
pub fn deviation_pos(
    my: &impl PhysicBody, 
    target: &impl PhysicBody, 
    vel: f32, 
) -> nalgebra::Point2<f32> {
    let dist = target.position() - my.position();
    let dist_s = (dist.x.powi(2) + dist.y.powi(2)).sqrt();
    let dur = dist_s / vel;
    target.position() + target.velocity() * dur
}

/// AABBによる当たり判定
pub fn aabb(
    a: &impl PhysicBody, 
    b: &impl PhysicBody, 
) -> bool {
    let dist = (b.position() - a.position()).abs();
    let size_sum = ((a.size() + b.size()) * 0.5).abs();
    dist < size_sum
}