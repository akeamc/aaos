/// 8x16 IBM VGA font. [source](https://int10h.org/oldschool-pc-fonts/fontlist/font?ibm_bios-2y)
pub static IBM_BIOS: Font<'static> = Font::from_dos_f16(include_bytes!("./BIOS_D.F16"));

pub struct Font<'a> {
    pub size: u16,
    pub height: u8,
    pub data: &'a [u8],
}

impl<'a> Font<'a> {
    pub const fn from_dos_f16(data: &'a [u8]) -> Self {
        Self {
            size: 256,
            height: 16,
            data,
        }
    }
}
