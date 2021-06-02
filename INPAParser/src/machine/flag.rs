
#[derive(Copy, Clone, Default)]
pub struct Flag(u32);

impl std::fmt::Debug for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Flag")
            .field("Carry", &(self.0 & 0x01 != 0))
            .field("Zero", &(self.0 & 0x02 != 0))
            .field("Sign", &(self.0 & 0x04 != 0))
            .field("Overflow", &(self.0 & 0x08 != 0))
        .finish()
    }
}

impl Flag {

    #[inline(always)]
    pub fn set_carry_bit(&mut self, state: bool) {
        self.0 = (self.0 & !0x01) | state as u32;
    }

    #[inline(always)]
    pub fn set_zero_bit(&mut self, state: bool) {
        self.0 = (self.0 & !0x02) | (state as u32) << 1;
    }

    #[inline(always)]
    pub fn set_sign_bit(&mut self, state: bool) {
        self.0 = (self.0 & !0x04) | (state as u32) << 2;
    }

    #[inline(always)]
    pub fn set_overflow_bit(&mut self, state: bool) {
        self.0 = (self.0 & !0x08) | (state as u32) << 3;
    }

    pub fn update_flags(&mut self, value: u32, length: u32) -> super::Result<()> {
        let masks: (u32, u32); // Value, Sign
        match length {
            1 => masks = (0x000000FF, 0x00000080),
            2 => masks = (0x0000FFFF, 0x00008000),
            4 => masks = (0xFFFFFFFF, 0x80000000),
            _ => return Err(super::EdiabasError::InvalidDataLength)
        }
        self.set_zero_bit(value & masks.0 == 0);
        self.set_sign_bit(value & masks.0 == 0);
        Ok(())
    }

    pub fn set_overflow(&mut self, v1: u32, v2: u32, res: u32, length: u32) -> super::Result<()> {
        let sign_mask: u32;
        match length {
            1 => sign_mask = 0x00000080,
            2 => sign_mask = 0x00008000,
            4 => sign_mask = 0x80000000,
            _ => return Err(super::EdiabasError::InvalidDataLength)
        }
        if (v1 & sign_mask) != (v2 & sign_mask) {
            self.set_overflow_bit(false);
        } else if (v1 & sign_mask) == (res & sign_mask) {
            self.set_overflow_bit(false);
        } else {
            self.set_overflow_bit(true);
        }
        Ok(())
    }

    pub fn set_carry(&mut self, v: u64, length: u32) -> super::Result<()> {
        let carry_mask: u64;
        match length {
            1 => carry_mask = 0x00000100,
            2 => carry_mask = 0x00001000,
            4 => carry_mask = 0x10000000,
            _ => return Err(super::EdiabasError::InvalidDataLength)
        }
        self.set_carry_bit((v & carry_mask) != 0);
        Ok(())
    }

    pub fn to_value(&self) -> u32 {
        self.0
    }

    pub fn from_value(v: u32) -> Self {
        Self(v)
    }
}