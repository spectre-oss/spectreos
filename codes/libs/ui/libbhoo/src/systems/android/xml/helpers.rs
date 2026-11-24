use super::*;

/// Android 15's mdpi default contact picture, for hosts that request it.
pub fn quick_contact_default_avatar()->Image {
    static IMAGE:std::sync::OnceLock<Image>=std::sync::OnceLock::new();
    IMAGE.get_or_init(||Image::rgba(32,32,include_bytes!("../../../../assets/android-quick-contact/avatar.rgba").to_vec()).expect("valid Android contact avatar")).clone()
}
pub(super) fn paint_quick_contact_overlay(f:&mut Frame<'_>,bounds:Rect,response:Response,rtl:bool) {
    static IMAGES:std::sync::OnceLock<[Image;6]>=std::sync::OnceLock::new();
    let sources:[&[u8];6]=[include_bytes!("../../../../assets/android-quick-contact/light_normal.rgba"),include_bytes!("../../../../assets/android-quick-contact/light_pressed.rgba"),include_bytes!("../../../../assets/android-quick-contact/light_focused.rgba"),include_bytes!("../../../../assets/android-quick-contact/dark_normal.rgba"),include_bytes!("../../../../assets/android-quick-contact/dark_pressed.rgba"),include_bytes!("../../../../assets/android-quick-contact/dark_focused.rgba")];
    let images=IMAGES.get_or_init(||std::array::from_fn(|i|Image::rgba(24,24,sources[i].to_vec()).expect("valid Android contact overlay")));
    let Color(r,g,b,_)=f.theme().background;let dark=(r as u32*299+g as u32*587+b as u32*114)<128000;
    let visual=if response.pressed {1}else if response.focused&&f.ui.keyboard_focus{2}else{0};let image=&images[usize::from(dark)*3+visual];
    let mark=if dark&&visual==0{2.0}else{4.0};let source=[0.0,mark,mark+1.0,24.0];
    let dest=|size:f32| {let fixed=23.0;let scale=(size/fixed).clamp(0.0,1.0);[0.0,mark*scale,(size-(23.0-mark)*scale).max(0.0),size.max(0.0)]};
    let xs=dest(bounds.width);let ys=dest(bounds.height);
    for y in 0..3 {for x in 0..3 {let width=xs[x+1]-xs[x];let height=ys[y+1]-ys[y];if width<=0.0||height<=0.0{continue;}let rect=Rect{x:bounds.x+if rtl{bounds.width-xs[x+1]}else{xs[x]},y:bounds.y+ys[y],width,height};if rect.intersect(f.clip).width<=0.0||rect.intersect(f.clip).height<=0.0{continue;}let (u0,u1)=if rtl{(source[x+1]/24.0,source[x]/24.0)}else{(source[x]/24.0,source[x+1]/24.0)};f.commands.push(DrawCommand::Image{bounds:rect,uv:[u0,source[y]/24.0,u1,source[y+1]/24.0],radius:0.0,color:Color(255,255,255,255),image:image.clone()});}}
}

pub(super) fn zoom_activated(f:&mut Frame<'_>, r:Response, b:Rect, state:&mut LayoutState, id:u64, key:&str) -> bool {
    if r.pressed && r.enabled && !f.input.cancelled && !f.input.blurred {
        let now=f.time();
        let hold=state.zoom_holds.entry(id).or_insert((now+0.5,false,false));
        if !hold.1 && f.input.pointer.is_some_and(|p| p[0]<b.x-8.0 || p[0]>b.x+b.width+8.0 || p[1]<b.y-8.0 || p[1]>b.y+b.height+8.0) { hold.2=true; }
        if !hold.2 {
            if now>=hold.0 {
                hold.1=true;
                let speed=state.zoom_speed_ms.get(key).copied().unwrap_or(1000).max(1);
                hold.0=now+speed as f64/1000.0;
                f.wake_at(hold.0);
                return true;
            }
            f.wake_at(hold.0);
        }
        false
    } else {
        let suppressed=state.zoom_holds.remove(&id).is_some_and(|(_,repeated,cancelled)| repeated || cancelled);
        r.clicked && !suppressed
    }
}
/// Stable runtime ID for a named Android XML view.
pub fn view_id(id: &str) -> u64 {
    id.bytes().fold(14695981039346656037, |h, b| {
        (h ^ b as u64).wrapping_mul(1099511628211)
    }) & ((1u64 << 52) - 1)
}

