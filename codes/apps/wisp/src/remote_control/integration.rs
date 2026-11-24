// Included inside compositor so the development interface can use its private routing.
impl WispCompositorState {
    fn remote_status(&self, id: u64) -> crate::remote_control::protocol::Reply {
        let mut reply = crate::remote_control::protocol::Reply::ok(id);
        reply.json = serde_json::json!({"id":id,"ok":true,"version":1,
            "width":self.phys_size[0],"height":self.phys_size[1],"scale":self.scale,
            "logical_width":self.phys_size[0] as f64/self.scale as f64,
            "logical_height":self.phys_size[1] as f64/self.scale as f64,"locked":self.shell.lockscreen.locked,"screen_on":self.screen_on,"unlock_prompt":self.shell.lockscreen.bouncer_open,"indicator":"device_only","tap_feedback":true,"stream_codec":"h264","stream_codecs":["h264","png"]});
        reply
    }

    fn remote_cleanup(&mut self, remote: &mut crate::remote_control::RemoteControl) {
        remote.revoke();
        if !remote.input.contacts.is_empty() {
            self.handle_touch_cancel();
        }
        for code in remote.input.keys.drain() {
            self.remote_key(code, false);
        }
        remote.input.contacts.clear();
        remote.pointers.clear();
        remote.taps.clear();
        for (id, reply) in remote.pending.drain(..) {
            let _ = reply.try_send(crate::remote_control::protocol::Reply::error(
                id,
                "session ended",
            ));
        }
        remote.session = None;
        remote.alive = None;
        self.needs_render = true;
    }

    fn remote_local_takeover(&mut self) -> bool {
        let mut took_over = false;
        if let Some(mut remote) = self.remote_control.take() {
            if remote.session.is_some() {
                took_over = true;
                tracing::info!("remote_control ended by local input");
                self.remote_cleanup(&mut remote);
            }
            self.remote_control = Some(remote);
        }
        took_over
    }

