pub fn ceiling(p: [f32;3]) -> ([f32;3],[f32;2]) {
    ([p[0]-0.0026,1.3181,p[2]+0.1762],[0.416,0.325])
}
#[cfg(test)]
mod tests {
    #[test]
    fn video_surface_sits_on_the_models_positive_z_glass_face() {
        for p in crate::store_arcade::CEILING_TVS {
            let (center,size)=super::ceiling(p);
            let glass_front=p[2]+0.015+0.121903*1.3;
            assert!(center[2]>glass_front && center[2]-glass_front<0.003);
            assert!(center[1]-size[1]/2.0>1.01+0.09*1.3);
            assert!(center[1]+size[1]/2.0<1.01+0.389*1.3);
        }
    }
}
