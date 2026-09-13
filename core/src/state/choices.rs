#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pass {
    Exam,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sitting {
    Written,
    Copypaste,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    Ok,
    Partial,
    Miss,
}

pub(super) const PASS: [(&str, Pass); 2] = [("exam", Pass::Exam), ("skip", Pass::Skip)];

pub(super) const SITTING: [(&str, Sitting); 2] = [
    ("written", Sitting::Written),
    ("copypaste", Sitting::Copypaste),
];

pub(super) const GRADE: [(&str, Grade); 3] = [
    ("ok", Grade::Ok),
    ("partial", Grade::Partial),
    ("miss", Grade::Miss),
];

impl Pass {
    pub fn label(self) -> &'static str {
        label(&PASS, self)
    }
}

impl Sitting {
    pub fn label(self) -> &'static str {
        label(&SITTING, self)
    }
}

impl Grade {
    pub fn label(self) -> &'static str {
        label(&GRADE, self)
    }
}

fn label<T: Copy + PartialEq>(options: &[(&'static str, T)], value: T) -> &'static str {
    options
        .iter()
        .find(|(_, known)| *known == value)
        .map_or("", |(name, _)| name)
}
