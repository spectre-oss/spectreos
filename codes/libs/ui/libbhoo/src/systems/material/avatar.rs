use crate::*;

/// Avatar outline shape.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarShape { #[default] Circle, Rounded, Square }

/// Caller-owned avatar content. Hosts decide when an image has loaded or failed.
#[derive(Clone, Copy, Debug)]
pub enum AvatarContent<'a> { Image(&'a Image), Initials(&'a str), Fallback }

/// Configuration for an overlapping avatar group.
#[derive(Clone, Copy, Debug)]
pub struct AvatarGroupOptions {
    /// Maximum visible slots, including a surplus badge; clamped to at least two.
    pub max: usize,
    /// Total people, including those not present in the supplied items.
    pub total: Option<usize>,
    /// Logical-pixel overlap of neighboring bordered avatars.
    pub overlap: f32,
    /// Content diameter or side length before the 2 dp outline.
    pub size: f32,
    pub shape: AvatarShape,
    pub rtl: bool,
}
impl Default for AvatarGroupOptions {
    fn default() -> Self { Self {max:5,total:None,overlap:8.0,size:40.0,shape:AvatarShape::Circle,rtl:false} }
}

/// Number of supplied avatars shown and people represented by the surplus badge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvatarGroupCounts { pub shown: usize, pub surplus: usize }

impl Frame<'_> {
    /// Paint an image, initials or the Material person fallback inside an avatar shape.
    /// The caller owns image loading and chooses fallback content on failure.
    pub fn avatar_at(&mut self, bounds: Rect, shape: AvatarShape, content: AvatarContent<'_>) {
        let size=bounds.width.min(bounds.height);
        if !size.is_finite() || size<=0.0 {return;}
        let b=Rect{x:bounds.x+(bounds.width-size)*0.5,y:bounds.y+(bounds.height-size)*0.5,width:size,height:size};
        let radius=match shape {AvatarShape::Circle=>size*0.5,AvatarShape::Rounded=>4.0_f32.min(size*0.5),AvatarShape::Square=>0.0};
        let theme=self.theme();
        self.paint_rect(b,radius,theme.secondary_container);
        match content {
            AvatarContent::Image(image)=>{self.paint_image(b,image,ImageFit::Cover,radius);},
            AvatarContent::Initials(text) if !text.is_empty()=>self.styled_text(b,text,theme.on_secondary_container,TextStyle{size:size*0.5,align:1,..TextStyle::default()}),
            AvatarContent::Initials(_) | AvatarContent::Fallback=>{
                let inset=size*0.125;
                self.paint_material_icon(b.inset(inset),MaterialIcon::Person,theme.on_secondary_container);
            },
        }
    }
    /// Draw a group using MUI's max/total counting and caller-owned avatar content.
    /// Only visible members generate drawing commands, even for a very large group.
    pub fn avatar_group_with_options(&mut self, items:&[AvatarContent<'_>], options:AvatarGroupOptions) -> AvatarGroupCounts {
        let size=if options.size.is_finite() {options.size.clamp(1.0,256.0)}else{40.0};
        let outer=size+4.0;
        let overlap=if options.overlap.is_finite() {options.overlap.clamp(0.0,outer-1.0)}else{8.0_f32.min(outer-1.0)};
        let step=outer-overlap;
        let total=options.total.filter(|&value|value>0).unwrap_or(items.len());
        let max=options.max.max(2);
        let max=if total==max {max.saturating_add(1)}else{max}.min(total.saturating_add(1));
        let shown=items.len().min(max.saturating_sub(1));
        let surplus=total.saturating_sub(max).max(total.saturating_sub(shown));
        let count=shown.saturating_add(usize::from(surplus>0));
        let row=self.reserve(outer);
        let clip=row.intersect(self.clip);
        if count==0 || clip.width<=0.0 || clip.height<=0.0 {return AvatarGroupCounts{shown,surplus};}
        let origin=if options.rtl {row.x+row.width-outer}else{row.x};
        let (low,high)=if options.rtl {
            ((origin-(clip.x+clip.width))/step,(origin+outer-clip.x)/step)
        } else {((clip.x-origin-outer)/step,(clip.x+clip.width-origin)/step)};
        let first=(low.floor().max(0.0) as usize).saturating_sub(1).min(count);
        let last=((high.ceil().max(0.0) as usize).saturating_add(1)).min(count);
        let radius=match options.shape {AvatarShape::Circle=>outer*0.5,AvatarShape::Rounded=>6.0_f32.min(outer*0.5),AvatarShape::Square=>0.0};
        for index in (first..last).rev() {
            let b=Rect{x:origin+if options.rtl{-(index as f32)*step}else{index as f32*step},y:row.y,width:outer,height:outer};
            if b.intersect(clip).width<=0.0 {continue;}
            self.paint_rect(b,radius,self.theme().background);
            if index==shown {
                let label=format!("+{surplus}");
                self.avatar_at(b.inset(2.0),options.shape,AvatarContent::Initials(&label));
            } else {self.avatar_at(b.inset(2.0),options.shape,items[index]);}
        }
        AvatarGroupCounts{shown,surplus}
    }
    /// Draw overlapping initial avatars and an optional surplus count.
    pub fn avatar_group(&mut self, labels: &[&str], visible: usize) {
        let row = self.reserve(44.0);
        let visible = visible.clamp(1, 8);
        let shown = labels.len().min(visible);
        let surplus = labels.len().saturating_sub(shown);
        let count = shown + usize::from(surplus > 0);
        for i in (0..count).rev() {
            let b = Rect {
                x: row.x + i as f32 * 32.0,
                y: row.y + 2.0,
                width: 40.0,
                height: 40.0,
            };
            self.paint_rect(b, 20.0, self.ui.theme.background);
            self.paint_rect(b.inset(2.0), 18.0, self.ui.theme.secondary);
            let text = if i == shown {
                format!("+{surplus}")
            } else {
                labels[i].to_owned()
            };
            self.styled_text(
                b,
                &text,
                self.ui.theme.on_secondary,
                TextStyle {
                    size: 14.0,
                    align: 1,
                    ..TextStyle::default()
                },
            );
        }
    }
}

#[cfg(test)]
#[path = "avatar.spec.rs"]
mod tests;
