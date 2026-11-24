use crate::*;

impl Frame<'_> {
    /// Menu action: transparent rows share the panel surface, with immediate state layers.
    /// Android Material popup row: body-large text and an optional trailing symbol.
    pub fn android_popup_item(&mut self,id:u64,label:&str,enabled:bool,trailing:Option<MaterialIcon>)->Response {
        let b=self.allocate(48.);let r=self.interact(id,b,enabled);let t=self.ui.theme;
        let opacity=if r.pressed {0.08} else if r.focused && self.ui.keyboard_focus {0.12} else if r.hovered && self.input.pointer_kind != PointerKind::Touch {0.08} else {0.};
        if r.enabled && opacity>0. {self.rect(b,0.,t.text.opacity(opacity));}
        self.ink(r,0.,t.text);
        self.styled_text(Rect{x:b.x+16.,width:(b.width-if trailing.is_some(){56.}else{32.}).max(0.),..b},label,t.text.opacity(if r.enabled {1.} else {0.38}),TextStyle{size:16.,weight:400,..TextStyle::default()});
        if let Some(icon)=trailing {self.paint_material_icon(Rect{x:b.x+b.width-40.,y:b.y+12.,width:24.,height:24.},icon,t.on_surface_variant);}
        self.focus_ring(r);self.semantic(r,Role::MenuItem,label,false,None,None,None);r
    }
    pub fn menu_item(&mut self, id: u64, label: &str, enabled: bool) -> Response {
        self.menu_item_inner(id, label, enabled, None, false, None)
    }
    /// Menu item with a decorative Material icon at the logical leading edge.
    pub fn menu_item_with_icon(
        &mut self,
        id: u64,
        label: &str,
        enabled: bool,
        icon: AppBarIcon,
        rtl: bool,
    ) -> Response {
        self.menu_item_inner(id, label, enabled, Some(icon), rtl, None)
    }
    /// Menu item with a leading checkmark when selected, retaining alignment for other items.
    pub fn menu_item_with_selection(&mut self,id:u64,label:&str,enabled:bool,selected:bool,rtl:bool) -> Response {
        self.menu_item_inner(id,label,enabled,Some(AppBarIcon::More),rtl,Some(selected))
    }
    #[allow(clippy::too_many_arguments)]
    fn menu_item_inner(
        &mut self,
        id: u64,
        label: &str,
        enabled: bool,
        icon: Option<AppBarIcon>,
        rtl: bool,
        selection: Option<bool>,
    ) -> Response {
        let b = self.allocate(48.0);
        let r = self.interact(id, b, enabled);
        let t = self.ui.theme;
        let opacity = if r.focused && self.ui.keyboard_focus {
            0.12
        } else if r.hovered {
            0.08
        } else {
            0.0
        };
        if r.enabled && opacity > 0.0 {
            self.rect(b, 0.0, t.text.opacity(opacity));
        }
        self.ink(r, 0.0, t.text);
        if let Some(icon) = icon {
            self.commands.push(DrawCommand::Shape {
                bounds: Rect {
                    x: if rtl {
                        b.x + b.width - 40.0
                    } else {
                        b.x + 16.0
                    },
                    y: b.y + 12.0,
                    width: 24.0,
                    height: 24.0,
                },
                index: if selection.is_some() { (crate::beercss_shapes::SHAPES.len()+crate::icons::material_navigation::ICONS.len()+MaterialIcon::Check as usize) as u8 } else { icon.shape_index(rtl) },
                color: if selection==Some(false) { Color(0,0,0,0) } else if r.enabled {
                    t.on_surface_variant
                } else {
                    t.text.opacity(0.38)
                },
                rotation: 0.0,
            });
        }
        self.styled_text(
            if icon.is_some() {
                Rect {
                    x: b.x + if rtl { 16.0 } else { 56.0 },
                    y: b.y - 1.0,
                    width: (b.width - 72.0).max(0.0),
                    ..b
                }
            } else {
                Rect { y: b.y - 1.0, ..b }
            },
            label,
            t.text.opacity(if r.enabled { 1.0 } else { 0.38 }),
            TextStyle {
                align: if rtl && icon.is_some() { 2 } else { 0 },
                padding: if icon.is_some() { 0.0 } else { 12.0 },
                ..TextStyle::LABEL
            },
        );
        self.focus_ring(r);
        self.semantic(r, Role::MenuItem, label, false, None, None, None);
        r
    }
}

/// Per-menu typeahead buffer. Reset when the menu closes.
#[derive(Default)]
pub struct MenuTypeaheadState {
    buffer: String,
    last_input: f64,
    last_focus: Option<u64>,
}
impl MenuTypeaheadState {
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.last_focus = None;
    }
}

impl Frame<'_> {
    /// Match printable keys against enabled menu labels while a menu item has focus.
    /// IDs must be contiguous starting at `first_id`; callers reset the state on close.
    pub fn menu_typeahead(
        &mut self,
        state: &mut MenuTypeaheadState,
        first_id: u64,
        items: &[(&str, bool)],
    ) -> Option<usize> {
        let focus = self.focused_id();
        if self.input.blurred || self.input.cancelled || !self.enabled || focus.is_none() {
            state.reset();
            return None;
        }
        let Some(current) = focus
            .and_then(|id| id.checked_sub(first_id))
            .and_then(|i| usize::try_from(i).ok())
            .filter(|i| *i < items.len())
        else {
            state.reset();
            return None;
        };
        let found = self.typeahead_match(state, focus.unwrap(), current, items);
        if let Some(index) = found {
            self.request_focus(first_id + index as u64);
            state.last_focus = Some(first_id + index as u64);
        }
        found
    }

    /// Match printable keys while a closed select trigger owns focus.
    /// The caller commits the returned option and resets the buffer on opening.
    pub fn select_typeahead(&mut self, state: &mut MenuTypeaheadState, trigger_id: u64, current: usize, items: &[(&str, bool)]) -> Option<usize> {
        if self.input.blurred || self.input.cancelled || !self.enabled || self.focused_id()!=Some(trigger_id) || current>=items.len() {
            state.reset();
            return None;
        }
        self.typeahead_match(state, trigger_id, current, items)
    }

    fn typeahead_match(&mut self, state: &mut MenuTypeaheadState, focus: u64, current: usize, items: &[(&str, bool)]) -> Option<usize> {
        if state.last_focus.is_some_and(|last| last != focus) { state.reset(); }
        if self.key_consumed { return None; }
        let key = self.input.key.filter(|key| key.pressed)?;
        if !self.ui.time.is_finite() || self.ui.time - state.last_input > 0.2 || self.ui.time < state.last_input { state.reset(); }
        let ch = match key.key {
            Key::Character(ch) => ch,
            Key::Space if !state.buffer.is_empty() => ' ',
            _ => { state.reset(); return None; }
        };
        if ch.is_control() || ch.is_whitespace() && state.buffer.is_empty() { return None; }
        if state.buffer.len() >= 128 { state.reset(); }
        let fresh = state.buffer.is_empty();
        state.buffer.extend(ch.to_lowercase());
        state.last_input = self.ui.time;
        state.last_focus = Some(focus);
        self.key_consumed = true;
        let start = if fresh { (current + 1) % items.len() } else { current };
        let found = (0..items.len()).map(|step| (start + step) % items.len())
            .find(|i| items[*i].1 && items[*i].0.trim().to_lowercase().starts_with(&state.buffer));
        if found.is_none() { state.reset(); }
        found
    }
}

#[cfg(test)]
#[path = "menu.spec.rs"]
mod tests;
