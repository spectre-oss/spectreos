//! Flattening and expansion state for Android ExpandableListView adapters.
use std::collections::{BTreeSet,HashSet};
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ExpandableChild {pub id:u64,pub label:String,pub selectable:bool}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ExpandableGroup {pub id:u64,pub label:String,pub children:Vec<ExpandableChild>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum FlatPosition {Group(usize),Child{group:usize,child:usize}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ExpandableEvent {Expanded(usize),Collapsed(usize),ChildClicked{group:usize,child:usize}}
/// Adapter callbacks, including automatic collapse when the expansion limit is reached.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ExpandableAdapterEvent {Expanded(usize),Collapsed(usize)}
/// Caller-owned adapter contents; expanded groups survive stable-ID updates.
#[derive(Clone,Debug,Default)]
pub struct ExpandableListState {groups:Vec<ExpandableGroup>,expanded:BTreeSet<usize>,tree:Vec<usize>,max_expanded:Option<usize>,revision:u64,adapter_events:Vec<ExpandableAdapterEvent>}
impl ExpandableListState {
    pub fn groups(&self)->&[ExpandableGroup]{&self.groups}
    pub fn revision(&self)->u64{self.revision}
    /// Drain adapter callbacks in framework order.
    pub fn take_adapter_events(&mut self)->Vec<ExpandableAdapterEvent>{std::mem::take(&mut self.adapter_events)}
    /// Replace data transactionally; group and child IDs must be unique within their scope.
    pub fn set_groups(&mut self,groups:Vec<ExpandableGroup>)->Result<(),&'static str>{
        let mut ids=HashSet::new();for g in &groups{if !ids.insert(g.id){return Err("duplicate group ID");}let mut children=HashSet::new();if g.children.iter().any(|c|!children.insert(c.id)){return Err("duplicate child ID");}}
        let previously:HashSet<u64>=self.expanded.iter().map(|i|self.groups[*i].id).collect();self.expanded=groups.iter().enumerate().filter_map(|(i,g)|previously.contains(&g.id).then_some(i)).collect();self.groups=groups;self.rebuild();self.revision=self.revision.wrapping_add(1);Ok(())
    }
    pub fn set_max_expanded(&mut self,max:Option<usize>){self.max_expanded=max;}
    pub fn is_expanded(&self,group:usize)->bool{self.expanded.contains(&group)}
    pub fn expand(&mut self,group:usize)->bool{if group>=self.groups.len()||self.is_expanded(group)||self.max_expanded==Some(0){return false;}if self.max_expanded.is_some_and(|max|self.expanded.len()>=max){if let Some(first)=self.expanded.first().copied(){self.expanded.remove(&first);self.add(first,-(self.groups[first].children.len() as i64));self.adapter_events.push(ExpandableAdapterEvent::Collapsed(first));}}self.expanded.insert(group);self.add(group,self.groups[group].children.len() as i64);self.adapter_events.push(ExpandableAdapterEvent::Expanded(group));self.revision=self.revision.wrapping_add(1);true}
    pub fn collapse(&mut self,group:usize)->bool{if !self.expanded.remove(&group){return false;}self.add(group,-(self.groups[group].children.len() as i64));self.adapter_events.push(ExpandableAdapterEvent::Collapsed(group));self.revision=self.revision.wrapping_add(1);true}
    pub fn visible_count(&self)->usize{self.prefix(self.groups.len())}
    pub fn flat_index(&self,position:FlatPosition)->Option<usize>{match position{FlatPosition::Group(g) if g<self.groups.len()=>Some(self.prefix(g)),FlatPosition::Child{group,child} if self.is_expanded(group)&&child<self.groups.get(group)?.children.len()=>Some(self.prefix(group)+1+child),_=>None}}
    pub fn position_at(&self,flat:usize)->Option<FlatPosition>{if flat>=self.visible_count(){return None;}let mut index=0usize;let mut sum=0usize;let mut bit=1usize;while bit<self.tree.len(){bit<<=1;}while bit>0 {let next=index+bit;if next<self.tree.len()&&sum+self.tree[next]<=flat{sum+=self.tree[next];index=next;}bit>>=1;}let group=index;if flat==sum {Some(FlatPosition::Group(group))}else{Some(FlatPosition::Child{group,child:flat-sum-1})}}
    fn rebuild(&mut self){self.tree=vec![0;self.groups.len()+1];for i in 0..self.groups.len(){self.add(i,1+if self.is_expanded(i){self.groups[i].children.len() as i64}else{0});}}
    fn add(&mut self,index:usize,delta:i64){let mut i=index+1;while i<self.tree.len(){self.tree[i]=(self.tree[i] as i64+delta) as usize;i+=i&i.wrapping_neg();}}
    fn prefix(&self,count:usize)->usize{let mut i=count;let mut sum=0;while i>0{sum+=self.tree[i];i&=i-1;}sum}
}

/// Destination of Android's bounded scroll after expanding a group. The group
/// remains visible even when its children cannot all fit in the viewport.
pub fn expansion_scroll_target(group_flat:usize, children:usize, row:f32, viewport:f32, offset:f32, total_rows:usize)->f32 {
    if row<=0.0 || viewport<=0.0 { return offset; }
    let group_top=group_flat as f32*row;
    let last_bottom=(group_flat.saturating_add(children).saturating_add(1)) as f32*row;
    let max=(total_rows as f32*row-viewport).max(0.0);
    (if group_top<offset {group_top} else {offset.max((last_bottom-viewport).min(group_top))}).clamp(0.0,max)
}

/// Android's position scroller uses a 200 ms decelerating scroll.
#[derive(Clone,Copy,Debug)]
pub struct ExpansionScroll {pub from:f32,pub to:f32,pub start:f64}
impl ExpansionScroll {
    pub fn sample(self,now:f64)->(f32,bool) {
        let progress=((now-self.start)/0.2).clamp(0.0,1.0) as f32;
        (self.from+(self.to-self.from)*(1.0-(1.0-progress).powi(2)),progress<1.0)
    }
}

#[cfg(test)]
#[path = "expandable_list.spec.rs"]
mod tests;
