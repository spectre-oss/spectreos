//! State for Android's deprecated DialerFilter. See Android 15 DialerFilter.java.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum DialerMode {
    #[default] DigitsAndLetters,
    DigitsAndLettersNoDigits,
    DigitsAndLettersNoLetters,
    DigitsOnly,
    LettersOnly,
}
#[derive(Clone,Debug,Default)]
pub struct DialerFilterState { pub mode:DialerMode, pub letters:String, pub digits:String }
impl DialerFilterState {
    pub fn set_mode(&mut self,mode:DialerMode){self.mode=mode;}
    pub fn letters(&self)->&str {if matches!(self.mode,DialerMode::DigitsAndLetters|DialerMode::DigitsAndLettersNoDigits|DialerMode::LettersOnly){&self.letters}else{""}}
    pub fn digits(&self)->&str {if matches!(self.mode,DialerMode::DigitsAndLetters|DialerMode::DigitsAndLettersNoLetters|DialerMode::DigitsOnly){&self.digits}else{""}}
    pub fn filter_text(&self)->&str {if self.mode==DialerMode::DigitsOnly{self.digits()}else{self.letters()}}
    /// Append like Android's programmatic append (not key filtering).
    pub fn append(&mut self,text:&str){let value=text.to_uppercase();match self.mode {DialerMode::DigitsAndLetters=>{self.digits.push_str(&value);self.letters.push_str(&value);},DialerMode::DigitsAndLettersNoLetters|DialerMode::DigitsOnly=>self.digits.push_str(&value),_=>self.letters.push_str(&value)}}
    /// Handle a printable QWERTY character, following the platform's mode change.
    pub fn type_char(&mut self,c:char){if c.is_control(){return;}let accepted=c.is_ascii_digit()||matches!(c,'*'|'#'|'+'|'-'|'('|')'|' '|'.'|','|'/'|';'|'N'|'n');let value=c.to_uppercase().collect::<String>();match self.mode {DialerMode::DigitsAndLetters=>{self.letters.push_str(&value);if accepted{self.digits.push_str(&value);}else{self.mode=DialerMode::DigitsAndLettersNoDigits;}},DialerMode::DigitsAndLettersNoDigits|DialerMode::LettersOnly=>self.letters.push_str(&value),DialerMode::DigitsAndLettersNoLetters|DialerMode::DigitsOnly=>if accepted{self.digits.push_str(&value)}}}
    pub fn backspace(&mut self){match self.mode {DialerMode::DigitsAndLetters=>{self.digits.pop();self.letters.pop();},DialerMode::DigitsAndLettersNoDigits=>{self.letters.pop();if self.letters.chars().count()==self.digits.chars().count(){self.mode=DialerMode::DigitsAndLetters;}},DialerMode::DigitsAndLettersNoLetters=>{if self.letters.chars().count()==self.digits.chars().count(){self.letters.pop();self.mode=DialerMode::DigitsAndLetters;}self.digits.pop();},DialerMode::DigitsOnly=>{self.digits.pop();},DialerMode::LettersOnly=>{self.letters.pop();}}}
    /// Apply the host's complete edited value, including paste and deletion.
    pub fn replace_user_text(&mut self,text:&str){let mode=match self.mode {DialerMode::DigitsOnly=>DialerMode::DigitsOnly,DialerMode::LettersOnly=>DialerMode::LettersOnly,DialerMode::DigitsAndLettersNoLetters=>DialerMode::DigitsAndLettersNoLetters,_=>DialerMode::DigitsAndLetters};self.letters.clear();self.digits.clear();self.mode=mode;for ch in text.chars(){self.type_char(ch);}}
    pub fn clear(&mut self){self.digits.clear();self.letters.clear();self.mode=DialerMode::DigitsAndLetters;}
}

#[cfg(test)]
#[path = "dialer_filter.spec.rs"]
mod tests;
