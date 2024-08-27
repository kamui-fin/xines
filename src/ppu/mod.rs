#[cfg(target_family = "wasm")]
pub mod emscripten;
pub mod memory;

use memory::VRAM;

type RGB = (u8, u8, u8);

const MASTER_PALETTE: [RGB; 0x40] = [
    (98, 98, 98),
    (1, 32, 144),
    (36, 11, 160),
    (71, 0, 144),
    (96, 0, 98),
    (106, 0, 36),
    (96, 17, 0),
    (71, 39, 0),
    (36, 60, 0),
    (1, 74, 0),
    (0, 79, 0),
    (0, 71, 36),
    (0, 54, 98),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (171, 171, 171),
    (31, 86, 225),
    (77, 57, 255),
    (126, 35, 239),
    (163, 27, 183),
    (180, 34, 100),
    (172, 55, 14),
    (140, 85, 0),
    (94, 114, 0),
    (45, 136, 0),
    (7, 144, 0),
    (0, 137, 71),
    (0, 115, 157),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (255, 255, 255),
    (103, 172, 255),
    (149, 141, 255),
    (200, 117, 255),
    (242, 106, 255),
    (255, 111, 197),
    (255, 131, 106),
    (230, 160, 31),
    (184, 191, 0),
    (133, 216, 1),
    (91, 227, 53),
    (69, 222, 136),
    (73, 202, 227),
    (78, 78, 78),
    (0, 0, 0),
    (0, 0, 0),
    (255, 255, 255),
    (191, 224, 255),
    (209, 211, 255),
    (230, 201, 255),
    (247, 195, 255),
    (255, 196, 238),
    (255, 203, 201),
    (247, 215, 169),
    (230, 227, 151),
    (209, 238, 151),
    (191, 243, 169),
    (181, 242, 201),
    (181, 235, 238),
    (184, 184, 184),
    (0, 0, 0),
    (0, 0, 0),
];
pub struct PatternTable {
    pub tile_map: [[u8; 16]; 256],
}

pub struct NesColor {
    native_codes: u8,
    rgbs: RGB,
}

pub enum PatternTableType {
    Background,
    Sprite,
}

impl PatternTable {
    pub fn from_memory(ptype: PatternTableType, vram: &VRAM) -> Self {
        let mut tile_map = [[0; 16]; 256];
        let mut last_tile_pos = 0x0000;
        let mem_range = match ptype {
            PatternTableType::Background => 256..512,
            PatternTableType::Sprite => 0..256,
        };
        for k in mem_range {
            let tile = &vram.buffer[last_tile_pos..(last_tile_pos + 16)];
            tile_map[k].copy_from_slice(tile);
            // for i in 0..8 {
            //     for j in 0..8 {
            //         let first_bit = (tile[i].reverse_bits() >> j) & 1;
            //         let second_bit = (tile[i + 8].reverse_bits() >> j) & 1;
            //         let color_index = (second_bit << 1) | first_bit;
            //         tile_map[k][i][j] = color_index;
            //     }
            // }
            last_tile_pos = last_tile_pos + 16;
        }
        Self { tile_map }
    }
}

// NES 256x240
// 960 bytes (32 x 30 tiles) + 64 bytes AT
pub struct Nametable {
    table_2d: Vec<Vec<u8>>,
    attr: Vec<Vec<u8>>,
}

pub enum NametableArrangement {
    HorizontalMirror,
    VerticalMirror,
}

