use crate::store_jukebox::{CENTER,SPEAKERS};

pub fn mix(listener: [f32;3], right: [f32;3], active: bool) -> [f32;2] {
    if !active { return [0.0;2]; }
    let mut sources=Vec::new();
    for source in SPEAKERS.into_iter().chain(std::iter::once(CENTER)) {
        let d: [f32;3]=std::array::from_fn(|i| source[i]-listener[i]);
        let distance=d.iter().map(|x| x*x).sum::<f32>().sqrt();
        let gain=1.0/(1.0+(distance/7.5).powi(2));
        let pan=d.iter().zip(right).map(|(d,r)| d*r).sum::<f32>()/distance.max(0.01);
        sources.push((gain,pan.clamp(-1.0,1.0)));
    }
    sources.sort_by(|a,b| b.0.total_cmp(&a.0));
    let weight=sources.iter().take(3).map(|s| s.0*s.0).sum::<f32>();
    let pan=sources.iter().take(3).map(|s| s.1*s.0*s.0).sum::<f32>()/weight.max(0.001);
    let level=0.20+0.80*sources[0].0;
    [level*(1.0-0.65*pan.max(0.0)),level*(1.0+0.65*pan.min(0.0))]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proximity_grows_near_sources_and_head_rotation_swaps_channels() {
        let near=mix([7.8,-0.2,2.3],[1.0,0.0,0.0],true);
        let far=mix([-15.0,-0.2,6.0],[1.0,0.0,0.0],true);
        assert!(near[0].max(near[1])>far[0].max(far[1]));
        let a=mix([0.0,0.15,3.0],[1.0,0.0,0.0],true);
        let b=mix([0.0,0.15,3.0],[-1.0,0.0,0.0],true);
        assert!((a[0]-b[1]).abs()<0.0001 && (a[1]-b[0]).abs()<0.0001);
        assert_eq!(mix(CENTER,[1.0,0.0,0.0],false),[0.0;2]);
    }
}