    fn poll_remote_control(&mut self) {
        use crate::remote_control::{
            protocol::{Command, Reply},
            server::Event,
        };
        let Some(mut remote) = self.remote_control.take() else {
            return;
        };
        if remote.session.is_some() && !remote.active() {
            self.remote_cleanup(&mut remote);
        }
        let had_taps = !remote.taps.is_empty();
        remote
            .taps
            .retain(|(_, when)| when.elapsed() < Duration::from_millis(700));
        if had_taps {
            self.needs_render = true;
        }
        for _ in 0..16 {
            let Ok(event) = remote.incoming.try_recv() else {
                break;
            };
            match event {
                Event::Begin {
                    session,
                    alive,
                    reply,
                    id,
                } => {
                    if !alive.load(std::sync::atomic::Ordering::SeqCst) {
                        continue;
                    }
                    if remote.session.is_some() {
                        self.remote_cleanup(&mut remote);
                    }
                    remote.session = Some(session);
                    remote.alive = Some(alive);
                    self.needs_render = true;
                    tracing::info!("remote_control connected (visible banner enabled)");
                    if reply.try_send(self.remote_status(id)).is_err() {
                        self.remote_cleanup(&mut remote);
                    }
                }
                Event::Request {
                    session,
                    request,
                    reply,
                } => {
                    let id = request.id;
                    if remote.session != Some(session) || !remote.active() {
                        let _ = reply.try_send(Reply::error(id, "session ended"));
                        continue;
                    }
                    let command = request.command;
                    let is_status = matches!(command, Command::Status);
                    let is_input = matches!(
                        &command,
                        Command::TouchDown { .. }
                            | Command::TouchMove { .. }
                            | Command::TouchUp { .. }
                            | Command::Key { .. }
                            | Command::Action { .. }
                    );
                    if is_input && self.screen_off_transition.is_some() {
                        let _ = reply.try_send(Reply::error(id, "screen is turning off"));
                        continue;
                    }
                    if is_input && !self.screen_on {
                        let _ = reply.try_send(Reply::error(id, "screen is off; wake first"));
                        continue;
                    }
                    if matches!(&command, Command::Action { .. }) && self.shell.lockscreen.locked {
                        let _ =
                            reply.try_send(Reply::error(id, "unlock before navigation actions"));
                        continue;
                    }
                    if let Err(e) = remote.input.validate(
                        &command,
                        self.phys_size[0] as f64 / self.scale as f64,
                        self.phys_size[1] as f64 / self.scale as f64,
                    ) {
                        let _ = reply.try_send(Reply::error(id, e));
                        continue;
                    }
                    let time = self.started.elapsed().as_millis() as u32;
                    let mut response = Reply::ok(id);
                    match command {
                        Command::Status => response = self.remote_status(id),
                        Command::Heartbeat => {
                            // An active controller is using the display. Explicit sleep
                            // still works; heartbeats never wake a sleeping phone.
                            if self.screen_on {
                                self.last_activity = Instant::now();
                            }
                        }
                        Command::Screenshot => {
                            if !self.screen_on {
                                response = Reply::error(id, "screen is off; wake first");
                            } else {
                                remote.pending.push((id, reply));
                                self.needs_render = true;
                                continue;
                            }
                        }
                        Command::TouchDown { slot, x, y } => {
                            remote.pointers.insert(slot, [x, y]);
                            self.handle_touch_down(100 + slot as i32, (x, y).into(), time)
                        }
                        Command::TouchMove { slot, x, y } => {
                            remote.pointers.insert(slot, [x, y]);
                            self.handle_touch_motion(100 + slot as i32, (x, y).into(), time)
                        }
                        Command::TouchUp { slot } => {
                            if let Some(point) = remote.pointers.remove(&slot) {
                                if remote.taps.len() >= 32 {
                                    remote.taps.remove(0);
                                }
                                remote.taps.push((point, Instant::now()));
                            }
                            self.handle_touch_up(100 + slot as i32, time)
                        }
                        Command::TouchCancel => {
                            remote.pointers.clear();
                            remote.taps.clear();
                            self.handle_touch_cancel()
                        }
                        Command::Key { code, down } => self.remote_key(code, down),
                        Command::Action { name } => {
                            self.shell.system_key(
                                match name.as_str() {
                                    "home" => 1,
                                    "overview" => 2,
                                    _ => 3,
                                },
                                self.started.elapsed().as_secs_f64(),
                            );
                            self.process_host_commands();
                            self.needs_render = true;
                        }
                        Command::Wake => {
                            if !self.screen_on {
                                self.wake_to_lockscreen();
                            }
                        }
                        Command::Sleep => {
                            if !remote.input.contacts.is_empty() {
                                self.handle_touch_cancel();
                                remote.input.contacts.clear();
                                remote.pointers.clear();
                                remote.taps.clear();
                            }
                            for code in remote.input.keys.drain() {
                                self.remote_key(code, false);
                            }
                            if self.screen_on {
                                self.sleep_and_lock("remote_control");
                            }
                        }
                        _ => response = Reply::error(id, "command is not valid here"),
                    }
                    if is_status {
                        response.json["contacts"] = serde_json::json!(remote.input.contacts.len());
                        response.json["keys"] = serde_json::json!(remote.input.keys.len());
                    }
                    if is_input {
                        self.last_activity = Instant::now();
                    }
                    let _ = reply.try_send(response);
                }
            }
        }
        self.remote_control = Some(remote);
    }

    fn remote_key(&mut self, code: u32, down: bool) {
        let forward = self.shell.is_wayland_passthrough();
        if let Some(keyboard) = self.seat.get_keyboard() {
            keyboard.input::<(), _>(
                self,
                (code + 8).into(),
                if down {
                    KeyState::Pressed
                } else {
                    KeyState::Released
                },
                SERIAL_COUNTER.next_serial(),
                self.started.elapsed().as_millis() as u32,
                |state, modifiers, symbol| {
                    // Always update XKB, including key releases after focus/navigation
                    // changes. This prevents stuck modifiers when a session ends.
                    if forward || !down {
                        return FilterResult::Forward;
                    }
                    use bhoo::Key;
                    let key = match code {
                        1 => Some(Key::Escape),
                        14 => Some(Key::Character('\u{8}')),
                        111 => Some(Key::Character('\u{7f}')),
                        15 => Some(Key::Tab),
                        28 => Some(Key::Enter),
                        57 => Some(Key::Space),
                        102 => Some(Key::Home),
                        103 => Some(Key::Up),
                        104 => Some(Key::PageUp),
                        105 => Some(Key::Left),
                        106 => Some(Key::Right),
                        107 => Some(Key::End),
                        108 => Some(Key::Down),
                        109 => Some(Key::PageDown),
                        127 => Some(Key::Menu),
                        115 => Some(Key::PageUp),
                        114 => Some(Key::PageDown),
                        _ => symbol
                            .modified_sym()
                            .key_char()
                            .filter(|c| !c.is_control())
                            .map(Key::Character),
                    };
                    if let Some(key) = key {
                        state.input.events.push_back(bhoo::Input {
                            key: Some(bhoo::KeyInput {
                                key,
                                pressed: true,
                                repeat: false,
                                shift: modifiers.shift,
                            }),
                            time: state.started.elapsed().as_secs_f64(),
                            ..Default::default()
                        });
                    }
                    FilterResult::Intercept(())
                },
            );
        }
        self.needs_render = true;
    }

