use super::*;

const ID: u64 = SHADE_ID + 600;
const MIN_ACTIVE_TILES: usize = 4;
const DEFAULT_ACTIVE_TILES: usize = 16;
const TILE_HEIGHT: f32 = 78.0;
const ROW_PITCH: f32 = 86.0;
const FIRST_ROW: f32 = 92.0;
const AVAILABLE_HEADER: f32 = 66.0;

fn key(tile: QsTileKind) -> &'static str {
    match tile {
        QsTileKind::Wifi => "wifi",
        QsTileKind::Bluetooth => "bluetooth",
        QsTileKind::Flashlight => "flashlight",
        QsTileKind::Shiraz => "shiraz",
        QsTileKind::Alarm => "alarm",
        QsTileKind::Airplane => "airplane",
        QsTileKind::DeviceControls => "device_controls",
        QsTileKind::AutoRotate => "auto_rotate",
        QsTileKind::BatterySaver => "battery_saver",
        QsTileKind::DarkTheme => "dark_theme",
        QsTileKind::Dnd => "do_not_disturb",
        QsTileKind::ScreenRecord => "screen_record",
        QsTileKind::NightLight => "night_light",
        QsTileKind::CameraAccess => "camera_access",
        QsTileKind::MicAccess => "mic_access",
        QsTileKind::Aod => "always_on_display",
        QsTileKind::Hotspot => "hotspot",
    }
}

fn defaults() -> Vec<QsTileKind> {
    QsTileKind::DEFAULT_ORDER[..DEFAULT_ACTIVE_TILES].to_vec()
}

fn parse_tiles(value: &str) -> Vec<QsTileKind> {
    let mut tiles = Vec::new();
    for tag in value.split_whitespace() {
        if let Some(&tile) = QsTileKind::DEFAULT_ORDER
            .iter()
            .find(|&&tile| key(tile) == tag)
            && !tiles.contains(&tile)
        {
            tiles.push(tile);
        }
    }
    if tiles.len() < MIN_ACTIVE_TILES {
        defaults()
    } else {
        tiles
    }
}

pub(super) fn load_tiles() -> Vec<QsTileKind> {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(root) = settings::xdg_config_home()
        && let Ok(value) = std::fs::read_to_string(root.join("wisp/quick-settings"))
    {
        return parse_tiles(&value);
    }
    defaults()
}

fn available_tiles(tiles: &[QsTileKind]) -> impl Iterator<Item = QsTileKind> + '_ {
    QsTileKind::DEFAULT_ORDER
        .iter()
        .copied()
        .filter(|tile| !tiles.contains(tile))
}

struct EditorLayout {
    body: Rect,
    active_count: usize,
    available_count: usize,
    section_y: f32,
}

impl EditorLayout {
    fn new(phone: Rect, top: f32, active_count: usize) -> Self {
        let body = Rect {
            y: top + 70.0,
            height: (phone.height - (top - phone.y) - 70.0).max(0.0),
            ..phone
        };
        Self {
            body,
            active_count,
            available_count: QsTileKind::DEFAULT_ORDER.len() - active_count,
            section_y: FIRST_ROW + active_count.div_ceil(2) as f32 * ROW_PITCH + 4.0,
        }
    }

    fn max_scroll(&self) -> f32 {
        let end = self.section_y
            + AVAILABLE_HEADER
            + self.available_count.div_ceil(2) as f32 * ROW_PITCH
            + 28.0;
        (end - self.body.height).max(0.0)
    }

    fn tile(&self, index: usize, scroll: f32) -> Rect {
        let (row, origin) = if index < self.active_count {
            (index, FIRST_ROW)
        } else {
            (index - self.active_count, self.section_y + AVAILABLE_HEADER)
        };
        let width = (self.body.width - 40.0) / 2.0;
        Rect {
            x: self.body.x + 16.0 + (row % 2) as f32 * (width + 8.0),
            y: self.body.y + origin + (row / 2) as f32 * ROW_PITCH - scroll,
            width,
            height: TILE_HEIGHT,
        }
    }

    fn drop_slot(&self, point: [f32; 2], scroll: f32) -> Option<usize> {
        if !self.body.contains(point) {
            return None;
        }
        let y = point[1] - self.body.y + scroll;
        if y >= self.section_y {
            return Some(self.active_count + 1);
        }
        let col = usize::from(point[0] >= self.body.x + self.body.width * 0.5);
        let row = ((y - FIRST_ROW).max(0.0) / ROW_PITCH) as usize;
        Some((row * 2 + col).min(self.active_count))
    }
}

