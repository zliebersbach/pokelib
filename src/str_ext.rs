pub trait StrExt {
    fn capitalize(&self) -> String;
}

impl StrExt for str {
    fn capitalize(&self) -> String {
        let _tmp;
        if self.len() < 1 {
            _tmp = self.to_uppercase();
        } else {
            _tmp = format!("{}{}", self[0..1].to_uppercase(), &self[1..])
        }
        _tmp
    }
}