/// Derive a stable anonymous child ID distinct from named XML IDs.
pub(super) fn anonymous_child_id(parent: u64, index: usize) -> u64 {
    let mut value = 14695981039346656037u64;
    for byte in parent.to_le_bytes().into_iter().chain((index as u64).to_le_bytes()) {
        value = (value ^ u64::from(byte)).wrapping_mul(1099511628211);
    }
    (value & ((1u64 << 52) - 1)) | (1u64 << 52)
}
/// Stable menu target identity across reorder and action/overflow moves.
pub(super) fn toolbar_menu_target_id(parent:u64,item:u64,in_overflow:bool)->u64 {
    let mut value=14695981039346656037u64;
    for byte in parent.to_le_bytes().into_iter().chain(item.to_le_bytes()).chain([u8::from(in_overflow)]) {value=(value^u64::from(byte)).wrapping_mul(1099511628211);}
    value
}
pub(super) fn spinner_next_enabled(flags:&[bool],count:usize,current:usize,forward:bool)->Option<usize> {
    let enabled=|i:usize|flags.get(i).copied().unwrap_or(true);
    if forward {(current.saturating_add(1)..count).find(|&i|enabled(i))}
    else {(0..current.min(count)).rev().find(|&i|enabled(i))}
}
pub(super) fn spinner_template_state(source:&LayoutState,rtl:bool)->LayoutState {
    let mut state=LayoutState::default();state.theme_style=source.theme_style;state.density=source.density;state.font_scale=source.font_scale;state.rtl=rtl;state.images=source.images.clone();state.image_scales=source.image_scales.clone();state.image_matrices=source.image_matrices.clone();state.image_sources=source.image_sources.clone();state.drawable_level=source.drawable_level.clone();state.drawable_transition_active=source.drawable_transition_active.clone();state
}
pub(super) fn hash_adapter_environment<H:std::hash::Hasher>(hash:&mut H,row_state:&LayoutState) {
    use std::hash::{Hash,Hasher};
        row_state.theme_style.hash(hash);
        row_state.effective_density().to_bits().hash(hash);
        row_state.effective_font_scale().to_bits().hash(hash);
        row_state.rtl.hash(hash);
        // Drawable geometry and per-view source overrides can change between frames.
        // Combine entry hashes without depending on HashMap iteration order.
        let mut images_hash=0u64;
        for (name,image) in &row_state.images {
            let mut entry=std::collections::hash_map::DefaultHasher::new();
            name.hash(&mut entry);image.width().hash(&mut entry);image.height().hash(&mut entry);
            images_hash^=entry.finish();
        }
        images_hash.hash(hash);row_state.images.len().hash(hash);
        let mut scales_hash=0u64;
        for (name,scale) in &row_state.image_scales {
            let mut entry=std::collections::hash_map::DefaultHasher::new();name.hash(&mut entry);scale.to_bits().hash(&mut entry);scales_hash^=entry.finish();
        }
        scales_hash.hash(hash);row_state.image_scales.len().hash(hash);
        let mut sources_hash=0u64;
        for (id,source) in &row_state.image_sources {
            let mut entry=std::collections::hash_map::DefaultHasher::new();id.hash(&mut entry);source.hash(&mut entry);sources_hash^=entry.finish();
        }
        sources_hash.hash(hash);row_state.image_sources.len().hash(hash);
}
// Cache row offsets because wrapping each adapter item on every animation frame is expensive.
pub(super) fn spinner_row_offsets(cache:&mut HashMap<String,(u64,std::sync::Arc<Vec<f32>>)>,key:&str,entries:&[String],templates:Option<SpinnerItemTemplates>,row_state:&mut Option<LayoutState>,width:f32,metrics:TextMetrics,id:u64,divider:f32)->std::sync::Arc<Vec<f32>> {
    use std::hash::{Hash,Hasher};
    let mut hash=std::collections::hash_map::DefaultHasher::new();
    entries.hash(&mut hash);width.to_bits().hash(&mut hash);divider.to_bits().hash(&mut hash);
    templates.map(|t|(t.dropdown as *const Node as usize,t.text_id)).hash(&mut hash);
    if let Some(row_state)=row_state.as_ref() {hash_adapter_environment(&mut hash,row_state);}
    for value in metrics.normal.iter().chain(metrics.medium.iter()) {value.to_bits().hash(&mut hash);}
    let fingerprint=hash.finish();
    if let Some((old,offsets))=cache.get(key) {if *old==fingerprint {return offsets.clone();}}
    let mut offsets=Vec::with_capacity(entries.len()+1);offsets.push(0.0);
    for (i,label) in entries.iter().enumerate() {
        let height=if let (Some(template),Some(row_state))=(templates,row_state.as_mut()) {
            row_state.text.insert(template.text_id.to_owned(),label.clone());
            match template.dropdown.height {
                Size::Match=>48.0,
                _=>template.dropdown.layout_intrinsic(false,row_state,width,metrics,anonymous_child_id(id,1024+i)),
            }
        } else {48.0};
        let height=if height.is_finite() {height.clamp(1.0,4096.0)} else {48.0};
        offsets.push(offsets.last().copied().unwrap_or(0.0)+height+divider);
    }
    let offsets=std::sync::Arc::new(offsets);cache.insert(key.to_owned(),(fingerprint,offsets.clone()));offsets
}
pub(super) fn spinner_visible_rows(offsets:&[f32],offset:f32,height:f32)->(usize,usize) {
    let count=offsets.len().saturating_sub(1);
    (offsets.partition_point(|y|*y<=offset).saturating_sub(1).min(count), offsets.partition_point(|y|*y<offset+height).min(count))
}
pub(super) fn grid_item_size(template:Option<ListItemTemplate>,row_state:&mut Option<LayoutState>,label:&str,column_width:f32,metrics:TextMetrics,id:u64,binding:Option<&AdapterItemBinding>)->(f32,f32) {
    let template=adapter_item_template(template,binding);
    if let (Some(template),Some(state))=(template,row_state.as_mut()) {
        with_adapter_binding(state,template.text_id,label,binding,|state| {
            let width=match template.row.width {Size::Match=>column_width,Size::Dp(value)=>state.dp(value),Size::Wrap=>template.row.intrinsic(true,state,id).min(column_width)}.max(0.0);
            let height=match template.row.height {Size::Match=>state.dp(48.0),_=>template.row.layout_intrinsic(false,state,width,metrics,id)};
            (width,if height.is_finite(){height.clamp(1.0,4096.0)}else{state.dp(48.0)})
        })
    } else {
        let lines=wrapped_text_lines(label,(column_width-32.0).max(1.0),TextStyle::default(),metrics,|_|{}).max(1);
        (column_width,(24.0*lines as f32).clamp(48.0,4096.0))
    }
}
pub(super) fn grid_view_columns(node:&Node,width:f32,density:f32)->(usize,f32,f32,f32) {
    let rule=|kind|node.relative_rules.iter().find(|r|r.kind==kind).map(|r|r.anchor);
    let requested_width=rule(92).and_then(|s|s.parse::<f32>().ok()).unwrap_or(0.0).max(0.0)*density;
    let requested_spacing=rule(93).and_then(|s|s.parse::<f32>().ok()).unwrap_or(0.0).max(0.0)*density;
    let columns=match rule(91).unwrap_or("1") {"auto_fit"=>if requested_width>0.0 {((width+requested_spacing)/(requested_width+requested_spacing)).floor().max(1.0) as usize}else{2},s=>s.parse::<usize>().unwrap_or(1).max(1)}.min(256);
    let base=if requested_width>0.0 {requested_width}else{((width-(columns-1) as f32*requested_spacing)/columns as f32).max(0.0)};
    let leftover=width-columns as f32*base-(columns-1) as f32*requested_spacing;
    match rule(95).unwrap_or("columnWidth") {
        "none"=>(columns,base,requested_spacing,0.0),
        "spacingWidth"=>(columns,base,(requested_spacing+leftover/(columns.saturating_sub(1).max(1) as f32)).max(0.0),0.0),
        "spacingWidthUniform"=>{let spacing=(requested_spacing+leftover/(if columns>1 {columns+1}else{1}) as f32).max(0.0);(columns,base,spacing,spacing)},
        _=>(columns,(base+leftover/columns as f32).max(0.0),requested_spacing,0.0),
    }
}
pub(super) fn adapter_item_long_press(f:&mut Frame<'_>,response:Response,hold:&mut Option<ListHold>,enabled:bool)->bool {
    if !enabled {return false;}
    if f.input.pressed && response.pressed && let Some(origin)=f.input.pointer {
        *hold=Some(ListHold {item_id:response.id,origin,deadline:f.time()+0.5,fired:false});
    }
    if let Some(current)=hold.as_mut().filter(|hold|hold.item_id==response.id && !hold.fired) && let Some(pointer)=f.input.pointer {
        let dx=pointer[0]-current.origin[0];let dy=pointer[1]-current.origin[1];
        if dx*dx+dy*dy>64.0 || !response.enabled {*hold=None;f.ui.active=None;}
        else if response.pressed {
            if f.time()>=current.deadline {current.fired=true;f.ui.active=None;return true;}
            f.wake_at(current.deadline);
        }
    }
    false
}
pub(super) fn list_fixed_height(view:&Node,state:&LayoutState,width:f32,metrics:TextMetrics,id:u64)->f32 {
    let height=if matches!(view.height,Size::Match) {48.0}else{view.layout_intrinsic(false,state,width,metrics,id)};
    if height.is_finite(){height.clamp(0.0,4096.0)}else{0.0}
}
pub(super) fn draw_list_fixed(f:&mut Frame<'_>,fixed:ListFixedView,bounds:Rect,state:&mut LayoutState,id:u64,enabled:bool,controls:&mut HashMap<String,bool>)->(bool,Option<&'static str>) {
    let interactive=inspect_expandable_template(fixed.view,"").2;
    let mut control=None;
    if interactive {state.adapter_namespace=Some(id);state.checked=std::mem::take(controls);f.within(bounds,|f|f.with_enabled(enabled,|f|control=fixed.view.draw(f,bounds,state,id)));*controls=std::mem::take(&mut state.checked);state.adapter_namespace=None;}
    else {f.without_input(true,|f|f.within(bounds,|f|{fixed.view.draw(f,bounds,state,id);}));}
    let response=if fixed.selectable {Some(f.target(id,bounds,fixed.label,enabled,Role::Button))}else{None};
    let clicked=if let Some(response)=response {f.state_layer(response,0.0,f.theme().text);response.clicked&&control.is_none()}else{false};
    (clicked,control)
}
pub(super) fn paint_spinner_template(f:&mut Frame<'_>,template:&'static Node,text_id:&str,text:&str,bounds:Rect,state:&mut LayoutState,id:u64,response:Response,activated:bool,binding:Option<&AdapterItemBinding>) {
    let root_key=if template.id.is_empty(){format!("#{id}")}else{template.id.to_owned()};
    with_adapter_binding(state,text_id,text,binding,|state| {
        state.view_activated.insert(root_key.clone(),activated);state.adapter_root_response=Some((id,response));
        f.with_opacity(if response.enabled {1.0} else {0.38},|f| f.without_input(true,|f| f.within(bounds,|f| {template.draw(f,bounds,state,id);} )));
        state.adapter_root_response=None;state.view_activated.remove(&root_key);
    });
}