pub enum Quadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Nametable {
    pub fn new(quad: Quadrant, mem: &VRAM) -> Self {
        let starting_addr = match quad {
            Quadrant::TopLeft => 0x2000,
            Quadrant::TopRight => 0x2400,
            Quadrant::BottomLeft => 0x2800,
            Quadrant::BottomRight => 0x2C00,
        };
        // Load nametable from $2000
        let table = &mem.buffer[starting_addr..(starting_addr + 960)];

        let num_rows = 30;
        let num_cols = 32;

        // Each byte in nametable: Index into PT
        let mut table_2d = Vec::with_capacity(num_rows);
        for row in 0..num_rows {
            let start = row * num_cols;
            let end = start + num_cols;
            table_2d.push(table[start..end].to_vec());
        }

        let attribute_table = &mem.buffer[(starting_addr + 960)..(starting_addr + 960 + 64)];
        let mut attr: Vec<Vec<u8>> = vec![vec![0; num_cols]; num_rows];
        for (index, block_attr) in attribute_table.iter().enumerate() {
            let i = index / 8 * 4;
            let j = index % 8 * 4;

            // Each byte in AT tells you the pallete number:
            // Quad 0 is controlled by Bits 0,1
            // Quad 1 is controlled by Bits 2,3
            // Quad 2 is controlled by Bits 4,5
            // Quad 3 is controlled by Bits 6,7
            let quad_0_palette = block_attr & 0b11;
            let quad_1_palette = (block_attr & 0b1100) >> 2;
            let quad_2_palette = (block_attr & 0b110000) >> 4;
            let quad_3_palette = (block_attr & 0b11000000) >> 6;

            attr[i][j] = quad_0_palette;
            attr[i][j + 1] = quad_0_palette;
            attr[i + 1][j] = quad_0_palette;
            attr[i + 1][j + 1] = quad_0_palette;

            attr[i][j + 2] = quad_1_palette;
            attr[i][j + 3] = quad_1_palette;
            attr[i + 1][j + 2] = quad_1_palette;
            attr[i + 1][j + 3] = quad_1_palette;

            attr[i + 2][j] = quad_2_palette;
            attr[i + 2][j + 1] = quad_2_palette;
            attr[i + 3][j] = quad_2_palette;
            attr[i + 3][j + 1] = quad_2_palette;

            attr[i + 2][j + 2] = quad_3_palette;
            attr[i + 2][j + 3] = quad_3_palette;
            attr[i + 3][j + 2] = quad_3_palette;
            attr[i + 3][j + 3] = quad_3_palette;
        }

        Self { attr, table_2d }
    }
}

pub struct Palette {
    /// Address	Purpose
    /// $3F00	Universal background color
    /// $3F01-$3F03	Background palette 0
    /// $3F04	Normally unused color 1
    /// $3F05-$3F07	Background palette 1
    /// $3F08	Normally unused color 2
    /// $3F09-$3F0B	Background palette 2
    /// $3F0C	Normally unused color 3
    /// $3F0D-$3F0F	Background palette 3
    /// $3F10	Mirror of universal background color
    /// $3F11-$3F13	Sprite palette 0
    /// $3F14	Mirror of unused color 1
    /// $3F15-$3F17	Sprite palette 1
    /// $3F18	Mirror of unused color 2
    /// $3F19-$3F1B	Sprite palette 2
    /// $3F1C	Mirror of unused color 3
    /// $3F1D-$3F1F	Sprite palette 3
    palette_index: PaletteIndex,
    starting_addr: usize,
}

pub enum PaletteIndex {
    Bg(u8),
    Sprite(u8),
}

impl Palette {
    // Pallete::sync_with_memory(&mem) -> populate internal state
    // Input: bg/fg (ENUM), number
    // create the table mapping from 1 byte hex to rgb
    // store the 4 colors
    // get() convenience method for getting the RGB color

    pub fn new(p_idx: PaletteIndex) -> Self {
        let starting_addr = match p_idx {
            PaletteIndex::Bg(table_num) => match table_num {
                0 => 0x3F01,
                1 => 0x3F05,
                2 => 0x3F09,
                3 => 0x3F0C,
                _ => 0,
            },
            PaletteIndex::Sprite(table_num) => match table_num {
                0 => 0x3F11,
                1 => 0x3F15,
                2 => 0x3F19,
                3 => 0x3F1D,
                _ => 0,
            },
        };

        Palette {
            palette_index: p_idx,
            starting_addr,
        }
    }

    pub fn get_colors(&self, vram: &VRAM) -> Vec<RGB> {
        let mut colors = vec![];
        for i in 0..4 {
            colors.push(MASTER_PALETTE[vram.get((self.starting_addr + i)) as usize]);
        }

        colors
    }

    pub fn get_color(&self, vram: &VRAM, idx: usize) -> RGB {
        self.get_colors(vram)[idx]
    }
}

/// The OAM (Object Attribute Memory) is internal memory inside the PPU that contains a display list of up to 64 sprites, where each sprite's information occupies 4 bytes.
/// Byte 0: Y position of top of sprite
/// Byte 1: Tile index number
/// Byte 2: Attributes
/// Byte 3: X position of left side of sprite.
///
/// Most programs write to a copy of OAM somewhere in CPU addressable RAM (often $0200-$02FF) and then copy it to OAM each frame using the OAMDMA ($4014) register.
/// Writing N to this register causes the DMA circuitry inside the 2A03/07 to fully initialize the OAM by writing OAMDATA 256 times using successive bytes from starting at address $100*N).
/// The CPU is suspended while the transfer is taking place.

pub struct OAM {
    // We must handle 4 bytes at a time when working with this DRAM
    sprite_info: [u8; 256],
}

impl OAM {
    pub fn new() -> Self {
        Self {
            sprite_info: [0; 256],
        }
    }
}

pub struct PPU {
    curr_row: usize,
    curr_col: usize,
    curr_scanline: usize,

