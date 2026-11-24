use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct Choice<'a> {
    pub id: u64,
    pub label: &'a str,
    pub enabled: bool,
}

impl Frame<'_> {
    fn radio_indicator(&mut self,id:u64,outer:Rect,selected:bool,enabled:bool,tint:Option<Color>) {
        use crate::timing::{Easing,TweenSpec};
        let target=f32::from(selected);let linear=TweenSpec::new(0.05,Easing::Linear);
        let color_progress=self.ui.motion.css_tween(id,210,target,target,linear,linear);
        let opacity=self.ui.motion.css_tween(id,211,target,target,linear,linear);
        let scale=self.ui.motion.css_tween(id,212,target,target,TweenSpec::new(0.3,Easing::EmphasizedDecelerate),TweenSpec::new(0.0,Easing::Linear));
        let theme=self.ui.theme;let color=tint.unwrap_or(if enabled{theme.on_surface_variant.mix(theme.primary,color_progress)}else{theme.text.opacity(0.38)});
        self.commands.push(DrawCommand::Outline{bounds:outer,radius:10.0,width:2.0,color});
        let alpha=if enabled{opacity}else{target};let scale=if !enabled||!selected{1.0}else{scale};let diameter=10.0*scale;
        if alpha>0.001&&diameter>0.001 {let dot=Rect{x:outer.x+(outer.width-diameter)/2.0,y:outer.y+(outer.height-diameter)/2.0,width:diameter,height:diameter};self.rect(dot,diameter/2.0,tint.map(|c|c.opacity(alpha)).unwrap_or(if enabled{theme.primary.opacity(alpha)}else{theme.text.opacity(0.38)}));}
    }
    /// Radio choice primitive. Group selection/navigation is supplied by `radio_group`.
    pub fn radio(&mut self, id: u64, label: &str, selected: bool, enabled: bool) -> Response {
        self.radio_directional(id, label, selected, enabled, false)
    }
    /// Radio choice with layout-direction-aware indicator placement.
    pub fn radio_directional(&mut self, id: u64, label: &str, selected: bool, enabled: bool, rtl: bool) -> Response {
        self.radio_directional_tinted(id,label,selected,enabled,rtl,None,false)
    }
    pub(crate) fn radio_directional_tinted(&mut self,id:u64,label:&str,selected:bool,enabled:bool,rtl:bool,tint:Option<Color>,custom_indicator:bool)->Response {
        let b = self.allocate(40.0);
        let r = self.interact(id, b, enabled);
        let t = self.ui.theme;
        let outer = Rect {
            x: if rtl { b.x + b.width - 30.0 } else { b.x + 10.0 },
            y: b.y + 10.0,
            width: 20.0,
            height: 20.0,
        };
        self.state_layer(
            Response {
                bounds: Rect {
                    x: outer.x - 10.0,
                    y: outer.y - 10.0,
                    width: 40.0,
                    height: 40.0,
                },
                ..r
            },
            20.0,
            if selected {t.primary}else{t.on_surface_variant},
        );
        if !custom_indicator {self.radio_indicator(id,outer,selected,r.enabled,tint);}
        self.styled_text(
            Rect {
                x: if rtl { b.x } else { b.x + 44.0 },
                width: (b.width - 44.0).max(0.0),
                ..b
            },
            label,
            t.text.opacity(if r.enabled { 1.0 } else { 0.38 }),
            TextStyle {
                size: 14.0,
                align: if rtl { 2 } else { 0 },
                letter_spacing: 0.25,
                ..TextStyle::default()
            },
        );
        self.focus_ring(r);
        self.semantic(
            r,
            Role::Radio,
            label,
            selected,
            Some(if selected {
                CheckState::Checked
            } else {
                CheckState::Unchecked
            }),
            None,
            None,
        );
        r
    }
    /// A roving radio group: one Tab stop, arrows/Home/End select enabled choices.
    /// Each choice has a stable semantic/motion ID distinct from the group ID.
    pub fn radio_group(
        &mut self,
        id: u64,
        label: &str,
        choices: &[Choice<'_>],
        selected: &mut u64,
        enabled: bool,
    ) -> Response {
        self.radio_group_directional(id,label,choices,selected,enabled,false)
    }
    /// Radio group with mirrored indicator and label placement for right-to-left layouts.
    pub fn radio_group_directional(&mut self,id:u64,label:&str,choices:&[Choice<'_>],selected:&mut u64,enabled:bool,rtl:bool)->Response {
        self.label(label);
        let b = self.allocate((52.0 * choices.len() as f32 - 12.0).max(0.0));
        let mut r = self.interact(id, b, enabled && choices.iter().any(|c| c.enabled));
        let previous = *selected;
        let valid = |index: usize| choices[index].enabled;
        let current = choices.iter().position(|c| c.id == *selected && c.enabled);
        if current.is_none()
            && let Some(first) = choices.iter().find(|c| c.enabled)
        {
            *selected = first.id;
        }
        if r.clicked
            && self.input.released
            && self.ui.capture == Capture::Pointer
            && let Some(p) = self.input.pointer
        {
            let index = ((p[1] - b.y) / 52.0).floor().max(0.0) as usize;
            if let Some(choice) = choices.get(index).filter(|c| c.enabled) {
                *selected = choice.id;
            } else {
                r.clicked = false;
            }
        }
        if let Some(key) = self.key_for(r) {
            let len = choices.len();
            let start = choices.iter().position(|c| c.id == *selected).unwrap_or(0);
            let target = match key {
                Key::Down => (1..=len).map(|n| (start + n) % len).find(|i| valid(*i)),
                Key::Up => (1..=len)
                    .map(|n| (start + len - n) % len)
                    .find(|i| valid(*i)),
                Key::Right if !rtl => (1..=len).map(|n| (start+n)%len).find(|i|valid(*i)),
                Key::Left if rtl => (1..=len).map(|n| (start+n)%len).find(|i|valid(*i)),
                Key::Left if !rtl => (1..=len).map(|n| (start+len-n)%len).find(|i|valid(*i)),
                Key::Right if rtl => (1..=len).map(|n| (start+len-n)%len).find(|i|valid(*i)),
                Key::Home => (0..len).find(|i| valid(*i)),
                Key::End => (0..len).rev().find(|i| valid(*i)),
                _ => None,
            };
            if let Some(index) = target {
                *selected = choices[index].id;
            }
        }
        self.semantic(r, Role::RadioGroup, label, false, None, None, None);
        for (index, choice) in choices.iter().enumerate() {
            assert!(self.ui.seen.insert(choice.id), "Duplicate radio choice ID");
            let row = Rect {
                y: b.y + index as f32 * 52.0,
                height: 40.0,
                ..b
            };
            let checked = choice.id == *selected;
            let response = Response {
                id: choice.id,
                bounds: row,
                enabled: r.enabled && choice.enabled,
                focused: r.focused && checked,
                ..Response::default()
            };
            let t = self.ui.theme;
            let outer = Rect {
                x: if rtl {row.x+row.width-30.0}else{row.x+10.0},
                y: row.y + 10.0,
                width: 20.0,
                height: 20.0,
            };
            let opacity = if response.enabled { 1.0 } else { 0.38 };
            let press = r.pressed
                && (r.focused && self.ui.keyboard_focus && checked
                    || self.input.pointer.is_some_and(|p| row.contains(p)));
            self.state_layer(
                Response {
                    pressed: press,
                    hovered: response.enabled
                        && self.input.pointer.is_some_and(|p| row.contains(p)),
                    bounds: Rect {
                        x: outer.x - 10.0,
                        y: outer.y - 10.0,
                        width: 40.0,
                        height: 40.0,
                    },
                    ..response
                },
                20.0,
                if checked {t.primary}else{t.on_surface_variant},
            );
            self.radio_indicator(choice.id,outer,checked,response.enabled,None);
            self.styled_text(
                Rect {
                    x: if rtl {row.x}else{row.x+44.0},
                    width: (row.width - 44.0).max(0.0),
                    ..row
                },
                choice.label,
                t.text.opacity(opacity),
                TextStyle {
                    size: 14.0,
                    align: if rtl {2}else{0},
                    letter_spacing: 0.25,
                    ..TextStyle::default()
                },
            );
            self.focus_ring(response);
            self.semantic(
                response,
                Role::Radio,
                choice.label,
                checked,
                Some(if checked {
                    CheckState::Checked
                } else {
                    CheckState::Unchecked
                }),
                None,
                None,
            );
        }
        r.changed = previous != *selected;
        r
    }
}

#[cfg(test)]
#[path = "radio.spec.rs"]
mod tests;
