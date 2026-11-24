//! Value and range model for Android's three-row NumberPicker selector wheel.

/// Caller-owned NumberPicker value. View animation and text editing are separate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumberPickerState { min:i32, max:i32, value:i32, wrap_preferred:bool, displayed_values:Option<Vec<String>> }
impl NumberPickerState {
    pub fn new(min:i32,max:i32,value:i32)->Result<Self,&'static str> {
        if min<0 || max<min{return Err("NumberPicker requires 0 <= min <= max");}
        Ok(Self{min,max,value:value.clamp(min,max),wrap_preferred:true,displayed_values:None})
    }
    pub fn has_displayed_values(&self)->bool{self.displayed_values.is_some()}
    pub fn min(&self)->i32{self.min} pub fn max(&self)->i32{self.max} pub fn value(&self)->i32{self.value}
    /// Android enables wrapping only when max-min is at least three.
    pub fn wraps(&self)->bool{self.wrap_preferred && i64::from(self.max)-i64::from(self.min)>=3}
    pub fn set_wrap_preferred(&mut self,enabled:bool){self.wrap_preferred=enabled;}
    /// Range and optional labels change together; invalid input leaves state unchanged.
    pub fn set_range(&mut self,min:i32,max:i32,labels:Option<Vec<String>>)->Result<(),&'static str>{
        if min<0 || max<min{return Err("NumberPicker requires 0 <= min <= max");}
        let len=(i64::from(max)-i64::from(min)+1) as usize;
        if labels.as_ref().is_some_and(|items|items.len()!=len){return Err("displayed values must cover the range");}
        self.min=min;self.max=max;self.value=self.value.clamp(min,max);self.displayed_values=labels;Ok(())
    }
    pub fn set_value(&mut self,value:i32)->bool {
        let value=if self.wraps(){let len=i64::from(self.max)-i64::from(self.min)+1;(i64::from(self.min)+(i64::from(value)-i64::from(self.min)).rem_euclid(len)) as i32}else{value.clamp(self.min,self.max)};
        let changed=value!=self.value;self.value=value;changed
    }
    pub fn step(&mut self,direction:i8)->bool {if !matches!(direction,-1|1){return false;}let next=i64::from(self.value)+i64::from(direction);if next>i64::from(self.max) {return if self.wraps(){self.set_value(self.min)}else{false};}if next<i64::from(self.min) {return if self.wraps(){self.set_value(self.max)}else{false};}self.set_value(next as i32)}
    /// Neighbor displayed in a row, or None beyond a nonwrapping endpoint.
    pub fn neighbor(&self,delta:i8)->Option<i32>{let value=i64::from(self.value)+i64::from(delta);let min=i64::from(self.min);let max=i64::from(self.max);if self.wraps(){Some((min+(value-min).rem_euclid(max-min+1)) as i32)}else if value>=min&&value<=max{Some(value as i32)}else{None}}
    pub fn label(&self,value:i32)->Option<String>{if value<self.min||value>self.max{return None;}Some(self.displayed_values.as_ref().map_or_else(||value.to_string(),|labels|labels[(value-self.min) as usize].clone()))}
    /// Returns the candidate for set_value: empty text restores the old value;
    /// unmatched text falls back to the minimum as Android does.
    pub fn parse_input(&self,input:&str)->Option<i32>{self.parse_input_localized(input,&["0","1","2","3","4","5","6","7","8","9"].map(str::to_owned))}
    /// Accept the host locale's decimal digits as well as ASCII digits.
    pub fn parse_input_localized(&self,input:&str,digits:&[String;10])->Option<i32>{
        let input=input.trim();if input.is_empty(){return None;}
        if let Some(labels)=&self.displayed_values {let lower=input.to_lowercase();if let Some(i)=labels.iter().position(|label|label.to_lowercase().starts_with(&lower)){return Some(self.min+i as i32);}}
        Some(parse_localized_decimal(input,digits).unwrap_or(self.min))
    }
}

/// Parse an integer written using locale digits or ASCII. Invalid input is rejected.
pub fn parse_localized_decimal(input:&str,digits:&[String;10])->Option<i32>{
    let mut remaining=input.trim();if remaining.is_empty(){return None;}let mut number=String::new();
    if let Some(rest)=remaining.strip_prefix('-'){number.push('-');remaining=rest;}else if let Some(rest)=remaining.strip_prefix('+'){number.push('+');remaining=rest;}
    while !remaining.is_empty(){let Some((value,digit))=digits.iter().enumerate().filter(|(_,d)|!d.is_empty()&&remaining.starts_with(d.as_str())).max_by_key(|(_,d)|d.len()) else {if let Some(c)=remaining.chars().next().filter(|c|c.is_ascii_digit()){number.push(c);remaining=&remaining[c.len_utf8()..];continue;}return None;};number.push((b'0'+value as u8) as char);remaining=&remaining[digit.len()..];}
    number.parse().ok()
}

#[cfg(test)]
#[path = "number_picker.spec.rs"]
mod tests;
