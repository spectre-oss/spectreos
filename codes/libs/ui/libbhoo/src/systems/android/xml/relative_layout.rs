//! Initial Android RelativeLayout rule resolution. The rule ordering and gone
//! anchor traversal follow Android 15 RelativeLayout (Apache-2.0).
use super::*;
use std::collections::HashMap;

// Absolute rules 0..7, relative horizontal rules 8..11, parent rules 12..17,
// center rules 18..20, align-with-parent-if-missing 21, baseline 22, and container ignore-gravity target 23.
fn rule(node:Node, kind:u16, rtl:bool) -> Option<&'static str> {
    let get=|k|node.relative_rules.iter().find(|r|r.kind==k).map(|r|r.anchor);
    if kind<=1 && (get(8).is_some() || get(9).is_some()) {
        return get(match (kind,rtl) {(0,false)|(1,true)=>8,_=>9});
    }
    if (4..=5).contains(&kind) && (get(10).is_some() || get(11).is_some()) {
        return get(match (kind,rtl) {(4,false)|(5,true)=>10,_=>11});
    }
    if (12..=13).contains(&kind) && (get(16).is_some() || get(17).is_some()) {
        return get(match (kind,rtl) {(12,false)|(13,true)=>16,_=>17});
    }
    get(kind)
}
struct Axis<'a> {
    nodes:&'a [Node], ids:HashMap<&'static str,usize>, state:&'a LayoutState,
    parent:u64, origin:f32, extent:f32, center_origin:f32, center_extent:f32, horizontal:bool, wrap_parent:bool,
    widths:Option<&'a [(f32,f32)]>, metrics:TextMetrics,
    result:Vec<Option<(f32,f32)>>, visiting:Vec<bool>,
}
impl Axis<'_> {
    fn anchor(&self, node:Node, kind:u16) -> Option<usize> {
        let mut name=rule(node,kind,self.state.rtl)?;
        for _ in 0..self.nodes.len() {
            let index=*self.ids.get(name)?;
            let found=self.nodes[index];
            if found.effective_visibility(self.state)!=Visibility::Gone { return Some(index); }
            name=rule(found,kind,self.state.rtl)?;
        }
        None
    }
    fn related(&mut self, node:Node, kind:u16) -> Option<(f32,f32,[f32;4])> {
        let index=self.anchor(node,kind)?;
        if self.visiting[index] { return None; }
        let (pos,size)=self.position(index);
        Some((pos,size,self.nodes[index].resolved_margin(self.state)))
    }
    fn position(&mut self, index:usize) -> (f32,f32) {
        if let Some(value)=self.result[index] { return value; }
        if self.visiting[index] { return (self.origin,0.0); }
        self.visiting[index]=true;
        let n=self.nodes[index];let m=n.resolved_margin(self.state);
        let (before,after)=if self.horizontal {(m[0],m[2])} else {(m[1],m[3])};
        let (low_rule,high_rule,align_low,align_high,parent_low,parent_high,center)=if self.horizontal {(1,0,4,5,12,13,18)} else {(3,2,6,7,14,15,19)};
        let (anchor_after,anchor_before)=if self.horizontal {(2,0)} else {(3,1)};
        let fallback=rule(n,21,self.state.rtl).is_some();
        let baseline=if !self.horizontal { self.anchor(n,22).and_then(|anchor| {
            if self.visiting[anchor] { return None; }
            let (pos,height)=self.position(anchor);
            let width=self.widths.map_or(0.0,|widths|widths[anchor].1);
            self.nodes[anchor].relative_baseline(width,height,self.state,self.metrics,child_view_id(self.parent,anchor,&self.nodes[anchor])).map(|offset|pos+offset)
        }) } else { None };
        let rtl_h = self.horizontal && self.state.rtl;
        let allow_parent_low = !self.wrap_parent || !rtl_h;
        let allow_parent_high = !self.wrap_parent || rtl_h;
        let mut low=None;let mut high=None;
        if rule(n,high_rule,self.state.rtl).is_some() {
            if let Some((pos,_,margin))=self.related(n,high_rule) { high=Some(pos-margin[anchor_before]-after); }
            else if fallback && allow_parent_high { high=Some(self.origin+self.extent-after); }
        }
        if rule(n,low_rule,self.state.rtl).is_some() {
            if let Some((pos,size,margin))=self.related(n,low_rule) { low=Some(pos+size+margin[anchor_after]+before); }
            else if fallback && allow_parent_low { low=Some(self.origin+before); }
        }
        if rule(n,align_low,self.state.rtl).is_some() {
            if let Some((pos,_,_))=self.related(n,align_low) { low=Some(pos+before); }
            else if fallback && allow_parent_low { low=Some(self.origin+before); }
        }
        if rule(n,align_high,self.state.rtl).is_some() {
            if let Some((pos,size,_))=self.related(n,align_high) { high=Some(pos+size-after); }
            else if fallback && allow_parent_high { high=Some(self.origin+self.extent-after); }
        }
        if allow_parent_low && rule(n,parent_low,self.state.rtl).is_some() { low=Some(self.origin+before); }
        if allow_parent_high && rule(n,parent_high,self.state.rtl).is_some() { high=Some(self.origin+self.extent-after); }
        if baseline.is_some() {low=None;high=None;}
        let available_raw=high.unwrap_or(self.origin+self.extent-after)-low.unwrap_or(self.origin+before);
        let available=available_raw.max(0.0);
        let spec=if self.horizontal { n.width } else { n.height };
        let natural=if self.horizontal { n.layout_intrinsic(true,self.state,available,self.metrics,child_view_id(self.parent,index,&n)) } else {
            let width=self.widths.map_or(0.0,|widths|widths[index].1);
            n.layout_intrinsic(false,self.state,width,self.metrics,child_view_id(self.parent,index,&n))
        };
        let size=if low.is_some() && high.is_some() { available } else { match spec {
            Size::Dp(value)=>{let value=self.state.dp(value);if available_raw<0.0 {value} else {value.min(available)}},
            Size::Wrap=>if available_raw<0.0 {natural} else {natural.min(available)},
            Size::Match=>if self.wrap_parent { natural.min(available) } else { available },
        }};
        let pos=if let Some(target)=baseline {
            let width=self.widths.map_or(0.0,|widths|widths[index].1);
            target-n.relative_baseline(width,size,self.state,self.metrics,child_view_id(self.parent,index,&n)).unwrap_or(0.0)
        } else if let Some(value)=low { value } else if let Some(value)=high { value-size }
            else if !self.wrap_parent && (rule(n,center,self.state.rtl).is_some() || rule(n,20,self.state.rtl).is_some()) { self.center_origin+(self.center_extent-size)*0.5 }
            else if rtl_h { self.origin+self.extent-after-size }
            else { self.origin+before };
        let result=(pos,size);self.visiting[index]=false;self.result[index]=Some(result);result
    }
}
fn axis(nodes:&[Node],parent:u64,bounds:Rect,padding:[f32;4],state:&LayoutState,metrics:TextMetrics,horizontal:bool,wrap_parent:bool,widths:Option<&[(f32,f32)]>) -> Vec<(f32,f32)> {
    let mut axis=Axis {nodes,ids:nodes.iter().enumerate().filter(|(_,n)|!n.id.is_empty()).map(|(i,n)|(n.id,i)).collect(),state,parent,
        origin:if horizontal {bounds.x} else {bounds.y},extent:if horizontal {bounds.width} else {bounds.height},center_origin:if horizontal {bounds.x-padding[0]} else {bounds.y-padding[1]},center_extent:if horizontal {bounds.width+padding[0]+padding[2]} else {bounds.height+padding[1]+padding[3]},horizontal,wrap_parent,widths,metrics,result:vec![None;nodes.len()],visiting:vec![false;nodes.len()]};
    for i in 0..nodes.len() { if nodes[i].effective_visibility(state)!=Visibility::Gone { axis.position(i); } }
    axis.result.into_iter().map(|x|x.unwrap_or((axis.origin,0.0))).collect()
}
pub(super) fn layout(node:&Node, parent:u64, bounds:Rect, state:&LayoutState, metrics:TextMetrics, wrap_width:bool, wrap_height:bool) -> Vec<Rect> {
    let padding=node.resolved_padding(state);
    let widths=axis(node.children,parent,bounds,padding,state,metrics,true,wrap_width,None);
    let heights=axis(node.children,parent,bounds,padding,state,metrics,false,wrap_height,Some(&widths));
    let mut rects:Vec<Rect>=widths.into_iter().zip(heights).map(|((x,width),(y,height))|Rect {x,y,width,height}).collect();
    // Android applies container gravity to the union of child bounds, including margins.
    let horizontal=node.content_gravity[0]; let vertical=node.content_gravity[1];
    if horizontal!=0 || vertical!=0 {
        let ignore=rule(*node,23,state.rtl);
        let mut left=f32::INFINITY; let mut top=f32::INFINITY;
        let mut right=f32::NEG_INFINITY; let mut bottom=f32::NEG_INFINITY;
        for (child,rect) in node.children.iter().zip(&rects) {
            if child.effective_visibility(state)==Visibility::Gone {continue;}
            let m=child.resolved_margin(state);
            let item=(rect.x-m[0],rect.y-m[1],rect.x+rect.width+m[2],rect.y+rect.height+m[3]);
            let ignored=ignore.is_some_and(|id|id==child.id);
            // Match RelativeLayout's asymmetric bounds when only one axis has gravity.
            if !ignored || vertical!=0 {left=left.min(item.0);top=top.min(item.1);}
            if !ignored || horizontal!=0 {right=right.max(item.2);bottom=bottom.max(item.3);}
        }
        if left.is_finite() && right.is_finite() && top.is_finite() && bottom.is_finite() {
            let desired_x=match horizontal {1=>bounds.x+(bounds.width-(right-left))*0.5, 2=>if state.rtl {bounds.x} else {bounds.x+bounds.width-(right-left)}, 3=>bounds.x, 4=>bounds.x+bounds.width-(right-left), _=>left};
            let desired_y=match vertical {1=>bounds.y+(bounds.height-(bottom-top))*0.5, 2=>bounds.y+bounds.height-(bottom-top), _=>top};
            let dx=if wrap_width {0.0} else {desired_x-left};let dy=if wrap_height {0.0} else {desired_y-top};
            for (child,rect) in node.children.iter().zip(&mut rects) {
                if child.effective_visibility(state)!=Visibility::Gone && !ignore.is_some_and(|id|id==child.id) {rect.x+=dx;rect.y+=dy;}
            }
        }
    }
    rects
}
