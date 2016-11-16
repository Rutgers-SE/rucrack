pub trait WrappedInc {
    fn inc(self) -> Self;
    fn dec(self) -> Self;
}

pub trait WrappedStep {
    fn step(self, &Self) -> Self;
    fn back(self, &Self) -> Self;
}

impl WrappedStep for u8 {
    fn step(self, other: &Self) -> Self {
        let mut counter: Self = 0;
        let mut output = self.clone();
        while counter < other.clone() {
            output = output.inc();
            counter += 1;
        }
        output
    }
    fn back(self, other: &Self) -> Self {
        unimplemented!();
    }
}

impl WrappedInc for u8 {
    fn inc(self) -> Self {
        match self {
            255u8 => 0u8,
            _ => self + 1u8,
        }
    }

    fn dec(self) -> Self {
        match self {
            0u8 => 255u8,
            _ => self - 1,
        }
    }
}
