use crate::input::Controls;
use openxr as xr;

pub struct StoreNavigation {
    pub pose: xr::Posef,
    yaw: f32,
    grab_mode: bool,
    anchors: [Option<[f32; 3]>; 2],
    velocity: [f32; 2],
}
impl Default for StoreNavigation {
    fn default() -> Self { Self { pose: xr::Posef::IDENTITY, yaw: 0.0,
        grab_mode: false, anchors: [None; 2], velocity: [0.0; 2] } }
}
impl StoreNavigation {
    pub fn reset(&mut self) { self.anchors = [None; 2]; self.velocity = [0.0; 2]; }

    pub fn update(&mut self, controls: Controls, head: xr::Posef,
        hands: [Option<xr::Posef>; 2], dt: f32) {
        let dt = dt.clamp(0.0, 0.04);
        if controls.x {
            self.grab_mode = !self.grab_mode; self.reset();
            eprintln!("Store movement: {}", if self.grab_mode { "glide" } else { "walk" });
        }
        let turn = analog(controls.sticks[1][0],0.15);
        if turn != 0.0 {
            let local = self.inverse_ray(point(head),[0.0;3]).0;
            self.yaw -= turn*std::f32::consts::FRAC_PI_2*dt;
            self.pose.orientation = xr::Quaternionf { x:0.0,y:(self.yaw/2.0).sin(),z:0.0,w:(self.yaw/2.0).cos() };
            self.place_head(point(head),local);
            self.anchors = [None;2];
        }
        let local = self.inverse_ray(point(head),[0.0;3]).0;
        let mut target = local;
        let mut pulls = Vec::new();
        for (i, hand) in hands.iter().enumerate() {
            let Some(hand) = hand else {
                if self.anchors[i].is_some() { self.velocity = [0.0;2]; }
                self.anchors[i] = None; continue;
            };
            if !controls.grips[i] { self.anchors[i] = None; continue; }
            let p = self.inverse_ray(point(*hand),[0.0;3]).0;
            let anchor = *self.anchors[i].get_or_insert(p);
            let delta = [anchor[0]-p[0],anchor[2]-p[2]];
            if delta[0].hypot(delta[1]) > 0.8 { self.anchors[i] = None; self.velocity = [0.0;2]; continue; }
            pulls.push(delta);
        }
        if !pulls.is_empty() {
            for p in &pulls { target[0] += p[0]/pulls.len() as f32; target[2] += p[1]/pulls.len() as f32; }
        } else {
            let [x,y] = controls.sticks[0];
            let magnitude = x.hypot(y);
            let q = hands[0].unwrap_or(head).orientation;
            let mut forward = [-2.0*(q.x*q.z+q.w*q.y),-1.0+2.0*(q.x*q.x+q.y*q.y)];
            if forward[0].hypot(forward[1]) < 0.25 {
                let q = head.orientation;
                forward = [-2.0*(q.x*q.z+q.w*q.y),-1.0+2.0*(q.x*q.x+q.y*q.y)];
            }
            let length = forward[0].hypot(forward[1]).max(0.001);
            let top_speed = if controls.sprint { 3.8 } else if self.grab_mode { 3.0 } else { 1.9 };
            let speed = ((magnitude-0.12)/0.88).clamp(0.0,1.0)*top_speed;
            let stick = if magnitude > 0.12 { [x/magnitude,y/magnitude] } else { [0.0;2] };
            let world = [(forward[0]*stick[1]-forward[1]*stick[0])*speed/length,
                (forward[1]*stick[1]+forward[0]*stick[0])*speed/length];
            let d = self.inverse_ray([0.0;3],[world[0],0.0,world[1]]).1;
            if magnitude > 0.12 || !self.grab_mode {
                let response = 1.0-(-20.0*dt).exp();
                self.velocity[0] += (d[0]-self.velocity[0])*response;
                self.velocity[1] += (d[2]-self.velocity[1])*response;
            } else {
                let drag = (-1.3*dt).exp(); self.velocity[0] *= drag; self.velocity[1] *= drag;
            }
            target[0] += self.velocity[0]*dt; target[2] += self.velocity[1]*dt;
        }
        let delta = [target[0]-local[0],target[2]-local[2]];
        let factor = (0.25/delta[0].hypot(delta[1]).max(0.00001)).min(1.0);
        target[0] = local[0]+delta[0]*factor; target[2] = local[2]+delta[1]*factor;
        let accepted = crate::store_collision::slide(local,target);
        if !pulls.is_empty() && dt > 0.0 {
            let velocity = [(accepted[0]-local[0])/dt,(accepted[2]-local[2])/dt];
            let response = 1.0-(-25.0*dt).exp();
            for i in 0..2 { self.velocity[i] += (velocity[i]-self.velocity[i])*response; }
            let cap = (3.0/self.velocity[0].hypot(self.velocity[1]).max(0.0001)).min(1.0);
            self.velocity[0] *= cap; self.velocity[1] *= cap;
        }
        for (axis,i) in [(0,0),(2,1)] {
            if (accepted[axis]-target[axis]).abs()>0.0001 { self.velocity[i] = 0.0; }
        }
        self.place_head(point(head),accepted);
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
fn analog(x: f32, dead: f32) -> f32 { x.signum()*((x.abs()-dead)/(1.0-dead)).clamp(0.0,1.0) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn movement_is_frame_rate_independent_and_smooth_turn_pivots_at_head() {
        let mut c = Controls::default(); c.sticks[0][1] = 1.0;
        let mut a = StoreNavigation::default(); let mut b = StoreNavigation::default();
        for _ in 0..72 { a.update(c,xr::Posef::IDENTITY,[None;2],1.0/72.0); }
        for _ in 0..144 { b.update(c,xr::Posef::IDENTITY,[None;2],1.0/144.0); }
        assert!((a.pose.position.z-b.pose.position.z).abs()<0.012);
        let mut head = xr::Posef::IDENTITY; head.position.x = 0.6;
        let before = a.inverse_ray(point(head),[0.0;3]).0;
        c = Controls::default(); c.sticks[1][0] = 1.0;
        a.reset(); a.update(c,head,[None;2],0.01);
        let after = a.inverse_ray(point(head),[0.0;3]).0;
        assert!((before[0]-after[0]).abs()<0.0001 && (before[2]-after[2]).abs()<0.0001);
    }
    #[test]
    fn free_grip_tracks_hand_motion_and_tracking_loss_releases_anchor() {
        let mut n = StoreNavigation::default(); let mut c = Controls::default(); c.grips[0] = true;
        let mut hand = xr::Posef::IDENTITY;
        n.update(c,xr::Posef::IDENTITY,[Some(hand),None],0.01);
        assert!(n.anchors[0].is_some());
        hand.position.z = 0.1;
        n.update(c,xr::Posef::IDENTITY,[Some(hand),None],0.01);
        assert!((n.pose.position.z-0.1).abs()<0.001);
        n.update(c,xr::Posef::IDENTITY,[None;2],0.01);
        assert!(n.anchors[0].is_none()); assert_eq!(n.velocity,[0.0;2]);
    }
    #[test]
    fn lowered_controller_does_not_disable_walking() {
        let mut n = StoreNavigation::default(); let mut c = Controls::default(); c.sticks[0][1] = 1.0;
        let mut hand = xr::Posef::IDENTITY;
        hand.orientation = xr::Quaternionf { x:0.5f32.sqrt(),y:0.0,z:0.0,w:0.5f32.sqrt() };
        for _ in 0..72 { n.update(c,xr::Posef::IDENTITY,[Some(hand),None],1.0/72.0); }
        assert!(n.pose.position.z > 1.7);
    }
    #[test]
    fn held_sprint_is_faster_and_still_stops_at_shelves() {
        let mut walk = StoreNavigation::default(); let mut run = StoreNavigation::default();
        let mut c = Controls::default(); c.sticks[0][1] = 1.0;
        for _ in 0..72 {
            walk.update(c,xr::Posef::IDENTITY,[None;2],1.0/72.0);
            c.sprint = true; run.update(c,xr::Posef::IDENTITY,[None;2],1.0/72.0); c.sprint = false;
        }
        assert!(run.pose.position.z > walk.pose.position.z*1.8);
        c.sprint = true;
        for _ in 0..144 { run.update(c,xr::Posef::IDENTITY,[None;2],1.0/72.0); }
        let head = run.inverse_ray([0.0;3],[0.0;3]).0;
        assert!(crate::store_layout::free(head,0.20));
        assert!(head[2] > -4.21);
    }
}