    fn remote_render(&mut self, elements: &mut Vec<WispRenderElement>) {
        use crate::remote_control::{capture, protocol::Reply};
        let Some(mut remote) = self.remote_control.take() else {
            return;
        };
        if remote.active() {
            // Native Smithay texture elements keep touch markers above layer-shell
            // keyboards as well as above the shell and application surfaces.
            use smithay::backend::renderer::element::{
                Kind,
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            };
            let mut pixels = vec![0u8; 64 * 64 * 4];
            for y in 0..64 {
                for x in 0..64 {
                    let r = ((x as f64 - 31.5).powi(2) + (y as f64 - 31.5).powi(2)).sqrt();
                    let color = if r <= 30.0 && r >= 25.0 {
                        [0, 175, 255, 255]
                    } else if r < 25.0 {
                        [0, 48, 70, 70]
                    } else {
                        [0, 0, 0, 0]
                    };
                    pixels[(y * 64 + x) * 4..(y * 64 + x + 1) * 4].copy_from_slice(&color);
                }
            }
            for point in remote
                .pointers
                .values()
                .chain(remote.taps.iter().map(|(p, _)| p))
            {
                let buffer = MemoryRenderBuffer::from_slice(
                    &pixels,
                    Fourcc::Abgr8888,
                    (64, 64),
                    1,
                    Transform::Normal,
                    None,
                );
                let location = (
                    (point[0] - 12.0) * self.scale as f64,
                    (point[1] - 12.0) * self.scale as f64,
                );
                match MemoryRenderBufferRenderElement::from_buffer(
                    &mut self.renderer,
                    location,
                    &buffer,
                    None,
                    Some(Rectangle::from_size((64.0, 64.0).into())),
                    Some((24, 24).into()),
                    Kind::Unspecified,
                ) {
                    Ok(marker) => elements.insert(0, WispRenderElement::Memory(marker)),
                    Err(error) => tracing::warn!("remote_control touch marker: {error}"),
                }
            }
            if !remote.pending.is_empty() {
                // A single connection serializes capture, so there is normally one pending reply.
                match capture::capture(
                    &mut self.renderer,
                    elements,
                    self.phys_size,
                    self.scale as f64,
                ) {
                    Ok(pixels) => {
                        if let Some((id, reply)) = remote.pending.pop() {
                            let mut response = self.remote_status(id);
                            response.pixels = Some(pixels);
                            let _ = reply.try_send(response);
                        }
                    }
                    Err(e) => {
                        for (id, reply) in remote.pending.drain(..) {
                            let _ = reply.try_send(Reply::error(id, &e));
                        }
                    }
                }
            }
            // This is deliberately appended AFTER readback. It belongs to the
            // on-device UI and must never be included in remote images/video.
            let width = self.phys_size[0] as f32 / self.scale;
            let bounds = bhoo::Rect {
                x: 8.0,
                y: 36.0,
                width: width - 16.0,
                height: 28.0,
            };
            self.commands.borrow_mut().extend([
                bhoo::DrawCommand::RoundedRect {
                    bounds,
                    radius: 8.0,
                    color: bhoo::Color(180, 20, 30, 255),
                },
                bhoo::DrawCommand::Text {
                    bounds,
                    text: Arc::from("REMOTE CONTROL · touch phone to stop"),
                    color: bhoo::Color(255, 255, 255, 255),
                    style: bhoo::TextStyle {
                        size: 12.0,
                        weight: 700,
                        align: 1,
                        ..Default::default()
                    },
                },
            ]);
        }
        self.remote_control = Some(remote);
    }
}