#[derive(Clone,Default)]
pub(super) struct TimeClockTransition {pub(super) target:bool,pub(super) start:f64,pub(super) from:f32,pub(super) value:f32}
impl TimeClockTransition {
    pub(super) fn advance(&mut self,now:f64){let p=((now-self.start)/0.5).clamp(0.0,1.0) as f32;let eased=(1.0-(std::f32::consts::PI*p).cos())*0.5;self.value=self.from+(f32::from(self.target)-self.from)*eased;}
    pub(super) fn sample(&mut self,now:f64,target:bool,reduced:bool)->(f32,bool){self.advance(now);if self.target!=target{self.from=self.value;self.target=target;self.start=now;}if reduced{self.value=f32::from(target);self.from=self.value;}else{self.advance(now);} (self.value,(self.value-f32::from(target)).abs()>0.00001)}
}
#[derive(Clone,Default)]
pub(super) struct NumberPickerMotion {pub(super) origin:Option<f32>,pub(super) last:f32,pub(super) last_time:f64,pub(super) velocity:f32,pub(super) dragging:bool,pub(super) offset:f32,pub(super) settle:Option<(f64,f32)>,pub(super) fling:Option<crate::scroll::SplineFling>,pub(super) hold:Option<(i8,f64,bool)>}
/// Apply signed wheel displacement; return whether the selected value changed and whether a boundary was reached.
pub(super) fn number_picker_move(picker:&mut crate::android_number_picker::NumberPickerState,offset:&mut f32,delta:f32,row:f32,changes:&mut Vec<i32>)->(bool,bool) {
    *offset+=delta;let mut changed=false;
    while offset.abs()>row/2.0+0.001 {let direction=if *offset<0.0{1}else{-1};if picker.step(direction){changed=true;changes.push(picker.value());*offset+=f32::from(direction)*row;}else{*offset=0.0;return(changed,true);}}
    (changed,false)
}
#[derive(Clone)]
pub(super) struct DatePickerFade { pub(super) target:bool, pub(super) start:f64, pub(super) from:[f32;2], pub(super) value:[f32;2]}
impl Default for DatePickerFade {fn default()->Self {Self{target:false,start:0.0,from:[1.0,0.0],value:[1.0,0.0]}}}
impl DatePickerFade {
    pub(super) fn advance(&mut self,now:f64) {let p=((now-self.start)/0.25).clamp(0.0,1.0) as f32;let targets=if self.target {[0.0,1.0]}else{[1.0,0.0]};for (i,target) in targets.into_iter().enumerate(){let easing=if target>self.from[i]{p*p}else{1.0-(1.0-p)*(1.0-p)};self.value[i]=self.from[i]+(target-self.from[i])*easing;}}
    pub(super) fn sample(&mut self,now:f64,target:bool,reduced:bool)->([f32;2],bool) {self.advance(now);if self.target!=target{self.from=self.value;self.target=target;self.start=now;}if reduced{self.from=if target{[0.0,1.0]}else{[1.0,0.0]};self.value=self.from;}else{self.advance(now);}let moving=if target{self.value!=[0.0,1.0]}else{self.value!=[1.0,0.0]};(self.value,moving)}
}
#[derive(Clone, Default)]
pub(super) struct CalendarMotion { pub(super) origin:Option<[f32;2]>, pub(super) last:Option<(f64,f32)>, pub(super) velocity:f32, pub(super) offset:f32, pub(super) dragging:bool, pub(super) rejected:bool, pub(super) settling:Option<(f64,f32,f64)>}
pub(super) fn draw_calendar_page(f:&mut Frame<'_>,calendar:&mut crate::android_calendar::CalendarState,b:Rect,id:u64,rtl:bool,enabled:bool,interactive:bool,locale:&crate::android_calendar::CalendarLocale,today:crate::android_calendar::CalendarDate,m3:bool,density:f32)->(i8,bool) {
    let t=f.theme();let (year,month)=calendar.visible_month();
    // Android's SimpleMonthView: 20dp horizontal padding, 6dp top padding,
    // 56dp month header, 36dp weekday row and six 40dp day rows.
    let pad_x=20.0_f32.min(b.width/4.0);let top=6.0_f32.min(b.height);let scale=((b.height-top).max(0.0)/332.0).min(1.0);let header_h=56.0*scale;let week_h=36.0*scale;let row_h=40.0*scale;let cell_w=(b.width-2.0*pad_x).max(0.0)/7.0;let grid_x=b.x+pad_x;let mut page_change=0;let mut chosen=false;
    f.styled_text(Rect{x:b.x+48.0,y:b.y+top,width:(b.width-96.0).max(0.0),height:header_h},&locale.title(year,month),if m3 { t.on_surface_variant } else { t.text },TextStyle{size:14.0,weight:500,align:1,..TextStyle::default()});
    let range=calendar.range();
    for (i,dir) in [-1_i8,1].into_iter().enumerate(){
        let available=if dir<0{(year,month)>(range.0.year,range.0.month)}else{(year,month)<(range.1.year,range.1.month)};
        if !available {continue;}
        let side=if rtl {1-i}else{i};let button=48.0_f32.min(b.width);let center=if side==0{grid_x+cell_w/2.0}else{grid_x+6.5*cell_w};let r=Rect{x:center-button/2.0,y:b.y+top+(header_h-button)/2.0,width:button,height:button};
        let hit=if interactive {f.target(anonymous_child_id(id,9100+i),r,if dir<0{"Previous month"}else{"Next month"},enabled&&available,Role::Button)}else{Response {enabled:enabled&&available,..Response::default()}};
        if interactive {f.state_layer(hit,button/2.0,t.text);}let cx=r.x+r.width/2.0;let cy=r.y+r.height/2.0;let sign=if side==0{-1.0}else{1.0};let color=(if m3 { t.on_surface_variant } else { t.text }).opacity(if hit.enabled{1.0}else{0.38});
        f.paint_line([cx-sign*2.0,cy-5.0],[cx+sign*3.0,cy],2.0,color);f.paint_line([cx+sign*3.0,cy],[cx-sign*2.0,cy+5.0],2.0,color);if hit.clicked{page_change=dir;}
    }
    for col in 0..7 {let visual=if rtl{6-col}else{col};let r=Rect{x:grid_x+visual as f32*cell_w,y:b.y+top+header_h,width:cell_w,height:week_h};let weekday=(usize::from(calendar.first_day_of_week()-1)+col)%7;f.styled_text(r,&locale.weekdays[weekday],if m3 { t.on_surface_variant } else { t.text.opacity(0.7) },TextStyle{size:12.0,align:1,..TextStyle::default()});}
    for (index,cell) in calendar.cells().into_iter().enumerate(){
        let Some(cell)=cell else{continue;};let col=index%7;let visual=if rtl{6-col}else{col};let r=Rect{x:grid_x+visual as f32*cell_w,y:b.y+top+header_h+week_h+row_h*(index/7) as f32,width:cell_w,height:row_h};
        let hit=if interactive {f.target(anonymous_child_id(id,9200+usize::from(cell.date.day)),r,&locale.number(u32::from(cell.date.day)),enabled&&cell.enabled,Role::Button)}else{Response {enabled:enabled&&cell.enabled,..Response::default()}};
        let diameter=40.0_f32.min(cell_w).min(row_h);let circle=Rect{x:r.x+(r.width-diameter)/2.0,y:r.y+(r.height-diameter)/2.0,width:diameter,height:diameter};
        if cell.selected {
            f.paint_rect(circle,diameter/2.0,t.primary);
        } else if m3 && cell.date==today {
            f.commands.push(DrawCommand::Outline { bounds: circle, radius: diameter/2.0, width: (1.0 * density).max(1.0), color: t.primary.opacity(if hit.enabled { 1.0 } else { 0.38 }) });
        }
        if interactive {f.state_layer(Response{bounds:circle,..hit},diameter/2.0,t.text);}
        let color=if cell.selected{t.on_primary}else if cell.date==today && hit.enabled {t.primary}else{t.text}.opacity(if hit.enabled{1.0}else{0.38});f.styled_text(r,&locale.number(u32::from(cell.date.day)),color,TextStyle{size:12.0,align:1,..TextStyle::default()});if hit.clicked&&calendar.select_day(cell.date.day){chosen=true;}
    }
    (page_change,chosen)
}

