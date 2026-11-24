//! Android TimePicker spinner-mode time arithmetic.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum TimeSpinnerField {Hour,Minute,Period}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct TimeSpinnerRange {pub min:i32,pub max:i32,pub value:i32,pub wrap:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum HourCycle {H11,H12,H23,H24}
#[derive(Clone,Debug)]
pub struct TimeSpinnerState {hour:u8,minute:u8,is_24_hour:bool,cycle12:HourCycle,cycle24:HourCycle}
impl TimeSpinnerState {
    pub fn new(hour:u8,minute:u8,is_24_hour:bool)->Result<Self,&'static str>{if hour>23||minute>59{return Err("invalid time");}Ok(Self{hour,minute,is_24_hour,cycle12:HourCycle::H12,cycle24:HourCycle::H23})}
    pub fn hour(&self)->u8{self.hour}pub fn minute(&self)->u8{self.minute}pub fn is_24_hour(&self)->bool{self.is_24_hour}
    pub fn set_24_hour(&mut self,value:bool){self.is_24_hour=value;}
    pub fn set_hour_cycles(&mut self,cycle12:HourCycle,cycle24:HourCycle)->Result<(),&'static str>{if !matches!(cycle12,HourCycle::H11|HourCycle::H12)||!matches!(cycle24,HourCycle::H23|HourCycle::H24){return Err("hour cycle does not match mode");}self.cycle12=cycle12;self.cycle24=cycle24;Ok(())}
    pub fn hour_cycle(&self)->HourCycle{if self.is_24_hour{self.cycle24}else{self.cycle12}}
    pub fn range(&self,field:TimeSpinnerField)->TimeSpinnerRange {match field {
        TimeSpinnerField::Hour=>match self.hour_cycle(){
            HourCycle::H11=>TimeSpinnerRange{min:0,max:11,value:i32::from(self.hour%12),wrap:true},
            HourCycle::H12=>TimeSpinnerRange{min:1,max:12,value:i32::from(if self.hour%12==0{12}else{self.hour%12}),wrap:true},
            HourCycle::H23=>TimeSpinnerRange{min:0,max:23,value:i32::from(self.hour),wrap:true},
            HourCycle::H24=>TimeSpinnerRange{min:1,max:24,value:i32::from(if self.hour==0{24}else{self.hour}),wrap:true},
        },
        TimeSpinnerField::Minute=>TimeSpinnerRange{min:0,max:59,value:i32::from(self.minute),wrap:true},
        TimeSpinnerField::Period=>TimeSpinnerRange{min:0,max:1,value:i32::from(self.hour>=12),wrap:false},
    }}
    pub fn set_field(&mut self,field:TimeSpinnerField,value:i32)->Result<bool,&'static str>{
        let r=self.range(field);if value<r.min||value>r.max{return Err("time spinner value outside range");}if value==r.value{return Ok(false);}
        match field {
            TimeSpinnerField::Hour if self.is_24_hour=>self.hour=value as u8%24,
            TimeSpinnerField::Hour=>{let mut pm=self.hour>=12;let boundary=if self.hour_cycle()==HourCycle::H11{0}else{12};if (r.value==11&&value==boundary)||(r.value==boundary&&value==11){pm=!pm;}self.hour=(value as u8%12)+if pm{12}else{0};},
            TimeSpinnerField::Minute=>{if r.value==59&&value==0 {self.hour=(self.hour+1)%24;}else if r.value==0&&value==59 {self.hour=(self.hour+23)%24;}self.minute=value as u8;},
            TimeSpinnerField::Period=>self.hour=(self.hour%12)+if value==1{12}else{0},
        }Ok(true)
    }
}

#[cfg(test)]
#[path = "time_spinner.spec.rs"]
mod tests;
