//! Android TimePicker radial dial selection state.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum DialPart {Hour,Minute}
#[derive(Clone,Debug)]
pub struct TimeClockState {hour:u8,minute:u8,is_24_hour:bool,part:DialPart}
impl TimeClockState {
    pub fn new(hour:u8,minute:u8,is_24_hour:bool)->Result<Self,&'static str>{if hour>23||minute>59{return Err("invalid time");}Ok(Self{hour,minute,is_24_hour,part:DialPart::Hour})}
    pub fn hour(&self)->u8{self.hour}pub fn minute(&self)->u8{self.minute}pub fn is_24_hour(&self)->bool{self.is_24_hour}pub fn part(&self)->DialPart{self.part}
    pub fn set_part(&mut self,part:DialPart){self.part=part;}
    pub fn set_24_hour(&mut self,value:bool){self.is_24_hour=value;}
    pub fn set_input_hour(&mut self,value:u8,zero_based:bool)->Result<bool,&'static str>{let max=if self.is_24_hour{if zero_based{23}else{24}}else if zero_based{11}else{12};let min=if zero_based{0}else{1};if value<min||value>max{return Err("hour outside range");}let hour=if self.is_24_hour{value%24}else{value%12+if self.hour>=12{12}else{0}};let changed=hour!=self.hour;self.hour=hour;Ok(changed)}
    pub fn set_input_minute(&mut self,value:u8)->Result<bool,&'static str>{if value>59{return Err("minute outside range");}let changed=value!=self.minute;self.minute=value;Ok(changed)}
    pub fn set_period(&mut self,pm:bool)->bool{let next=self.hour%12+if pm{12}else{0};let changed=next!=self.hour;self.hour=next;changed}
    /// Select an angle clockwise from twelve. Inner chooses the inner 24-hour ring.
    /// Hour selection advances to minutes only on release, including an unchanged tap.
    pub fn select(&mut self,degrees:u16,inner:bool,release:bool)->bool {
        let degrees=i32::from(degrees.min(360));let old=(self.hour,self.minute);
        match self.part {
            DialPart::Hour=>{let snapped=((degrees+15)/30*30)%360;let h=(snapped/30) as u8;
                self.hour=if self.is_24_hour {if inner {if h==0{0}else{h+12}}else if h==0{12}else{h}}else{h+if self.hour>=12{12}else{0}};
                if release{self.part=DialPart::Minute;}
            },
            DialPart::Minute=>self.minute=((snap_prefer_30s(degrees)%360)/6) as u8,
        }
        old!=(self.hour,self.minute)
    }
}
/// Android weights each visible five-minute mark more heavily than other minutes.
fn snap_prefer_30s(degrees:i32)->i32 {let mut output=0;let mut count=1;let mut expected=8;for input in 0..=360 {if input==degrees{return output;}if count==expected{output+=6;expected=if output==360{7}else if output%30==0{14}else{4};count=1;}else{count+=1;}}360}

#[cfg(test)]
#[path = "time_clock.spec.rs"]
mod tests;