impl<E: EmbeddedApps> LauncherDemo<E> {
    fn save_qs_tiles(&mut self, now: f64) {
        let last_page = self.qs_tiles.len().div_ceil(8) - 1;
        if self.qs_page > last_page {
            self.qs_page = last_page;
            self.qs_page_anim.snap(last_page as f32);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if self.native_compositor
            && let Some(root) = settings::xdg_config_home()
        {
            let dir = root.join("wisp");
            let temp = dir.join(format!(".quick-settings-{}", std::process::id()));
            let mut data = self
                .qs_tiles
                .iter()
                .copied()
                .map(key)
                .collect::<Vec<_>>()
                .join("\n");
            data.push('\n');
            if let Err(error) = std::fs::create_dir_all(&dir)
                .and_then(|()| std::fs::write(&temp, data))
                .and_then(|()| std::fs::rename(&temp, dir.join("quick-settings")))
            {
                let _ = std::fs::remove_file(temp);
                self.show_toast(format!("Could not save Quick Settings: {error}"), now);
            }
        }
    }

    fn add_qs_tile(&mut self, index: usize, slot: usize, now: f64) {
        let tile = available_tiles(&self.qs_tiles).nth(index);
        if let Some(tile) = tile {
            self.qs_tiles.insert(slot, tile);
            self.save_qs_tiles(now);
        }
    }

    pub(super) fn drop_qs_editor_tile(
        &mut self,
        phone: Rect,
        status_h: f32,
        index: usize,
        point: [f32; 2],
        now: f64,
    ) {
        let count = self.qs_tiles.len();
        let layout = EditorLayout::new(phone, phone.y + status_h, count);
        let Some(slot) = layout.drop_slot(point, self.qs_edit_scroll) else {
            return;
        };
        if index >= count {
            if slot <= count {
                self.add_qs_tile(index - count, slot, now);
            }
        } else if slot > count {
            if count > MIN_ACTIVE_TILES {
                self.qs_tiles.remove(index);
                self.save_qs_tiles(now);
            } else {
                self.show_toast("Keep at least four Quick Settings tiles", now);
            }
        } else {
            let to = slot.min(count - 1);
            if to != index {
                let tile = self.qs_tiles.remove(index);
                self.qs_tiles.insert(to, tile);
                self.save_qs_tiles(now);
            }
        }
    }

    fn paint_editor_tile(
        &self,
        f: &mut Frame<'_>,
        bounds: Rect,
        kind: QsTileKind,
        alpha: f32,
        interactive: bool,
    ) -> bool {
        let p = self.sys.palette();
        let (mut title, _, icon, active) = self.qs_tile_meta(kind);
        if kind == QsTileKind::Wifi {
            title = "Wi-Fi".into();
        }
        let unavailable = match kind {
            QsTileKind::Wifi => !self.sys.wifi_available,
            QsTileKind::Bluetooth => !self.sys.bluetooth_available,
            QsTileKind::Flashlight => !self.sys.flashlight_available,
            QsTileKind::DeviceControls => true,
            _ => false,
        };
        let (surface, ink) = if unavailable {
            (p.unavailable, p.on_surface_variant)
        } else if active {
            (p.primary, p.on_primary)
        } else {
            (p.card_pure, p.on_surface)
        };
        let radius = 28.0_f32.min(bounds.height * 0.5);
        f.paint_rect(bounds, radius, surface.opacity(alpha));
        let mut clicked = false;
        if interactive {
            let target = f.target(ID + 10 + kind as u64, bounds, &title, true, Role::Button);
            f.state_layer(target, radius, ink);
            clicked = target.clicked;
        }
        paint_sys_icon(
            f,
            Rect {
                x: bounds.x + 15.0,
                y: bounds.y + (bounds.height - 20.0) / 2.0,
                width: 20.0,
                height: 20.0,
            },
            icon,
            ink.opacity(alpha),
        );
        f.styled_text(
            Rect {
                x: bounds.x + 44.0,
                y: bounds.y,
                width: (bounds.width - 53.0).max(0.0),
                height: bounds.height,
            },
            &title,
            ink.opacity(alpha),
            text_style(13.0, 500, 0),
        );
        clicked
    }

    pub(super) fn render_qs_tile_editor(
        &mut self,
        f: &mut Frame<'_>,
        phone: Rect,
        status_h: f32,
        _pointer: Option<[f32; 2]>,
        pointer_down: bool,
        now: f64,
    ) {
        let p = self.sys.palette();
        let progress = self.qs_edit_anim.value.clamp(0.0, 1.0);
        if progress <= 0.005 {
            return;
        }

        let [ox, oy] = self.qs_edit_origin;
        let radius = lerp(16.0, phone.width.hypot(phone.height) * 1.04, progress);
        if progress >= 0.995 {
            f.paint_rect(phone, 0.0, p.qs_backdrop);
        } else {
            f.paint_rect(
                Rect {
                    x: ox - radius,
                    y: oy - radius,
                    width: radius * 2.0,
                    height: radius * 2.0,
                },
                radius,
                p.qs_backdrop,
            );
        }
        let alpha = |rect: Rect| {
            if progress >= 0.995 {
                return 1.0;
            }
            let distance = (rect.x + rect.width * 0.5 - ox).hypot(rect.y + rect.height * 0.5 - oy);
            smoothstep(distance - 4.0, distance + 12.0, radius)
        };

        let top = phone.y + status_h;
        let layout = EditorLayout::new(phone, top, self.qs_tiles.len());
        let max_scroll = layout.max_scroll();
        self.last_scroll_max[5] = max_scroll;
        self.qs_edit_scroll = self.qs_edit_scroll.clamp(0.0, max_scroll);
        let interactive = self.qs_edit_open && progress >= 0.95;
        let dt =
            (now - self.qs_edit_last_frame.replace(now).unwrap_or(now)).clamp(0.0, 0.04) as f32;
        if !pointer_down {
            self.qs_edit_drag = None;
        }
        if let Some((_, [_, y])) = self.qs_edit_drag {
            let from_top = y - layout.body.y;
            let from_bottom = layout.body.y + layout.body.height - y;
            let direction = (1.0 - from_bottom / 54.0).clamp(0.0, 1.0)
                - (1.0 - from_top / 54.0).clamp(0.0, 1.0);
            if direction != 0.0 {
                self.qs_edit_scroll =
                    (self.qs_edit_scroll + direction * 420.0 * dt).clamp(0.0, max_scroll);
                f.continuous(phone);
            }
        }
        if self.qs_edit_drag.is_none() {
            f.scroll_viewport(ID + 2, layout.body, &mut self.qs_edit_scroll, max_scroll);
        }

        let active_count = layout.active_count;
        let mut tiles = self.qs_tiles.clone();
        tiles.extend(available_tiles(&self.qs_tiles));
        self.last_qs_edit_tiles.clear();
        let lifted = self
            .qs_edit_drag
            .and_then(|(index, point)| tiles.get(index).copied().map(|tile| (index, point, tile)));
        let landing = lifted.and_then(|(_, point, _)| layout.drop_slot(point, self.qs_edit_scroll));
        if let Some((index, _, _)) = lifted
            && let Some(slot) = landing
            && index < active_count
            && slot < active_count
        {
            let tile = tiles.remove(index);
            tiles.insert(slot, tile);
        }

        let scroll = self.qs_edit_scroll;
        let instruction = Rect {
            y: layout.body.y + 27.0 - scroll,
            height: 36.0,
            ..layout.body
        };
        let section = Rect {
            x: phone.x + 8.0,
            y: layout.body.y + layout.section_y - scroll,
            width: phone.width - 16.0,
            height: (layout.body.height + scroll - layout.section_y).max(AVAILABLE_HEADER),
        };
        let mut add = None;
        f.within(layout.body, |f| {
            let active_hint = if lifted.is_some_and(|(i, _, _)| i >= active_count) {
                "Drag here to add this tile"
            } else {
                "Hold and drag to rearrange tiles"
            };
            f.styled_text(
                instruction,
                active_hint,
                p.on_surface_variant.opacity(alpha(instruction)),
                text_style(13.0, 400, 1),
            );
            f.paint_rect(
                section,
                22.0,
                p.settings_bg.opacity(alpha(Rect {
                    height: 28.0,
                    ..section
                })),
            );
            let caption = Rect {
                height: AVAILABLE_HEADER,
                ..section
            };
            let available_hint = if lifted.is_some_and(|(i, _, _)| i < active_count) {
                if active_count > MIN_ACTIVE_TILES {
                    "Drag here to remove this tile"
                } else {
                    "Keep at least four tiles"
                }
            } else if layout.available_count == 0 {
                "All tiles are in Quick Settings"
            } else {
                "Hold and drag or tap to add tiles"
            };
            f.styled_text(
                caption,
                available_hint,
                p.on_surface_variant.opacity(alpha(caption)),
                text_style(13.0, 400, 1),
            );

            for (index, &tile) in tiles.iter().enumerate() {
                let bounds = layout.tile(index, scroll);
                if bounds.y + bounds.height <= layout.body.y
                    || bounds.y >= layout.body.y + layout.body.height
                {
                    continue;
                }
                self.last_qs_edit_tiles
                    .push((index, bounds.intersect(layout.body)));
                let a = alpha(bounds);
                if a <= 0.01 {
                    continue;
                }
                if lifted.is_some_and(|(_, _, lifted_tile)| lifted_tile == tile) {
                    f.paint_rect(bounds, 28.0, p.outline.opacity(0.16 * a));
                    continue;
                }
                if self.paint_editor_tile(f, bounds, tile, a, interactive && lifted.is_none())
                    && index >= active_count
                {
                    add = Some(index - active_count);
                }
            }
        });
        if let Some(index) = add {
            self.add_qs_tile(index, active_count, now);
        }
        if let Some((_, [x, y], tile)) = lifted {
            let width = layout.tile(0, 0.0).width;
            self.paint_editor_tile(
                f,
                Rect {
                    x: x - width / 2.0,
                    y: y - TILE_HEIGHT / 2.0,
                    width,
                    height: TILE_HEIGHT,
                },
                tile,
                1.0,
                false,
            );
        }

        let header = Rect {
            height: layout.body.y - phone.y,
            ..phone
        };
        let header_alpha = alpha(Rect {
            y: top + 21.0,
            height: 44.0,
            ..phone
        });
        f.paint_rect(header, 0.0, p.qs_backdrop.opacity(header_alpha));
        let back = Rect {
            x: phone.x + 13.0,
            y: top + 21.0,
            width: 44.0,
            height: 44.0,
        };
        let reset = Rect {
            x: phone.x + phone.width - 64.0,
            y: back.y,
            width: 61.0,
            height: back.height,
        };
        if header_alpha > 0.01 {
            let back_target = f.target(ID, back, "Done editing", interactive, Role::Button);
            let reset_target = f.target(ID + 1, reset, "Reset tiles", interactive, Role::Button);
            f.state_layer(back_target, 22.0, p.on_surface.opacity(header_alpha));
            f.state_layer(reset_target, 22.0, p.on_surface.opacity(header_alpha));
            paint_sys_icon(
                f,
                back.inset(11.0),
                SysIcon::Back,
                p.on_surface.opacity(header_alpha),
            );
            f.styled_text(
                Rect {
                    x: phone.x + 77.0,
                    width: 100.0,
                    ..back
                },
                "Edit",
                p.on_surface.opacity(header_alpha),
                text_style(21.0, 500, 0),
            );
            f.styled_text(
                reset,
                "RESET",
                p.on_surface.opacity(header_alpha),
                text_style(12.5, 500, 1),
            );
            if back_target.clicked {
                self.qs_edit_open = false;
                self.qs_edit_anim.set(0.0, now, 0.28, 0.24);
                self.qs_edit_drag = None;
            }
            if reset_target.clicked {
                self.qs_tiles = defaults();
                self.qs_edit_scroll = 0.0;
                self.save_qs_tiles(now);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_order_deduplicates_and_preserves_only_known_tiles() {
        let tiles = parse_tiles("hotspot\nwifi\nunknown\nwifi\nbluetooth\nalways_on_display");
        assert_eq!(
            tiles,
            [
                QsTileKind::Hotspot,
                QsTileKind::Wifi,
                QsTileKind::Bluetooth,
                QsTileKind::Aod
            ]
        );
        assert_eq!(parse_tiles("wifi"), defaults());
    }

    #[test]
    fn editor_drops_reorder_and_move_tiles_between_sections() {
        let mut shell = LauncherDemo::<NoopEmbeddedApps>::default();
        shell.qs_tiles = defaults();
        let phone = Rect {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 832.0,
        };
        shell.drop_qs_editor_tile(phone, 31.0, 0, [300.0, 217.0], 1.0);
        assert_eq!(
            shell.qs_tiles[..2],
            [QsTileKind::Bluetooth, QsTileKind::Wifi]
        );
        let removed = shell.qs_tiles[0];
        shell.qs_edit_scroll = 245.0;
        shell.drop_qs_editor_tile(phone, 31.0, 0, [300.0, 700.0], 2.0);
        assert_eq!(shell.qs_tiles.len(), 15);
        assert!(!shell.qs_tiles.contains(&removed));
        shell.qs_edit_scroll = 0.0;
        shell.drop_qs_editor_tile(phone, 31.0, 15, [100.0, 217.0], 3.0);
        assert_eq!(shell.qs_tiles.len(), 16);
        assert_eq!(shell.qs_tiles[0], removed);
    }
}
