//! DatePicker spinner-mode date arithmetic, based on Android's spinner delegate.
use crate::android_calendar::{CalendarDate,days_in_month};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum DateSpinnerField { Day, Month, Year }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct SpinnerRange {pub min:i32,pub max:i32,pub value:i32,pub wrap_preferred:bool}
/// Date and constraints for the day/month/year NumberPickers.
/// Months are one-based here; Android's public DatePicker callbacks use zero-based months.
#[derive(Clone,Debug)]
pub struct DateSpinnerState {min:CalendarDate,max:CalendarDate,date:CalendarDate}
impl DateSpinnerState {
    pub fn new(min:CalendarDate,max:CalendarDate,date:CalendarDate)->Result<Self,&'static str> {
        if [min,max,date].into_iter().any(|d|CalendarDate::new(d.year,d.month,d.day)!=Some(d)){return Err("invalid date");}if min>max{return Err("maxDate must be >= minDate");}Ok(Self {min,max,date:date.clamp(min,max)})
    }
    pub fn date(&self)->CalendarDate {self.date}
    pub fn bounds(&self)->(CalendarDate,CalendarDate){(self.min,self.max)}
    pub fn set_date(&mut self,date:CalendarDate)->Result<bool,&'static str>{if CalendarDate::new(date.year,date.month,date.day)!=Some(date){return Err("invalid date");}let date=date.clamp(self.min,self.max);let changed=self.date!=date;self.date=date;Ok(changed)}
    /// Mirrors the spinner delegate: endpoint constraints apply at the exact min or max date.
    pub fn range(&self,field:DateSpinnerField)->SpinnerRange {
        let d=self.date;let at_min=d==self.min;let at_max=d==self.max;
        match field {
            DateSpinnerField::Day=>SpinnerRange {min:if at_min{i32::from(d.day)}else{1},max:if at_max{i32::from(d.day)}else{i32::from(days_in_month(d.year,d.month))},value:i32::from(d.day),wrap_preferred:!at_min&&!at_max},
            DateSpinnerField::Month=>SpinnerRange {min:if at_min{i32::from(d.month)}else{1},max:if at_max{i32::from(d.month)}else{12},value:i32::from(d.month),wrap_preferred:!at_min&&!at_max},
            DateSpinnerField::Year=>SpinnerRange {min:self.min.year,max:self.max.year,value:d.year,wrap_preferred:false},
        }
    }
    /// Apply an individual NumberPicker's new value, including Android's day/month carry.
    pub fn set_field(&mut self,field:DateSpinnerField,value:i32)->Result<bool,&'static str>{
        let range=self.range(field);if value<range.min||value>range.max{return Err("spinner value outside current range");}
        let old=self.date;if value==range.value{return Ok(false);}
        let new=match field {
            DateSpinnerField::Day=>{
                let end=days_in_month(old.year,old.month);
                if old.day==end && value==1 {shift_day(old,1)}else if old.day==1 && value==i32::from(end){shift_day(old,-1)}else{Some(CalendarDate{day:value as u8,..old})}
            },
            DateSpinnerField::Month=>{
                let delta=if old.month==12 && value==1 {1}else if old.month==1 && value==12 {-1}else{value-i32::from(old.month)};
                shift_month(old,delta)
            },
            DateSpinnerField::Year=>{
                // Calendar.set(YEAR, ...) is lenient: Feb 29 in a common year rolls to Mar 1.
                if old.month==2&&old.day==29&&days_in_month(value,2)==28 {CalendarDate::new(value,3,1)}else{CalendarDate::new(value,old.month,old.day)}
            },
        };
        let new=new.unwrap_or(if value>range.value{self.max}else{self.min}).clamp(self.min,self.max);self.date=new;Ok(new!=old)
    }
}
fn shift_day(d:CalendarDate,delta:i8)->Option<CalendarDate>{
    if delta>0 {if d.day<days_in_month(d.year,d.month){CalendarDate::new(d.year,d.month,d.day+1)}else{shift_month(CalendarDate{day:1,..d},1)}}
    else if d.day>1 {CalendarDate::new(d.year,d.month,d.day-1)}else{let previous=shift_month(d,-1)?;CalendarDate::new(previous.year,previous.month,days_in_month(previous.year,previous.month))}
}
fn shift_month(d:CalendarDate,delta:i32)->Option<CalendarDate>{
    let n=d.year*12+i32::from(d.month)-1+delta;let year=n.div_euclid(12);let month=(n.rem_euclid(12)+1) as u8;CalendarDate::new(year,month,d.day.min(days_in_month(year,month)))
}

#[cfg(test)]
#[path = "date_spinner.spec.rs"]
mod tests;
