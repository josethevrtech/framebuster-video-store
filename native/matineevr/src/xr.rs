use crate::{
    app::App, graphics::Graphics, input::Input, options::Options, overlays::Overlays,
    performance::Performance, xr_draw::Video,
};
use anyhow::Result;
use matineevr::statistics::{Statistics, timed};
use openxr as xr;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub fn run(options: &Options) -> Result<()> {
    let instance = Graphics::instance()?;
    eprintln!("OpenXR: {}", instance.properties()?.runtime_name);
    let system = instance.system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)?;
    let graphics = Graphics::new(&instance, system)?;
    let (session, mut waiter, mut stream) = graphics.session(&instance, system)?;
    let mut input = Input::new(&instance, &session, Overlays::hand_offset())?;
    let space =
        session.create_reference_space(xr::ReferenceSpaceType::LOCAL, xr::Posef::IDENTITY)?;
    let view_config = xr::ViewConfigurationType::PRIMARY_STEREO;
    let mut app = App::new(options);
    let mut video = Video::new(graphics.clone(), &instance, system, &session)?;
    video.renderer.stats = options.stats.then(Statistics::default);
    let mut overlays = Overlays::new(&session, graphics.clone())?;
    let controllers = crate::controller_visuals::ControllerVisuals::new(&session, graphics.clone())?;
    let mut performance = Performance::new(options.stats);
    let mut hud_snapshot = options.hud_snapshot.clone();
    let mut events = xr::EventDataBuffer::new();
    let mut space_changes = Vec::new();
    let mut running = false;
    let mut focused = false;
    let mut visible = false;
    let mut yaw = 0.0;
    let mut rendered = 0;
    let mut video_ready = false;
    let started = Instant::now();
    'run: loop {
        performance.report(&mut video.renderer, false);
        while let Some(event) = instance.poll_event(&mut events)? {
            match event {
                xr::Event::SessionStateChanged(event) => {
                    overlays.reset_hands();
                    app.reset_seek_repeat();
                    eprintln!("XR state: {:?}", event.state());
                    focused = event.state() == xr::SessionState::FOCUSED;
                    visible = focused || event.state() == xr::SessionState::VISIBLE;
                    if let Some(player) = &mut app.playback {
                        player.set_suspended(!visible);
                    }
                    match event.state() {
                        xr::SessionState::READY => {
                            session.begin(view_config)?;
                            running = true;
                        }
                        xr::SessionState::STOPPING => {
                            session.end()?;
                            running = false;
                        }
                        xr::SessionState::EXITING | xr::SessionState::LOSS_PENDING => break 'run,
                        _ => {}
                    }
                }
                xr::Event::InstanceLossPending(_) => break 'run,
                xr::Event::ReferenceSpaceChangePending(event)
                    if event.reference_space_type() == xr::ReferenceSpaceType::LOCAL =>
                {
                    space_changes.push(event.change_time());
                }
                _ => {}
            }
        }
        if options
            .seconds
            .is_some_and(|limit| started.elapsed().as_secs_f64() >= limit)
        {
            break;
        }
        if !running {
            std::thread::sleep(Duration::from_millis(20));
            continue;
        }
        let timing = performance.wait(&mut waiter, &mut stream, focused, visible)?;
        space_changes.retain(|time| {
            let pending = time.as_nanos() > timing.predicted_display_time.as_nanos();
            if !pending {
                overlays.reset_hands();
            }
            pending
        });
        if !timing.should_render || !visible {
            overlays.reset_hands();
            performance.suspend();
            performance.empty(&mut stream, timing.predicted_display_time)?;
            continue;
        }
        let work_started = Instant::now();
        let controls = if focused {
            input.poll(&session, &space, timing.predicted_display_time)?
        } else {
            Default::default()
        };
        app.update(controls);
        if options.stats
            && let Some(player) = &mut app.playback
        {
            player.diagnostics();
        }
        app.synchronize(&mut [&mut video.renderer, &mut overlays.thumbnail.renderer])?;
        let frame = app.frame().map(Rc::new);
        if let Err(error) = overlays.thumbnail.update(&mut app, frame.as_ref()) {
            app.fail(error);
        }
        let mut uploaded = false;
        if let Some(frame) = frame {
            match video.renderer.upload(frame.clone()) {
                Ok(()) => {
                    app.video_uploaded();
                    performance.size = (frame.pixels.width, frame.pixels.height);
                    video_ready = true;
                    uploaded = true;
                }
                Err(error) => app.fail(error),
            }
        }
        if app.playback.is_none() {
            app.synchronize(&mut [&mut video.renderer, &mut overlays.thumbnail.renderer])?;
            video_ready = false;
            performance.size = (0, 0);
        }
        let (state, views) =
            session.locate_views(view_config, timing.predicted_display_time, &space)?;
        if !state.contains(xr::ViewStateFlags::ORIENTATION_VALID) {
            overlays.reset_hands();
            performance.empty(&mut stream, timing.predicted_display_time)?;
            continue;
        }
        if controls.a && app.playback.is_some() && !app.adjustment.blocked() {
            let q = views[0].pose.orientation;
            yaw = (2.0 * (q.w * q.y + q.x * q.z)).atan2(1.0 - 2.0 * (q.x * q.x + q.y * q.y));
        }
        let facing = views[0].pose.orientation;
        video.draw(views, app.presentation, yaw, video_ready, options)?;
        let overlay_started = options.stats.then(Instant::now);
        overlays.update(&session, &graphics, &app, &performance, &mut hud_snapshot)?;
        if let Some(stats) = &mut performance.stats {
            stats.sample(
                "overlays_ms",
                overlay_started.unwrap().elapsed().as_secs_f64() * 1000.0,
            );
        }
        let mut quads = overlays.layers(&space, input.panel_poses, timing.predicted_display_time);
        if focused {
            quads.extend(controllers.layers(&space, input.controller_poses, facing));
        }
        timed(performance.stats.as_mut(), "xr_end_ms", || {
            crate::swapchains::submit(
                &mut stream,
                timing.predicted_display_time,
                &space,
                &video.eyes,
                &video.views,
                &quads,
            )
        })?;
        rendered += 1;
        performance.record(
            uploaded,
            timing.predicted_display_period.as_nanos(),
            work_started.elapsed(),
            !quads.is_empty(),
        );
        overlays.submitted(options.stats);
    }
    app.stop();
    app.synchronize(&mut [&mut video.renderer, &mut overlays.thumbnail.renderer])?;
    performance.report(&mut video.renderer, true);
    eprintln!("Submitted {rendered} stereo frames");
    Ok(())
}
