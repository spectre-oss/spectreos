use crate::*;

/// Material switch icon visibility.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum SwitchIconMode {#[default] None, Selected, Both}
#[derive(Clone,Copy)]
pub(crate) struct SwitchLayout {pub min_width:f32,pub padding:f32,pub thumb_text_padding:f32}
impl Default for SwitchLayout {fn default()->Self{Self{min_width:52.0,padding:4.0,thumb_text_padding:4.0}}}
pub(crate) fn switch_widths(texts:Option<(&str,&str)>,metrics:crate::TextMetrics,layout:SwitchLayout)->(f32,f32) {
    let text_min=texts.map(|(on,off)|{let style=TextStyle{size:9.0,..TextStyle::default()};(metrics.width(on,style).max(metrics.width(off,style))+2.0*layout.thumb_text_padding).max(24.0)}).unwrap_or(0.0);
    (text_min,(2.0*text_min+4.0).max(layout.min_width).max(52.0))
}

impl Frame<'_> {
    pub fn switch(&mut self, id: u64, label: &str, value: &mut bool, enabled: bool) -> Response {
        self.switch_impl(id, label, value, enabled, false, false, SwitchIconMode::None, [[None;2];2],None,SwitchLayout::default())
    }
    /// Switch with the control following its label, as in an Android Switch view.
    pub fn switch_trailing(&mut self, id: u64, label: &str, value: &mut bool, enabled: bool) -> Response {
        self.switch_impl(id, label, value, enabled, true, false, SwitchIconMode::None, [[None;2];2],None,SwitchLayout::default())
    }
    /// Android-style trailing switch with layout-direction-aware placement.
    pub fn switch_trailing_directional(&mut self, id: u64, label: &str, value: &mut bool, enabled: bool, rtl: bool) -> Response {
        self.switch_impl(id, label, value, enabled, !rtl, rtl, SwitchIconMode::None, [[None;2];2],None,SwitchLayout::default())
    }
    /// Switch with optional selected and unselected icons.
    pub fn switch_with_icons(&mut self,id:u64,label:&str,value:&mut bool,enabled:bool,icons:SwitchIconMode)->Response {self.switch_impl(id,label,value,enabled,false,false,icons,[[None;2];2],None,SwitchLayout::default())}
    /// Directional switch with optional icons and configurable control placement.
    pub fn switch_with_icons_directional(&mut self,id:u64,label:&str,value:&mut bool,enabled:bool,trailing:bool,rtl:bool,icons:SwitchIconMode)->Response {self.switch_impl(id,label,value,enabled,trailing,rtl,icons,[[None;2];2],None,SwitchLayout::default())}
    pub(crate) fn switch_trailing_tinted(&mut self,id:u64,label:&str,value:&mut bool,enabled:bool,rtl:bool,tints:[[Option<Color>;2];2],texts:Option<(&str,&str)>,layout:SwitchLayout)->Response {self.switch_impl(id,label,value,enabled,!rtl,rtl,SwitchIconMode::None,tints,texts,layout)}
    fn switch_impl(&mut self, id: u64, label: &str, value: &mut bool, enabled: bool, trailing: bool, rtl: bool, icons:SwitchIconMode, tints:[[Option<Color>;2];2], texts:Option<(&str,&str)>, layout:SwitchLayout) -> Response {
        let b = self.allocate(36.0);
        let mut r = self.interact(id, b, enabled);
        if r.clicked {
            *value = !*value;
            r.changed = true;
        }
        let a = self.animate_geometry(id, 3, f32::from(*value), f32::from(*value));
        let t = self.ui.theme;
        let (text_min_width,preferred_width)=switch_widths(texts,self.ui.text_metrics,layout);
        let track_width=preferred_width.min(b.width.max(0.0));
        let track = Rect {
            x: if trailing { (b.x + b.width - track_width).max(b.x) } else { b.x },
            y: b.y + 2.0,
            width: track_width,
            height: 32.0,
        };
        let disabled = t.background.mix(t.text, 0.12);
        let edge = tints[1][usize::from(*value)].unwrap_or(if r.enabled {
            t.outline.mix(t.primary, a)
        } else {
            disabled
        });
        let fill = tints[1][usize::from(*value)].unwrap_or(if r.enabled {
            t.surface_highest.mix(t.primary, a)
        } else {
            disabled
        });
        self.rect(track, 16.0, edge);
        self.rect(track.inset(2.0), 14.0, fill);
        let icon_visible=match icons {SwitchIconMode::None=>false,SwitchIconMode::Selected=>*value,SwitchIconMode::Both=>true};
        let base_size=if icons==SwitchIconMode::Both {24.0}else{16.0+8.0*a};
        let pressed_target=f32::from(r.enabled&&r.pressed);
        let press=self.ui.motion.css_tween(id,220,pressed_target,0.0,crate::timing::TweenSpec::new(0.1,crate::timing::Easing::Linear),crate::timing::TweenSpec::new(0.25,crate::timing::Easing::Standard));
        let diameter=base_size+(28.0-base_size)*press;
        let thumb_width=diameter.max(text_min_width).min((track.width-4.0).max(0.0));
        let start_center=if texts.is_some(){track.x+2.0+thumb_width/2.0}else{track.x+16.0};
        let travel=if texts.is_some(){(track.width-4.0-thumb_width).max(0.0)}else{(track.width-32.0).max(0.0)};
        let center_x=start_center+travel*if rtl{1.0-a}else{a};
        let thumb=Rect{x:center_x-thumb_width/2.0,y:track.y+16.0-diameter/2.0,width:thumb_width,height:diameter};
        let hover = self.animate(id, 0, f32::from(r.hovered), 0.0);
        let ink = Rect {
            x: center_x - 20.0,
            y: track.y - 4.0,
            width: 40.0,
            height: 40.0,
        };
        if r.enabled && (hover > 0.001 || r.focused && self.ui.keyboard_focus) {
            self.rect(
                ink,
                20.0,
                (if *value{t.primary}else{t.text}).opacity(if r.focused && self.ui.keyboard_focus {
                    0.12
                } else {
                    0.08 * hover
                }),
            );
        }
        self.ink(Response { bounds: ink, ..r }, 20.0, if *value{t.primary}else{t.text});
        let handle = tints[0][usize::from(*value)].unwrap_or(if r.enabled {
            t.outline.mix(t.on_primary, a)
        } else if *value {t.background} else {t.text.opacity(0.38)});
        self.rect(thumb, diameter/2.0, handle);
        if let Some((on,off))=texts {
            let text=if a>0.5 {on}else{off};
            self.commands.push(DrawCommand::PushClip(thumb.intersect(self.clip)));
            self.styled_text(thumb,text,t.on_surface_variant.opacity(if r.enabled{1.0}else{0.38}),TextStyle{size:9.0,align:1,..TextStyle::default()});
            self.commands.push(DrawCommand::PopClip);
        }
        let mark = if r.enabled {
            t.surface_highest.mix(t.on_primary_container, a)
        } else if *value {t.text.opacity(0.38)} else {t.surface_highest.opacity(0.38)};
        if icon_visible {
            let cx=thumb.x+thumb.width/2.0;let cy=thumb.y+thumb.height/2.0;
            if *value {self.paint_line([cx-5.0,cy],[cx-2.0,cy+3.0],1.5,mark);self.paint_line([cx-2.0,cy+3.0],[cx+5.0,cy-4.0],1.5,mark);}
            else {self.paint_line([cx-4.0,cy-4.0],[cx+4.0,cy+4.0],1.5,mark);self.paint_line([cx+4.0,cy-4.0],[cx-4.0,cy+4.0],1.5,mark);}
        }
        self.styled_text(
            Rect {
                x: if trailing { b.x } else { b.x + track_width + layout.padding },
                width: (b.width - track_width - layout.padding).max(0.0),
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
            Role::Switch,
            label,
            *value,
            Some(if *value {
                CheckState::Checked
            } else {
                CheckState::Unchecked
            }),
            None,
            None,
        );
        r
    }
}

#[cfg(test)]
#[path = "switch.spec.rs"]
mod tests;