    vram: VRAM,
    oam: OAM,

    // internal registers
    v: u16, // During rendering, used for the scroll position. Outside of rendering, used as the current VRAM address.
    t: u16, // During rendering, specifies the starting coarse-x scroll for the next scanline and the starting y scroll for the screen. Outside of rendering, holds the scroll or VRAM address before transferring it to v.
    fine_x: u16, // The fine-x position of the current scroll, used during rendering alongside v.
    w: bool, // Toggles on each write to either PPUSCROLL or PPUADDR, indicating whether this is the first or second write. Clears on reads of PPUSTATUS. Sometimes called the 'write latch' or 'write toggle'.

    increment: u8, // how much to increment the vram by per read/write
    sprite_pattern_address: u16,
    bg_pattern_address: u16,
    sprite_size: bool,
    mode: bool,
    generate_nmi: bool,
    master_slave_select: bool,
    num_sprites: usize,
    is_vblank: bool,
    sprite_hit: bool,

    base_nametable_address: usize,
    read_buffer: u8,
    oam_address: u8,
    x_scroll: u8,
    y_scroll: u8,

    is_greyscale: bool,
    clip_background: bool,
    clip_sprites: bool,
    show_background: bool,
    show_sprites: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,
}

// TODO: Reading any PPU port, including write-only ports $2000, $2001, $2003, $2005, $2006, returns the PPU I/O bus's value

fn get_bit(num: usize, i: u8) -> u8 {
    ((num >> i) & 1) as u8
}

fn set_bit(num: usize, idx: u8) -> u8 {
    (num | (1 << idx)) as u8
}

// fn set_n_bits(num: usize, idx: u8, n: u8) -> u8 {
//     unimplemented!()
// }

impl PPU {
    pub fn new() -> Self {
        Self {
            vram: VRAM::new(),
            oam: OAM::new(),
            oam_address: 0,

            v: 0,
            t: 0,
            fine_x: 0,
            w: false, // false: 1st write, true: 2nd write

            increment: 1,
            sprite_pattern_address: 0x0000,
            bg_pattern_address: 0x0000,
            sprite_size: false,
            mode: false,
            master_slave_select: false,
            generate_nmi: false,
            num_sprites: 0,
            is_vblank: false,
            sprite_hit: false,

            base_nametable_address: 0x2000,
            read_buffer: 0,
            x_scroll: 0,
            y_scroll: 0,

            is_greyscale: false,
            clip_background: false,
            clip_sprites: false,
            show_background: false,
            show_sprites: false,
            emphasize_red: false,
            emphasize_green: false,
            emphasize_blue: false,
        }
    }

