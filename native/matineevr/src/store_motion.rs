use crate::{input::Controls, store_layout};
use openxr as xr;

pub struct StoreNavigation {
    pub pose: xr::Posef,
    yaw: f32,
    turn_held: bool,
    grab_mode: bool,
    anchors: [Option<[f32; 3]>; 2],
    velocity: [f32; 2],
}
impl Default for StoreNavigation {
    fn default() -> Self { Self { pose: xr::Posef::IDENTITY, yaw: 0.0, turn_held: false,
        grab_mode: false, anchors: [None; 2], velocity: [0.0; 2] } }
}
impl StoreNavigation {
    pub fn reset(&mut self) { self.anchors = [None; 2]; self.velocity = [0.0; 2]; }

    pub fn update(&mut self, controls: Controls, head: xr::Posef,
        hands: [Option<xr::Posef>; 2], dt: f32) {
        let dt = dt.clamp(0.0, 0.04);
        if controls.x {
            self.grab_mode = !self.grab_mode; self.reset();
            eprintln!("Store movement: {}", if self.grab_mode { "surface grip" } else { "smooth stick" });
        }
        let pressed = controls.sticks[1][0].abs() > 0.7;
        if pressed && !self.turn_held {
            let local = self.inverse_ray(point(head), [0.0; 3]).0;
            self.yaw -= controls.sticks[1][0].signum() * std::f32::consts::PI / 6.0;
            self.pose.orientation = xr::Quaternionf { x: 0.0, y: (self.yaw/2.0).sin(), z: 0.0, w: (self.yaw/2.0).cos() };
            self.place_head(point(head), local); self.reset();
        }
        self.turn_held = pressed;
        let local = self.inverse_ray(point(head), [0.0; 3]).0;
        let mut target = local;
        if self.grab_mode {
            let mut pulls = Vec::new();
            for (i, hand) in hands.iter().enumerate() {
                let Some(hand) = hand else { self.anchors[i] = None; continue; };
                let p = self.inverse_ray(point(*hand), [0.0; 3]).0;
                if !controls.grips[i] { self.anchors[i] = None; continue; }
                if self.anchors[i].is_none() && store_layout::surface(p) { self.anchors[i] = Some(p); }
                if let Some(a) = self.anchors[i] { pulls.push([a[0]-p[0], a[2]-p[2]]); }
            }
            if !pulls.is_empty() {
                for p in &pulls { target[0] += p[0]/pulls.len() as f32; target[2] += p[1]/pulls.len() as f32; }
            } else {
                target[0] += self.velocity[0]*dt; target[2] += self.velocity[1]*dt;
                let drag = (-4.0*dt).exp();
                self.velocity[0] *= drag; self.velocity[1] *= drag;
            }
        } else {
            let [x,y] = controls.sticks[0];
            let magnitude = x.hypot(y);
            let q = head.orientation;
            let forward = [-2.0*(q.x*q.z+q.w*q.y), -1.0+2.0*(q.x*q.x+q.y*q.y)];
            let length = forward[0].hypot(forward[1]).max(0.001);
            let speed = ((magnitude-0.18)/0.82).clamp(0.0,1.0)*1.4;
            let stick = if magnitude > 0.18 { [x/magnitude,y/magnitude] } else { [0.0;2] };
            let world = [(forward[0]*stick[1]-forward[1]*stick[0])*speed/length,
                (forward[1]*stick[1]+forward[0]*stick[0])*speed/length];
            let d = self.inverse_ray([0.0;3],[world[0],0.0,world[1]]).1;
            let response = 1.0-(-12.0*dt).exp();
            self.velocity[0] += (d[0]-self.velocity[0])*response;
            self.velocity[1] += (d[2]-self.velocity[1])*response;
            target[0] += self.velocity[0]*dt; target[2] += self.velocity[1]*dt;
        }
        let max = if self.grab_mode { 2.0 } else { 1.4 } * dt;
        let delta = [target[0]-local[0],target[2]-local[2]];
        let factor = (max/delta[0].hypot(delta[1]).max(0.00001)).min(1.0);
        target[0] = local[0]+delta[0]*factor; target[2] = local[2]+delta[1]*factor;
        let target = slide(local,target);
        if self.grab_mode && self.anchors.iter().any(Option::is_some) && dt > 0.0 {
            self.velocity = [(target[0]-local[0])/dt,(target[2]-local[2])/dt];
        }
        self.place_head(point(head),target);
    }

    fn place_head(&mut self, head: [f32;3], local: [f32;3]) {
        let (s,c) = self.yaw.sin_cos();
        self.pose.position.x = head[0]-c*local[0]-s*local[2];
        self.pose.position.z = head[2]+s*local[0]-c*local[2];
    }
    pub fn inverse_ray(&self, p: [f32;3], d: [f32;3]) -> ([f32;3],[f32;3]) {
        let (s,c) = self.yaw.sin_cos();
        let x = p[0]-self.pose.position.x; let z = p[2]-self.pose.position.z;
        ([c*x-s*z,p[1]-self.pose.position.y,s*x+c*z],[c*d[0]-s*d[2],d[1],s*d[0]+c*d[2]])
    }
}
fn point(p: xr::Posef) -> [f32;3] { [p.position.x,p.position.y,p.position.z] }
fn slide(start: [f32;3], target: [f32;3]) -> [f32;3] {
    let mut p = start;
    for axis in [0,2] {
        let mut candidate = p; candidate[axis] = target[axis];
        if store_layout::free(candidate,0.22) { p = candidate; }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn movement_is_frame_rate_independent_and_snap_turn_pivots_at_head() {
        let mut c = Controls::default(); c.sticks[0][1] = 1.0;
        let mut a = StoreNavigation::default(); let mut b = StoreNavigation::default();
        for _ in 0..72 { a.update(c,xr::Posef::IDENTITY,[None;2],1.0/72.0); }
        for _ in 0..144 { b.update(c,xr::Posef::IDENTITY,[None;2],1.0/144.0); }
        assert!((a.pose.position.z-b.pose.position.z).abs()<0.012);
        let mut head = xr::Posef::IDENTITY; head.position.x = 0.6;
        let before = a.inverse_ray(point(head),[0.0;3]).0;
        c = Controls::default(); c.sticks[1][0] = 1.0;
        a.update(c,head,[None;2],0.0);
        let after = a.inverse_ray(point(head),[0.0;3]).0;
        assert!((before[0]-after[0]).abs()<0.0001 && (before[2]-after[2]).abs()<0.0001);
    }
    #[test]
    fn grip_requires_a_surface_and_tracking_loss_releases_anchor() {
        let mut n = StoreNavigation::default(); let mut c = Controls::default();
        c.x = true; n.update(c,xr::Posef::IDENTITY,[None;2],0.01); c.x = false; c.grips[0] = true;
        n.update(c,xr::Posef::IDENTITY,[Some(xr::Posef::IDENTITY),None],0.01);
        assert!(n.anchors[0].is_none());
        let mut hand = xr::Posef::IDENTITY;
        hand.position = xr::Vector3f { x:-3.2,y:-0.4,z:4.82 };
        n.update(c,xr::Posef::IDENTITY,[Some(hand),None],0.01); assert!(n.anchors[0].is_some());
        n.update(c,xr::Posef::IDENTITY,[None;2],0.01); assert!(n.anchors[0].is_none());
    }
}
