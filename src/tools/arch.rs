pub enum Arch {
    X86_64,
    Aarch64,
    Armv7hl,
}

impl Arch {
    pub fn entries() -> Vec<String> {
        vec![Arch::X86_64.name(), Arch::Aarch64.name(), Arch::Armv7hl.name()]
    }

    pub fn name(&self) -> String {
        match self {
            Arch::X86_64 => "x86_64".into(),
            Arch::Aarch64 => "aarch64".into(),
            Arch::Armv7hl => "armv7hl".into(),
        }
    }
}
