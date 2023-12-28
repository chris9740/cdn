use cdn::Cdn;

pub mod cache;
pub mod cdn;
pub mod rest;
pub mod storage;

#[macro_use]
pub mod macros;

pub mod colors {
    pub const RED: (u8, u8, u8) = (212, 63, 80);
    pub const GREEN: (u8, u8, u8) = (63, 212, 99);
    pub const MAGENTA: (u8, u8, u8) = (204, 45, 191);
}