    /// $2000
    pub fn ppu_ctrl(&mut self, value: u8) {
        self.base_nametable_address = match value & 0b11 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => 0x0000, // will never hit
        };
        self.increment = if get_bit(value.into(), 2) == 1 { 1 } else { 32 };
        self.sprite_pattern_address = if get_bit(value.into(), 3) == 1 {
            0x1000
        } else {
            0x0000
        };
        self.bg_pattern_address = if get_bit(value.into(), 4) == 1 {
            0x0000
        } else {
            0x1000
        };
        self.mode = get_bit(value.into(), 5) == 1; // 0 for 8x8, 1 for 8x16
        self.master_slave_select = get_bit(value.into(), 6) == 1; // (0: read backdrop from EXT pins; 1: output color on EXT pins)
        self.generate_nmi = get_bit(value.into(), 7) == 1; // Generate an NMI at the start of the vertical blanking interval (0: off; 1: on)
    }

    /// $2001
    pub fn ppu_mask(&mut self, value: u8) {
        self.is_greyscale = get_bit(value.into(), 0) == 1;
        self.clip_background = get_bit(value.into(), 1) == 1;
        self.clip_sprites = get_bit(value.into(), 2) == 1;
        self.show_background = get_bit(value.into(), 3) == 1;
        self.show_sprites = get_bit(value.into(), 4) == 1;
        self.emphasize_red = get_bit(value.into(), 5) == 1;
        self.emphasize_green = get_bit(value.into(), 6) == 1;
        self.emphasize_blue = get_bit(value.into(), 7) == 1;
    }

    /// $2002
    pub fn ppu_status(&mut self) -> u8 {
        // 7  bit  0
        // ---- ----
        // VSO. ....
        // |||| ||||
        // |||+-++++- PPU open bus. Returns stale PPU bus contents.
        // ||+------- Sprite overflow. The intent was for this flag to be set
        // ||         whenever more than eight sprites appear on a scanline, but a
        // ||         hardware bug causes the actual behavior to be more complicated
        // ||         and generate false positives as well as false negatives; see
        // ||         PPU sprite evaluation. This flag is set during sprite
        // ||         evaluation and cleared at dot 1 (the second dot) of the
        // ||         pre-render line.
        // |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
        // |          a nonzero background pixel; cleared at dot 1 of the pre-render
        // |          line.  Used for raster timing.
        // +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
        //         Set at dot 1 of line 241 (the line *after* the post-render
        //         line); cleared after reading $2002 and at dot 1 of the
        //         pre-render line.

        // TODO(backlog): setup working PPU open bus
        // clear write latch
        self.w = false;

        let mut val = 0b0000_0000;

        if self.num_sprites > 8 {
            val = set_bit(val.into(), 5);
        }

        if self.sprite_hit {
            val = set_bit(val.into(), 6);
        }

        if self.is_vblank {
            val = set_bit(val.into(), 7);
        }
        val
    }

    /// $2003
    pub fn oam_addr(&mut self, value: u8) {
        // Write the address of OAM you want to access here.
        // Most games just write $00 here and then use OAMDMA.
        self.oam_address = value;
    }

    /// $2004
    pub fn oam_data_read(&mut self) -> u8 {
        self.oam.sprite_info[self.oam_address as usize]
    }

    /// $2004
    pub fn oam_data_write(&mut self, value: u8) {
        // Should we ignore writes because DMA is usually always used over this?
        // Wiki says partial writes can cause corruption
        self.oam.sprite_info[self.oam_address as usize] = value;
        self.oam_address = self.oam_address.wrapping_add(1);
    }

    /// $2005
    pub fn ppu_scroll(&mut self, value: u8) {
        if self.w == false {
            self.x_scroll = value;
            self.w = true;
        } else {
            self.y_scroll = value;
            self.w = false;
        }
    }

    /// $2006
    pub fn ppu_addr(&mut self, value: u8) {
        if !self.w {
            // update low byte of t
            self.t = value as u16;
            self.w = true;
        } else {
            // update high byte of t
            self.t |= (value as u16) << 8;
            self.v = self.t;
        }
    }

    // $2007
    pub fn ppu_data_read(&mut self) -> u8 {
        let old_buffer = self.read_buffer;

        let read_result = self.vram.get(self.v.into());
        self.read_buffer = read_result;

        // increment v by bit 2 of $2000 of VRAM
        self.v = self
            .v
            .wrapping_add(((self.vram.get(0x2000) & 0b10) >> 1) as u16);

        old_buffer
    }

    /// $2007
    pub fn ppu_data_write(&mut self, value: u8) {
        self.vram.set(self.v.into(), value);

        // increment v by bit 2 of $2000 of VRAM
        self.v = self
            .v
            .wrapping_add(((self.vram.get(0x2000) & 0b10) >> 1) as u16);
    }

    /// $4014
    pub fn oam_dma(&mut self, mem_slice: &[u8]) {
        self.oam.sprite_info = mem_slice.try_into().unwrap();
    }

    pub fn fetch_bg_tile(&mut self) -> TileFetch {
        let pt_bg = PatternTable::from_memory(PatternTableType::Background, &mut self.vram);

        // 8 cycles of fetch + store to shift registers (BACKGROUND)
        let nt_byte_addr =
            self.base_nametable_address + self.curr_row * 32 + self.curr_col as usize;
        let nt_byte = self.vram.get(nt_byte_addr);
        let attr_byte_offset = (self.curr_row / 4) * 4 + (self.curr_col / 4) + 1;
        let attr_byte = self
            .vram
            .get(self.base_nametable_address + 960 + attr_byte_offset);
        let block_i = self.curr_row % 4;
        let block_j = self.curr_col % 4;
        let quad = if block_i < 2 {
            if block_j < 2 {
                1
            } else {
                2
            }
        } else {
            if block_j < 2 {
                3
            } else {
                4
            }
        };
        let attr_two_bit = match quad {
            1 => attr_byte & 0b0000_0011,
            2 => (attr_byte & 0b0000_1100) >> 2,
            3 => (attr_byte & 0b0011_0000) >> 4,
            4 => (attr_byte & 0b1100_0000) >> 6,
            _ => 0,
        };
        let pt_low_byte = pt_bg.tile_map[nt_byte as usize][self.curr_scanline % 8];
        let pt_hi_byte = pt_bg.tile_map[nt_byte as usize][self.curr_scanline % 8];

        TileFetch {
            nt_byte,
            attr_two_bit,
            pt_low_byte,
            pt_hi_byte,
        }
    }

    pub fn tick(&mut self) {}
}

pub struct TileFetch {
    nt_byte: u8,
    attr_two_bit: u8,
    pt_low_byte: u8,
    pt_hi_byte: u8,
}