pub(super) fn draw_xml_calendar(node:&'static Node,f:&mut Frame<'_>,inner:Rect,state:&mut LayoutState,id:u64)->Option<&'static str> {
    let mut action=None;
                use crate::android_calendar::{CalendarDate,CalendarState};
                let key=if node.id.is_empty(){format!("#{id}")}else{node.id.to_owned()};
                let parse=|v:&str|{let mut parts=v.split('-');CalendarDate::new(parts.next()?.parse().ok()?,parts.next()?.parse().ok()?,parts.next()?.parse().ok()?)};
                let rule=|kind|node.relative_rules.iter().find(|r|r.kind==kind).map(|r|r.anchor);
                let min=rule(99).and_then(parse).unwrap_or(CalendarDate{year:1900,month:1,day:1});let max=rule(100).and_then(parse).unwrap_or(CalendarDate{year:2100,month:12,day:31});
                let today=state.wall_clock.and_then(|c|parse(&format_wall_clock(c,"yyyy-MM-dd"))).unwrap_or(min);
                let first=rule(101).and_then(|v|v.parse().ok()).unwrap_or(state.calendar_locale.first_day_of_week);
                let m3=matches!(state.theme_style, AndroidTheme::Material3Light | AndroidTheme::Material3Dark);
                let density=state.effective_density();
                let calendar=state.calendars.entry(key.clone()).or_insert_with(||CalendarState::new(min,max,today,first).expect("validated CalendarView range"));
                let motion=state.calendar_motion.entry(key).or_default();let now=f.time();let rtl=state.rtl;let width=inner.width.max(1.0);
                if let Some((start,from,duration))=motion.settling {let progress=((now-start)/duration).clamp(0.0,1.0) as f32;motion.offset=from*(1.0-progress).powi(5);if progress>=1.0||f.reduced_motion(){motion.offset=0.0;motion.settling=None;}else{f.continuous(inner);}}
                if node.enabled && f.enabled && f.input.pointer_kind==PointerKind::Touch && f.input.pressed && f.pointer_in(inner) {motion.origin=f.input.pointer;motion.last=f.input.pointer.map(|point|(now,point[0]));motion.velocity=0.0;motion.dragging=false;motion.rejected=false;motion.settling=None;motion.offset=0.0;}
                if let (Some(origin),Some(point))=(motion.origin,f.input.pointer) {
                    let dx=point[0]-origin[0];let dy=point[1]-origin[1];
                    if let Some((last_time,last_x))=motion.last {let dt=now-last_time;if dt>0.001 && (point[0]-last_x).abs()>0.01 {motion.velocity=(point[0]-last_x)/(dt as f32);motion.last=Some((now,point[0]));}}
                    if !motion.dragging&&!motion.rejected && (dx.abs()>8.0||dy.abs()>8.0) {if dx.abs()>dy.abs(){motion.dragging=true;f.ui.active=None;}else{motion.rejected=true;}}
                    if motion.dragging {let dir=if (dx<0.0)^rtl {1}else{-1};let mut neighbor=calendar.clone();motion.offset=if neighbor.move_month(dir){dx.clamp(-width,width)}else{dx.clamp(-width,width)*0.2};f.ui.active=None;f.continuous(inner);}
                }
                if motion.origin.is_some() && (f.input.released||f.input.cancelled||f.input.blurred||!node.enabled||!f.enabled) {
                    if motion.dragging {f.ui.active=None;let cancelled=f.input.cancelled||f.input.blurred||!node.enabled||!f.enabled;let dir=if (motion.offset<0.0)^rtl {1}else{-1};let fling=motion.last.is_some_and(|(time,_)|now-time<0.1) && motion.offset.abs()>25.0 && motion.velocity.abs()>400.0 && motion.offset.signum()==motion.velocity.signum();if !cancelled && (motion.offset.abs()>width*0.4 || fling) && calendar.move_month(dir) {motion.offset+=if motion.offset<0.0 {width}else{-width};}if f.reduced_motion(){motion.offset=0.0;}else if motion.offset.abs()>0.01{let duration=(0.1*(1.0+f64::from(motion.offset.abs()/width))).min(0.6);motion.settling=Some((now,motion.offset,duration));f.continuous(inner);}}
                    motion.origin=None;motion.dragging=false;motion.rejected=false;
                }
                let offset=motion.offset;let interactive=!motion.dragging&&motion.settling.is_none();let mut page_change=0;let mut chosen=false;
                f.within(inner,|f| {
                    for dir in [-1_i8,0,1] {let mut page=calendar.clone();if dir!=0&&!page.move_month(dir){continue;}let side=if rtl {-f32::from(dir)}else{f32::from(dir)};let rect=Rect{x:inner.x+offset+side*width,..inner};if rect.x+rect.width<=inner.x||rect.x>=inner.x+inner.width{continue;}
                        if dir==0 {let (change,selected)=draw_calendar_page(f,calendar,rect,id,rtl,node.enabled&&f.enabled,interactive,&state.calendar_locale,today,m3,density);page_change=change;chosen=selected;}
                        else {draw_calendar_page(f,&mut page,rect,id,rtl,node.enabled&&f.enabled,false,&state.calendar_locale,today,m3,density);}
                    }
                });
                if chosen {action=Some(node.id);}
                if page_change!=0 && calendar.move_month(page_change) {if !f.reduced_motion(){motion.offset=if (page_change>0)^rtl {width}else{-width};motion.settling=Some((now,motion.offset,0.2));}f.continuous(inner);}

    action
}

