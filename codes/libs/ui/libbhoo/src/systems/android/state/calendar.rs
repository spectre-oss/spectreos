//! Date and navigation state for Android's CalendarView month pager.
//! Dates are proleptic Gregorian civil days; the host supplies local dates.

/// A validated civil date, using Android's one-based month numbering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CalendarDate { pub year: i32, pub month: u8, pub day: u8 }
impl CalendarDate {
    pub fn new(year:i32,month:u8,day:u8)->Option<Self> {
        if !(1..=9999).contains(&year) || !(1..=12).contains(&month) || day==0 || day>days_in_month(year,month) {None} else {Some(Self {year,month,day})}
    }
    /// Android Calendar.DAY_OF_WEEK: Sunday=1 through Saturday=7.
    pub fn weekday(self)->u8 {
        let mut y=i64::from(self.year);let m=i64::from(self.month);y-=i64::from(m<=2);
        let era=y.div_euclid(400);let yoe=y-era*400;let mp=m+if m>2{-3}else{9};
        let doy=(153*mp+2)/5+i64::from(self.day)-1;let doe=yoe*365+yoe/4-yoe/100+doy;
        ((era*146097+doe-719468+4).rem_euclid(7)+1) as u8
    }
}
pub fn days_in_month(year:i32,month:u8)->u8 {
    match month {1|3|5|7|8|10|12=>31,4|6|9|11=>30,2=>if year%4==0 && (year%100!=0 || year%400==0) {29}else{28},_=>0}
}

/// One month cell; dates outside the permitted range are visible but disabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DayCell { pub date:CalendarDate, pub enabled:bool, pub selected:bool }

/// Caller-owned model of the platform calendar month pager.
#[derive(Clone, Debug)]
pub struct CalendarState {
    min:CalendarDate, max:CalendarDate, selected:CalendarDate,
    visible_year:i32, visible_month:u8, first_day_of_week:u8,
}
impl CalendarState {
    /// Range and selected date must be valid. The selection is clamped to the range.
    pub fn new(min:CalendarDate,max:CalendarDate,selected:CalendarDate,first_day_of_week:u8)->Result<Self,&'static str> {
        if CalendarDate::new(min.year,min.month,min.day)!=Some(min) || CalendarDate::new(max.year,max.month,max.day)!=Some(max) || CalendarDate::new(selected.year,selected.month,selected.day)!=Some(selected) {return Err("invalid calendar date");}
        if min>max {return Err("maxDate must be >= minDate");}if !(1..=7).contains(&first_day_of_week) {return Err("invalid first day of week");}
        let selected=selected.clamp(min,max);Ok(Self{min,max,selected,visible_year:selected.year,visible_month:selected.month,first_day_of_week})
    }
    pub fn selected(&self)->CalendarDate {self.selected}
    pub fn range(&self)->(CalendarDate,CalendarDate) {(self.min,self.max)}
    pub fn visible_month(&self)->(i32,u8) {(self.visible_year,self.visible_month)}
    pub fn first_day_of_week(&self)->u8 {self.first_day_of_week}
    pub fn set_first_day_of_week(&mut self,day:u8)->Result<(),&'static str> {if !(1..=7).contains(&day){return Err("invalid first day of week");}self.first_day_of_week=day;Ok(())}
    /// Change bounds transactionally, clamping selection and returning to its month.
    pub fn set_range(&mut self,min:CalendarDate,max:CalendarDate)->Result<(),&'static str> {
        if CalendarDate::new(min.year,min.month,min.day)!=Some(min)||CalendarDate::new(max.year,max.month,max.day)!=Some(max){return Err("invalid calendar date");}if min>max{return Err("maxDate must be >= minDate");}
        self.min=min;self.max=max;self.set_date(self.selected);Ok(())
    }
    /// Programmatic setDate clamps to the range and moves to the selected month.
    pub fn set_date(&mut self,date:CalendarDate)->bool {
        if CalendarDate::new(date.year,date.month,date.day)!=Some(date){return false;}let date=date.clamp(self.min,self.max);self.selected=date;self.visible_year=date.year;self.visible_month=date.month;true
    }
    /// User taps can select only enabled dates in the visible month.
    pub fn select_day(&mut self,day:u8)->bool {
        let Some(date)=CalendarDate::new(self.visible_year,self.visible_month,day) else{return false;};if date<self.min||date>self.max{return false;}self.selected=date;true
    }
    /// Move one month without changing the selected date.
    pub fn move_month(&mut self,direction:i8)->bool {
        if !matches!(direction,-1|1){return false;}let index=self.visible_year*12+i32::from(self.visible_month)-1+i32::from(direction);
        let y=index.div_euclid(12);let m=(index.rem_euclid(12)+1) as u8;
        if !(1..=9999).contains(&y) || (y,m)<(self.min.year,self.min.month) || (y,m)>(self.max.year,self.max.month){return false;}
        self.visible_year=y;self.visible_month=m;true
    }
    /// Six rows of seven cells, in locale week order; empty cells are not dates.
    pub fn cells(&self)->[Option<DayCell>;42] {
        let mut cells=[None;42];let first=CalendarDate {year:self.visible_year,month:self.visible_month,day:1};
        let offset=(i32::from(first.weekday())-i32::from(self.first_day_of_week)).rem_euclid(7) as usize;
        for day in 1..=days_in_month(self.visible_year,self.visible_month) {let date=CalendarDate{day,..first};cells[offset+usize::from(day)-1]=Some(DayCell{date,enabled:date>=self.min&&date<=self.max,selected:date==self.selected});}cells
    }
}

