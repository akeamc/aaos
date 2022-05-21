use crate::vga_buffer::{Buffer, CharColor, ScreenChar, WRITER};

pub fn write_bytes(
    buffer: &mut Buffer,
    row: usize,
    col: usize,
    bytes: impl AsRef<[u8]>,
    color: CharColor,
) {
    for (i, b) in bytes.as_ref().iter().enumerate() {
        buffer.chars[row][col + i].write(ScreenChar::new(*b, color))
    }
}

/// Prints a 34x23 logo with its top left corner at `(row, col)`.
pub fn print_logo(row: usize, col: usize) {
    const LOGO: &str = "  &#   *@@@@@@@@*                 
  @#         #@@@@@@@,            
.@@@*          ,@@@&       .**    
.@@@@@@#.        *,     #&*@@@@%  
.& /@@@@@@@@@@&#,       (%     .  
,@@/ *@@@@@@@@@@@@@@@@&@%         
.@@@@@#. ./#@@@@@@@@@@@@@@@@@(    
&@@@@@@@@&(,   ./#@@@@@@@@@@@@%   
  (@@@@@@@@@@@@@@@@/. ./&@@@@@@@( 
     .(%@@@@@@@@@@@@@@@&* .&@@@@@,
            .*#@@@@@@@@@@@( /@@@@.
              @%    *%@@@@& .@@@. 
            *@*        .@@, (@%   
   ..,*/((#@&,.        ,*  %%     
,@@@@@@@@@@@@@@@@@@@@@/     */    
#@%.       .@@,#@@@@@@@@@@@@,     
@,      .&@#        (@@@@*        
(@@@@/                            ";

    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();

        for (i, line) in LOGO.lines().enumerate() {
            write_bytes(
                writer.buffer,
                row + i,
                col,
                line,
                CharColor::white_on_black(),
            )
        }

        write_bytes(
            writer.buffer,
            row + 20,
            col + 5,
            b" S\x99DRA LATINS GYMNASIUM ",
            CharColor::black_on_white(),
        );

        write_bytes(
            writer.buffer,
            row + 22,
            col + 1,
            b"vetenskap \x07 kultur \x07 kreativitet",
            CharColor::white_on_black(),
        );
    });
}