pub(super) fn draw_xml_number_picker(node:&'static Node,f:&mut Frame<'_>,inner:Rect,state:&mut LayoutState,id:u64,key:String)->(Option<&'static str>,Vec<i32>) {
    let mut action=None;let mut changes=Vec::new();let t=f.theme();
                let picker=state.number_pickers.entry(key.clone()).or_insert_with(||crate::android_number_picker::NumberPickerState::new(0,0,0).unwrap());
                if state.number_picker_committed.remove(&key){action=Some(node.id);changes.push(picker.value());}
                let editing=state.number_picker_editing.contains(&key);
                let motion=state.number_picker_motion.entry(key.clone()).or_default();let row=(inner.height/3.0).max(1.0);let now=f.time();
                if !node.enabled||!f.enabled {motion.fling=None;motion.settle=None;motion.offset=0.0;}
                if let Some((start,from))=motion.settle {let p=((now-start)/0.2).clamp(0.0,1.0) as f32;motion.offset=from*(1.0-p).powi(3);if p>=1.0||f.reduced_motion(){motion.offset=0.0;motion.settle=None;}else{f.continuous(inner);}}
                if let Some(mut fling)=motion.fling.take() {let (delta,active)=fling.step(now);let (changed,boundary)=number_picker_move(picker,&mut motion.offset,delta,row,&mut changes);if changed {action=Some(node.id);}if active&&!boundary&&!f.reduced_motion(){motion.fling=Some(fling);f.continuous(inner);}else if f.reduced_motion(){motion.offset=0.0;}else if motion.offset.abs()>0.01{motion.settle=Some((now,motion.offset));f.continuous(inner);}}
                if node.enabled&&f.enabled&&f.input.pressed&&f.pointer_in(inner) {if let Some(p)=f.input.pointer {let slot=((p[1]-inner.y)/row).floor() as i32;motion.hold=match slot {0=>Some((-1,now+0.5,false)),2=>Some((1,now+0.5,false)),_=>None};if let Some((_,deadline,_))=motion.hold{f.wake_at(deadline);}if f.input.pointer_kind==PointerKind::Touch {motion.origin=Some(p[1]);motion.last=p[1];motion.last_time=now;motion.velocity=0.0;motion.dragging=false;motion.settle=None;motion.fling=None;motion.offset=0.0;}}}
                if let (Some(origin),Some(pointer))=(motion.origin,f.input.pointer) {let y=pointer[1];if !motion.dragging&&(y-origin).abs()>8.0 {motion.dragging=true;motion.hold=None;f.ui.active=None;}if motion.dragging {let dt=now-motion.last_time;let delta=y-motion.last;if dt>0.001&&delta.abs()>0.01 {motion.velocity=(delta/dt as f32).clamp(-4000.0,4000.0);motion.last_time=now;}motion.last=y;let (changed,_)=number_picker_move(picker,&mut motion.offset,delta,row,&mut changes);if changed{action=Some(node.id);}f.ui.active=None;f.continuous(inner);}}
                if let Some((dir,mut deadline,mut fired))=motion.hold {if node.enabled&&f.enabled&&!f.input.cancelled&&!f.input.blurred && f.input.pointer.is_some_and(|p|inner.contains(p)&&if dir<0 {p[1]<inner.y+row}else{p[1]>=inner.y+2.0*row}) {let mut repeats=0;while now>=deadline&&repeats<8 {if picker.step(dir){changes.push(picker.value());action=Some(node.id);fired=true;f.ui.active=None;}deadline+=0.3;repeats+=1;}if now>=deadline{deadline=now+0.3;}motion.hold=Some((dir,deadline,fired));f.wake_at(deadline);}else{motion.hold=None;}}
                let was_dragging=motion.dragging;let was_held=motion.hold.is_some_and(|(_,_,fired)|fired);
                if f.input.released||f.input.cancelled||f.input.blurred||!node.enabled||!f.enabled {if was_held{f.ui.active=None;}motion.hold=None;if motion.origin.is_some() {if motion.dragging {f.ui.active=None;let can_fling=f.input.released&&!f.input.cancelled&&!f.input.blurred&&!f.reduced_motion()&&now-motion.last_time<0.1&&motion.velocity.abs()>=50.0;if can_fling {let mut fling=crate::scroll::SplineFling::default();fling.start(motion.velocity,now);motion.fling=Some(fling);f.continuous(inner);}else if f.reduced_motion(){motion.offset=0.0;}else{motion.settle=Some((now,motion.offset));f.continuous(inner);}}motion.origin=None;motion.dragging=false;}}
                let offset=motion.offset;let interactive=!editing&&!was_dragging&&!was_held&&motion.settle.is_none()&&motion.fling.is_none();
                f.within(inner,|f| {
                    for slot in 0..3 {let rect=Rect{x:inner.x,y:inner.y+slot as f32*row,width:inner.width,height:row};if slot!=1 {let dir=if slot==0{-1}else{1};let allowed=picker.neighbor(dir).is_some();let hit=f.target(anonymous_child_id(id,9600+slot),rect,if dir<0{"Decrease"}else{"Increase"},node.enabled&&interactive&&allowed,Role::Button);f.state_layer(hit,0.0,t.text);if hit.clicked&&picker.step(dir){changes.push(picker.value());action=Some(node.id);if !f.reduced_motion(){motion.offset=f32::from(dir)*row;motion.settle=Some((now,motion.offset));f.continuous(inner);}}}}
                    let center=Rect{x:inner.x,y:inner.y+row,width:inner.width,height:row};
                    if editing {f.target(id,center,"Number",node.enabled,Role::TextField);} else {let hit=f.target(id,center,"Edit value",node.enabled&&interactive,Role::Button);if hit.clicked {state.text.insert(key.clone(),if picker.has_displayed_values(){picker.label(picker.value()).unwrap_or_default()}else{state.calendar_locale.number(picker.value() as u32)});state.input_types.insert(key.clone(),if picker.has_displayed_values(){0}else{3});state.number_picker_editing.insert(key.clone());f.request_focus(id);f.continuous(inner);}}
                    for delta in -2_i8..=2 {if editing&&delta==0 {continue;}let Some(value)=picker.neighbor(delta)else{continue;};let rect=Rect{x:inner.x,y:inner.y+(1.0+f32::from(delta))*row+offset,width:inner.width,height:row};if rect.y+rect.height<=inner.y||rect.y>=inner.y+inner.height{continue;}let label=if picker.has_displayed_values(){picker.label(value).unwrap_or_default()}else{state.calendar_locale.number(value as u32)};let color=t.text.opacity(if node.enabled {if delta==0{1.0}else{0.5}}else{0.38});f.styled_text(rect,&label,color,TextStyle{size:20.0,align:1,..TextStyle::default()});}
                    let y1=inner.y+row;let y2=inner.y+row*2.0;f.paint_line([inner.x,y1],[inner.x+inner.width,y1],1.0,t.outline_variant);f.paint_line([inner.x,y2],[inner.x+inner.width,y2],1.0,t.outline_variant);
                });
    (action,changes)
}