/// Host-provided Gregorian calendar labels and locale week convention.
#[derive(Clone, Debug)]
pub struct CalendarLocale {
    pub months:[String;12],
    /// Sunday through Saturday, matching Android Calendar.DAY_OF_WEEK.
    pub weekdays:[String;7],
    pub short_months:[String;12],
    pub short_weekdays:[String;7],
    /// Localized weekday/month/day order for the DatePicker header.
    pub date_header:String,
    /// Locale order of month (M), day (D) and year (Y) wheels.
    pub date_order:[char;3],
    pub day_periods:[String;2],
    pub period_first:bool,
    pub hour_cycle_12:crate::android_time_spinner::HourCycle,
    pub hour_cycle_24:crate::android_time_spinner::HourCycle,
    pub hour_padded_12:bool,
    pub hour_padded_24:bool,
    pub time_separator:String,
    pub digits:[String;10],
    /// A template containing {month} and {year} in locale order.
    pub month_year:String,
    pub first_day_of_week:u8,
    /// Locale layout direction, used by hosts as the inherited XML direction.
    pub rtl:bool,
}
impl Default for CalendarLocale {
    fn default()->Self {Self {
        months:["January","February","March","April","May","June","July","August","September","October","November","December"].map(str::to_owned),
        weekdays:["S","M","T","W","T","F","S"].map(str::to_owned),
        short_months:["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"].map(str::to_owned),
        short_weekdays:["Sun","Mon","Tue","Wed","Thu","Fri","Sat"].map(str::to_owned),
        date_header:"{weekday}, {month} {day}".into(),date_order:['M','D','Y'],day_periods:["AM".into(),"PM".into()],period_first:false,hour_cycle_12:crate::android_time_spinner::HourCycle::H12,hour_cycle_24:crate::android_time_spinner::HourCycle::H23,hour_padded_12:false,hour_padded_24:false,time_separator:":".into(),
        digits:["0","1","2","3","4","5","6","7","8","9"].map(str::to_owned),
        month_year:"{month} {year}".into(),first_day_of_week:1,rtl:false,
    }}
}
impl CalendarLocale {
    pub fn number(&self,n:u32)->String {n.to_string().bytes().map(|c|self.digits[usize::from(c-b'0')].as_str()).collect()}
    pub fn title(&self,year:i32,month:u8)->String {self.month_year.replace("{month}",&self.months[usize::from(month-1)]).replace("{year}",&self.number(year as u32))}
    pub fn date_label(&self,date:CalendarDate)->String {self.date_header.replace("{weekday}",&self.short_weekdays[usize::from(date.weekday()-1)]).replace("{month}",&self.short_months[usize::from(date.month-1)]).replace("{day}",&self.number(u32::from(date.day)))}
    /// Decode the browser host's newline-separated UTF-8 locale snapshot transactionally.
    pub fn from_host(value:&str)->Option<Self> {
        let mut fields=value.split('\n');let first_day_of_week=fields.next()?.parse().ok()?;if !(1..=7).contains(&first_day_of_week){return None;}
        let rtl=match fields.next()? {"rtl"=>true,"ltr"=>false,_=>return None};
        let month_year=fields.next()?.to_owned();if !month_year.contains("{month}")||!month_year.contains("{year}"){return None;}
        let date_header=fields.next()?.to_owned();if !["{weekday}","{month}","{day}"].iter().all(|part|date_header.contains(part)){return None;}
        let order:Vec<char>=fields.next()?.chars().collect();if order.len()!=3 || !['M','D','Y'].iter().all(|c|order.contains(c)){return None;}let date_order=order.try_into().ok()?;
        let period_first=match fields.next()? {"first"=>true,"last"=>false,_=>return None};
        use crate::android_time_spinner::HourCycle;
        let hour_cycle_12=match fields.next()? {"h11"=>HourCycle::H11,"h12"=>HourCycle::H12,_=>return None};
        let hour_cycle_24=match fields.next()? {"h23"=>HourCycle::H23,"h24"=>HourCycle::H24,_=>return None};
        let hour_padded_12=match fields.next()? {"1"=>true,"0"=>false,_=>return None};let hour_padded_24=match fields.next()? {"1"=>true,"0"=>false,_=>return None};
        let time_separator=fields.next()?.to_owned();
        let labels:Vec<String>=fields.map(str::to_owned).collect();if labels.len()!=50 || labels.iter().any(|s|s.is_empty()){return None;}
        let months=labels[0..12].to_vec().try_into().ok()?;let weekdays=labels[12..19].to_vec().try_into().ok()?;let digits=labels[19..29].to_vec().try_into().ok()?;let short_months=labels[29..41].to_vec().try_into().ok()?;let short_weekdays=labels[41..48].to_vec().try_into().ok()?;let day_periods=labels[48..50].to_vec().try_into().ok()?;
        Some(Self{months,weekdays,digits,month_year,first_day_of_week,rtl,date_header,short_months,short_weekdays,date_order,day_periods,period_first,hour_cycle_12,hour_cycle_24,hour_padded_12,hour_padded_24,time_separator})
    }
}

#[cfg(test)]
#[path = "calendar.spec.rs"]
mod tests;