pub(super) fn paint_android_search_icon(f:&mut Frame<'_>,bounds:Rect,icon:usize,color:Color) {
    let index=crate::beercss_shapes::SHAPES.len()+crate::icons::material_navigation::ICONS.len()+crate::icons::material_symbols::ICONS.len()+2+crate::shapes::android_clock_shapes::SHAPES.len()+crate::icons::android_time_picker::ICONS.len()+icon;
    f.commands.push(DrawCommand::Shape{bounds,index:index as u8,color,rotation:0.0});
}
pub(super) fn render_autocomplete_popup(f:&mut Frame<'_>,state:&mut LayoutState)->Option<&'static str> {
    let Some(anchor)=state.autocomplete_anchor.clone() else {state.autocomplete_open=None;state.autocomplete_panel=None;return None;};
    let text=state.text.get(&anchor.key).cloned().unwrap_or_default();
    if anchor.multi && !state.comma_tokenizers.contains(&anchor.key) {state.autocomplete_open=None;state.autocomplete_panel=None;return None;}
    // Android positions selections in UTF-16. Clamp malformed offsets to a character boundary.
    let selection=state.text_selection_end.get(&anchor.key).copied().unwrap_or_else(||text.encode_utf16().count());
    let mut units=0;let mut token_end=0;
    for (i,ch) in text.char_indices() {let next=units+ch.len_utf16();if next>selection {break;}units=next;token_end=i+ch.len_utf8();}
    let token_start=if anchor.multi {let base=text[..token_end].rfind(',').map_or(0,|i|i+1);base+text[base..token_end].len()-text[base..token_end].trim_start_matches(' ').len()}else{0};
    let query=if anchor.multi {text[token_start..token_end].to_owned()}else if anchor.search {
        if state.search_suggestion_preview.get(&anchor.key).is_some_and(|(_,preview)|preview!=&text) {state.search_suggestion_preview.remove(&anchor.key);}
        state.search_suggestion_preview.get(&anchor.key).map_or_else(||text.clone(),|(original,_)|original.clone())
    }else{text.clone()};
    let focused=f.focused_id()==Some(anchor.id);
    if query.encode_utf16().count()<anchor.threshold {state.autocomplete_open=None;state.autocomplete_panel=None;state.autocomplete_highlight.remove(&anchor.key);return None;}
    let Some(options)=state.autocomplete_items.get(&anchor.key) else {state.autocomplete_open=None;state.autocomplete_panel=None;return None;};
    let matches=state.autocomplete_matches.entry(anchor.key.clone()).or_insert_with(||AutoMatches {query:None,indices:Vec::new()});
    if matches.query.as_deref()!=Some(query.as_str()) {
        let lower=query.to_lowercase();matches.indices=options.iter().enumerate().filter_map(|(i,value)|{let value=value.to_lowercase();(value.starts_with(&lower)||value.split(' ').any(|word|word.starts_with(&lower))).then_some(i)}).collect();matches.query=Some(query.clone());state.autocomplete_highlight.remove(&anchor.key);state.autocomplete_scroll.remove(&anchor.key);
    }
    if matches.indices.is_empty() {state.autocomplete_open=None;state.autocomplete_panel=None;return None;}
    if focused && state.autocomplete_dismissed.get(&anchor.key)!=Some(&query) {state.autocomplete_open=Some(anchor.key.clone());}
    if state.autocomplete_open.as_deref()!=Some(anchor.key.as_str()) {state.autocomplete_panel=None;return None;}
    let viewport=f.viewport();let width=match anchor.width {"match_parent"|"wrap_content"=>anchor.bounds.width,v=>v.parse::<f32>().unwrap_or(anchor.bounds.width)}.clamp(0.0,viewport.width.max(0.0));
    let desired=match anchor.height {"match_parent"=>viewport.height,"wrap_content"=>matches.indices.len() as f32*48.0,v=>v.parse::<f32>().unwrap_or(viewport.height)}.min(viewport.height).max(0.0);
    let below_start=(anchor.bounds.y+anchor.bounds.height+anchor.offset_y).clamp(viewport.y,viewport.y+viewport.height);let above_end=(anchor.bounds.y+anchor.offset_y).clamp(viewport.y,viewport.y+viewport.height);
    let below=viewport.y+viewport.height-below_start;let above=above_end-viewport.y;let use_below=below>=desired || below>=above;
    let height=desired.min(if use_below{below}else{above});let x=if anchor.rtl {anchor.bounds.x+anchor.bounds.width-width-anchor.offset_x}else{anchor.bounds.x+anchor.offset_x};
    let panel=Rect{x:x.clamp(viewport.x,viewport.x+(viewport.width-width).max(0.0)),y:if use_below{below_start}else{above_end-height},width,height};
    state.autocomplete_panel=Some(panel);
    if f.pressed_key(Key::Escape) || f.input.pressed && !f.pointer_in(panel) && !f.pointer_in(anchor.bounds) {
        state.autocomplete_dismissed.insert(anchor.key.clone(),query);state.autocomplete_open=None;state.autocomplete_panel=None;return None;
    }
    let max=(matches.indices.len() as f32*48.0-panel.height).max(0.0);let offset=state.autocomplete_scroll.entry(anchor.key.clone()).or_default();
    let dragging=f.scroll_viewport(anonymous_child_id(anchor.id,8190),panel,offset,max);let scroll=*offset;
    let start=(scroll.max(0.0)/48.0).floor() as usize;let end=(((scroll+panel.height).max(0.0)/48.0).ceil() as usize).min(matches.indices.len());
    let highlighted=state.autocomplete_highlight.get(&anchor.key).copied();let mut choice=None;
    f.elevation(panel,4.0,2);f.paint_rect(panel,4.0,f.theme().surface_highest);
    f.without_input(dragging,|f|f.within(panel,|f|for position in start..end {
        let index=matches.indices[position];let bounds=Rect{x:panel.x,y:panel.y+position as f32*48.0-scroll,width:panel.width,height:48.0};
        let response=f.target(anonymous_child_id(anchor.id,8192+index),bounds,&options[index],true,Role::Button);
        if highlighted==Some(position) {f.paint_rect(bounds,0.0,f.theme().primary_container);}f.state_layer(response,0.0,f.theme().text);
        f.styled_text(bounds,&options[index],f.theme().text,TextStyle {padding:16.0,..TextStyle::default()});if response.clicked {choice=Some(index);}
    }));
    if f.pressed_key(Key::Down)||f.pressed_key(Key::Up) {
        let next=if f.pressed_key(Key::Down){highlighted.map_or(0,|i|(i+1).min(matches.indices.len()-1))}else{highlighted.unwrap_or(matches.indices.len()).saturating_sub(1)};
        state.autocomplete_highlight.insert(anchor.key.clone(),next);let top=next as f32*48.0;if top<*offset {*offset=top;}else if top+48.0>*offset+panel.height {*offset=(top+48.0-panel.height).min(max);}f.continuous(panel);
        if anchor.search {let preview=options[matches.indices[next]].clone();state.search_suggestion_preview.insert(anchor.key.clone(),(query.clone(),preview.clone()));state.text.insert(anchor.key.clone(),preview);state.search_pending.insert(anchor.key.clone());}
    }
    if f.pressed_key(Key::Enter) && let Some(position)=highlighted {choice=matches.indices.get(position).copied();}
    if let Some(index)=choice {if anchor.search {state.search_suggestion_clicked.insert(anchor.key.clone(),index);state.search_suggestion_preview.remove(&anchor.key);state.autocomplete_dismissed.insert(anchor.key.clone(),text);state.autocomplete_open=None;state.autocomplete_panel=None;return Some(anchor.xml_id);}let selected=options[index].clone();let result=if anchor.multi {let terminated=if selected.trim_end_matches(' ').ends_with(',') {selected.clone()}else{format!("{selected}, ")};let mut result=text.clone();result.replace_range(token_start..token_end,&terminated);state.text_selection_end.insert(anchor.key.clone(),result[..token_start+terminated.len()].encode_utf16().count());result}else{state.text_selection_end.insert(anchor.key.clone(),selected.encode_utf16().count());selected.clone()};state.text.insert(anchor.key.clone(),result);state.autocomplete_selected.insert(anchor.key.clone(),index);state.autocomplete_dismissed.insert(anchor.key.clone(),selected);state.autocomplete_open=None;state.autocomplete_panel=None;f.request_focus(anchor.id);return Some(anchor.xml_id);}
    None
}
pub(super) fn render_til_dropdown_popup(f: &mut Frame<'_>, state: &mut LayoutState) -> Option<&'static str> {
    let Some(anchor) = state.til_dropdown_anchor.clone() else {
        state.til_dropdown_panel = None;
        return None;
    };
    if state.til_dropdown_open.as_deref() != Some(anchor.key.as_str()) {
        state.til_dropdown_panel = None;
        return None;
    }
    let options = state
        .entries
        .get(&anchor.key)
        .or_else(|| state.entries.get(&anchor.til_key))
        .or_else(|| state.autocomplete_items.get(&anchor.key))
        .or_else(|| state.spinner_items.get(&anchor.key))
        .cloned()
        .unwrap_or_default();
    if options.is_empty() {
        state.til_dropdown_open = None;
        state.til_dropdown_panel = None;
        return None;
    }
    let d = state.effective_density();
    let row_h = state.dp(48.0);
    let radius = state.dp(4.0);
    let viewport = f.viewport();
    let width = anchor.bounds.width.clamp(state.dp(112.0).min(viewport.width), viewport.width.max(0.0));
    let desired = (options.len() as f32 * row_h).min(viewport.height.max(0.0));
    let below_start = (anchor.bounds.y + anchor.bounds.height).clamp(viewport.y, viewport.y + viewport.height);
    let above_end = anchor.bounds.y.clamp(viewport.y, viewport.y + viewport.height);
    let below = (viewport.y + viewport.height - below_start).max(0.0);
    let above = (above_end - viewport.y).max(0.0);
    let use_below = below >= desired || below >= above;
    let height = desired.min(if use_below { below } else { above });
    let x = anchor.bounds.x.clamp(viewport.x, viewport.x + (viewport.width - width).max(0.0));
    let y = if use_below { below_start } else { above_end - height };
    let panel = Rect { x, y, width, height };
    state.til_dropdown_panel = Some(panel);
    if f.pressed_key(Key::Escape)
        || (f.input.pressed && !f.pointer_in(panel) && !f.pointer_in(anchor.bounds))
    {
        state.til_dropdown_open = None;
        state.til_dropdown_panel = None;
        return None;
    }
    let current = state.text.get(&anchor.key).map(String::as_str);
    let t = f.theme();
    paint_elevation_level(f, panel, radius, 2, d);
    f.paint_rect(panel, radius, t.surface_high);
    let mut chosen: Option<usize> = None;
    let style = TextStyle {
        size: state.sp(14.0),
        padding: state.dp(16.0),
        align: if anchor.rtl { 2 } else { 0 },
        ..TextStyle::default()
    };
    f.within(panel, |f| {
        for (idx, item) in options.iter().enumerate() {
            let row = Rect {
                x: panel.x,
                y: panel.y + idx as f32 * row_h,
                width: panel.width,
                height: row_h,
            };
            if row.y >= panel.y + panel.height {
                break;
            }
            let r = f.target(anonymous_child_id(anchor.id, 9860 + idx), row, item, true, Role::Button);
            if current == Some(item.as_str()) {
                f.paint_rect(row, 0.0, t.secondary_container);
            }
            f.state_layer(r, 0.0, t.text);
            f.styled_text(
                row,
                item,
                if current == Some(item.as_str()) { t.on_secondary_container } else { t.text },
                style,
            );
            if r.clicked {
                chosen = Some(idx);
            }
        }
    });
    if let Some(idx) = chosen {
        let selected = options[idx].clone();
        state.text.insert(anchor.key.clone(), selected);
        state.selected_index.insert(anchor.key.clone(), idx);
        state.selected_index.insert(anchor.til_key.clone(), idx);
        state.til_dropdown_open = None;
        state.til_dropdown_panel = None;
        return Some(anchor.xml_id);
    }
    None
}
pub(super) fn child_view_id(parent:u64, index:usize, child:&Node) -> u64 {
    if child.id.is_empty() { anonymous_child_id(parent,index) } else { view_id(child.id) }
}

// Distribute extra space to child constraints, rather than to track minima:
// an unweighted sibling can already satisfy part of a weighted child's size.
pub(super) fn grid_solve_constraints(count:usize,constraints:&[(usize,usize,f32,f32)],delta:f32,total:f32)->Vec<f32> {
    let mut lines=vec![0.0_f32;count+1];
    for start in 0..count {
        lines[start+1]=lines[start+1].max(lines[start]);
        for &(from,span,wanted,weight) in constraints.iter().filter(|item|item.0==start) {
            let extra=if total>0.0 {delta*weight/total}else{0.0};
            lines[from+span]=lines[from+span].max(lines[from]+wanted+extra);
        }
    }
    lines.windows(2).map(|pair|pair[1]-pair[0]).collect()
}
pub(super) fn grid_distribute(tracks:&mut Vec<f32>,constraints:&[(usize,usize,f32,f32)],available:f32) {
    let total=constraints.iter().map(|item|item.3).sum::<f32>();
    if total<=0.0 || !total.is_finite() || tracks.iter().sum::<f32>()>=available {return;}
    let solve=|delta:f32| grid_solve_constraints(tracks.len(),constraints,delta,total);
    let mut low=0.0_f32;let mut high=available.max(1.0);
    while solve(high).iter().sum::<f32>()<available && high<1.0e9 {high*=2.0;}
    for _ in 0..32 {let mid=(low+high)*0.5;if solve(mid).iter().sum::<f32>()<=available {low=mid;}else{high=mid;}}
    *tracks=solve(low);
}
