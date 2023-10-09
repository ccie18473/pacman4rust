#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use crate::*;

pub const PAC_CLOCK_SPEED: u32 = 3072000; // 3.072 MHz (= number of cycles per second)
pub const PAC_FPS: u32 = 60;
pub const PAC_CYCLES_PER_FRAME: u32 = PAC_CLOCK_SPEED / PAC_FPS;
pub const PAC_SCREEN_WIDTH: usize = 224;
pub const PAC_SCREEN_HEIGHT: usize = 288;

pub type update_screen = fn(g: &mut game);
pub type update_screen_null = fn(g: &mut game);
pub type push_sample = fn(g: &mut game, sample: i16);
pub type push_sample_null = fn(g: &mut game, sample: i16);

pub struct pac<'a> {
    pub cpu: z80::z80<'a>,
    pub rom: [u8; 0x10000],     // 0x0000-0x4000
    pub ram: [u8; 0x1000],      // 0x4000-0x5000
    pub sprite_pos: [u8; 0x10], // 0x5060-0x506f

    pub color_rom: [u8; 32],
    pub palette_rom: [u8; 0x100],
    pub tile_rom: [u8; 0x1000],
    pub sprite_rom: [u8; 0x1000],
    pub sound_rom1: [u8; 0x100],
    pub sound_rom2: [u8; 0x100],

    pub tiles: [u8; 256 * 8 * 8],    // to store predecoded tiles
    pub sprites: [u8; 64 * 16 * 16], // to store predecoded sprites

    pub int_vector: u8,
    pub vblank_enabled: u8,
    pub sound_enabled: u8,
    pub flip_screen: u8,

    // in 0 port
    pub p1_up: u8,
    pub p1_left: u8,
    pub p1_right: u8,
    pub p1_down: u8,
    pub rack_advance: u8,
    pub coin_s1: u8,
    pub coin_s2: u8,
    pub credits_btn: u8,

    // in 1 port
    pub board_test: u8,
    pub p1_start: u8,
    pub p2_start: u8,

    // ppu
    pub screen_buffer: [u8; PAC_SCREEN_HEIGHT * PAC_SCREEN_WIDTH * 3],
    pub update_screen: update_screen_null,

    // audio
    pub sound_chip: wsg::wsg,
    pub audio_buffer_len: i32,
    pub audio_buffer: Vec<i16>,
    pub sample_rate: i32,
    pub mute_audio: bool,
    pub push_sample: push_sample_null,
}
impl<'a> pac<'a> {
    pub fn new() -> Self {
        Self {
            cpu: z80::z80::new(),
            rom: [0; 0x10000],
            ram: [0; 0x1000],
            sprite_pos: [0; 0x10],
            color_rom: [0; 32],
            palette_rom: [0; 0x100],
            tile_rom: [0; 0x1000],
            sprite_rom: [0; 0x1000],
            sound_rom1: [0; 0x100],
            sound_rom2: [0; 0x100],
            tiles: [0; 256 * 8 * 8],
            sprites: [0; 64 * 16 * 16],
            int_vector: 0,
            vblank_enabled: 0,
            sound_enabled: 0,
            flip_screen: 0,
            // in 0 port
            p1_up: 0,
            p1_left: 0,
            p1_right: 0,
            p1_down: 0,
            rack_advance: 0,
            coin_s1: 0,
            coin_s2: 0,
            credits_btn: 0,
            // in 1 port
            board_test: 0,
            p1_start: 0,
            p2_start: 0,
            // ppu
            screen_buffer: [0; PAC_SCREEN_HEIGHT * PAC_SCREEN_WIDTH * 3],
            update_screen: update_screen,
            // audio
            sound_chip: wsg::wsg::new(),
            audio_buffer_len: 0,
            audio_buffer: Vec::new(),
            sample_rate: 0,
            mute_audio: false,
            push_sample: push_sample,
        }
    }
}

fn rb(userdata: &mut userdata, addr: u16) -> u8 {
    //println!("rb-pac");
    // according to https://www.csh.rit.edu/~jerry/arcade/pacman/daves/
    // the highest bit of the address is unused
    let addr = addr & 0x7fff;

    unsafe {
        if addr < 0x4000 {
            return (*userdata.game_ptr).p.rom[addr as usize];
        } else if addr < 0x5000 {
            return (*userdata.game_ptr).p.ram[(addr - 0x4000) as usize];
        } else if addr <= 0x50ff {
            // io
            if addr == 0x5003 {
                return (*userdata.game_ptr).p.flip_screen;
            } else if addr == 0x5004 || addr == 0x5005 {
                // lamps, not used in pacman
                return 0;
            } else if addr == 0x5006 {
                // coin lockout, not used in pacman
                return 0;
            } else if addr == 0x5007 {
                // coin counter
            } else if addr >= 0x5000 && addr <= 0x503f {
                // in 0
                let value: u8 = ((!(*userdata.game_ptr).p.p1_up & 0x1) << 0)
                    | ((!(*userdata.game_ptr).p.p1_left & 0x1) << 1)
                    | ((!(*userdata.game_ptr).p.p1_right & 0x1) << 2)
                    | ((!(*userdata.game_ptr).p.p1_down & 0x1) << 3)
                    | ((!(*userdata.game_ptr).p.rack_advance & 0x1) << 4)
                    | ((!(*userdata.game_ptr).p.coin_s1 & 0x1) << 5)
                    | ((!(*userdata.game_ptr).p.coin_s2 & 0x1) << 6)
                    | ((!(*userdata.game_ptr).p.credits_btn & 0x1) << 7);
                return value;
            } else if addr >= 0x5040 && addr <= 0x507f {
                // in 1
                let value: u8 = ((!(*userdata.game_ptr).p.p1_up & 0x1) << 0)
                    | ((!(*userdata.game_ptr).p.p1_left & 0x1) << 1)
                    | ((!(*userdata.game_ptr).p.p1_right & 0x1) << 2)
                    | ((!(*userdata.game_ptr).p.p1_down & 0x1) << 3)
                    | ((!(*userdata.game_ptr).p.board_test & 0x1) << 4)
                    | ((!(*userdata.game_ptr).p.p1_start & 0x1) << 5)
                    | ((!(*userdata.game_ptr).p.p2_start & 0x1) << 6)
                    | (1 << 7);
                return value;
                // cabinet mode: 1=upright 0=table
            } else if addr >= 0x5080 && addr <= 0x50bf {
                // dip switch
                // bit 0-1: 1 Coin 1 Credit
                // bit 2-3: 3 Pacman Per Game
                // bit 4-5: Bonus Player @ 10000 Pts
                // bit 6: Difficulty (normal=1, hard=0)
                // bit 7: Alternate ghost names
                return 0b11001001;
            }
        } else {
            println!("ERR: read at {:04x}", addr);
            return 0;
        }

        return 0xff;
    }
}

fn wb(userdata: &mut userdata, addr: u16, val: u8) {
    //println!("wb-pac");
    // according to https://www.csh.rit.edu/~jerry/arcade/pacman/daves/
    // the highest bit of the address is unused
    let addr = addr & 0x7fff;

    unsafe {
        if addr < 0x4000 {
            // cannot write to rom
        } else if addr < 0x5000 {
            (*userdata.game_ptr).p.ram[(addr - 0x4000) as usize] = val;
        } else if addr <= 0x50ff {
            // io
            if addr == 0x5000 {
                (*userdata.game_ptr).p.vblank_enabled = val & 1;
            } else if addr == 0x5001 {
                (*userdata.game_ptr).p.sound_enabled = val & 1;
            } else if addr == 0x5002 {
                // aux board?
            } else if addr == 0x5003 {
                (*userdata.game_ptr).p.flip_screen = val & 1;
            } else if addr == 0x5004 || addr == 0x5005 {
                // lamps, not used in pacman
            } else if addr == 0x5006 {
                // coin lockout, not used in pacman
            } else if addr == 0x5007 {
                // coin counter
            } else if addr >= 0x5040 && addr <= 0x505f {
                // audio
                wsg_write(
                    &mut (*userdata.game_ptr).p.sound_chip,
                    (addr - 0x5040) as u8,
                    val,
                );
            } else if addr >= 0x5060 && addr <= 0x506f {
                (*userdata.game_ptr).p.sprite_pos[(addr - 0x5060) as usize] = val;
            } else if addr >= 0x50c0 && addr <= 0x50ff {
                // watchdog: no action is needed here, because watchdog is not
                // implemented on the emu.
            }
        } else {
            println!("ERR: write {:02x} at {:04x}", val, addr);
        }
    }
}

pub fn port_in(z: &mut z80::z80, _port: u8) -> u8 {
    //println!("port_in");

    return 0;
}

pub fn port_out(z: &mut z80::z80, port: u8, val: u8) {
    //println!("port_out");

    // setting the interrupt vector
    if port == 0 {
        unsafe {
            (*z.userdata.game_ptr).p.int_vector = val;
        }
    }
}

// copies "nb_bytes" bytes from a file into memory
pub fn load_file(filename: &str, memory: &mut [u8], nb_bytes: usize) -> i32 {
    //println!("load_file");

    let mut f = File::open(filename).unwrap();

    // copying the bytes in memory:
    f.read_exact(memory).unwrap();

    return 0;
}

// MARK: graphics

// the color palette is stored in color_rom (82s123.7f). Each byte corresponds
// to one color and is composed of three components red, green and blue
// following that pattern: 0bBBGGGRRR.
// Each color component corresponds to a color intensity.
// @TODO: add comment on how to get from color intensity to RGB color.
pub fn get_color(game: &mut game, color_no: u8, r: &mut u8, g: &mut u8, b: &mut u8) {
    //println!("get_color");

    let data: u8 = game.p.color_rom[color_no as usize];

    *r = ((data >> 0) & 1) * 0x21 + ((data >> 1) & 1) * 0x47 + ((data >> 2) & 1) * 0x97;
    *g = ((data >> 3) & 1) * 0x21 + ((data >> 4) & 1) * 0x47 + ((data >> 5) & 1) * 0x97;
    *b = ((data >> 6) & 1) * 0x51 + ((data >> 7) & 1) * 0xae;
}

// Color palettes are defined in palette_rom (82s126.4a): each palette contains
// four colors (one byte for each color).
pub fn get_palette(g: &mut game, pal_no: u8, pal: &mut [u8; 4]) {
    //println!("get_palette");

    let pal_no = pal_no & 0x3f;

    pal[0] = g.p.palette_rom[pal_no as usize * 4 + 0];
    pal[1] = g.p.palette_rom[pal_no as usize * 4 + 1];
    pal[2] = g.p.palette_rom[pal_no as usize * 4 + 2];
    pal[3] = g.p.palette_rom[pal_no as usize * 4 + 3];
}

// decodes a strip from pacman tile/sprite roms to a bitmap output where each
// byte represents one pixel.
pub fn decode_strip(
    g: &mut game,
    input: *mut u8,
    output: *mut u8,
    bx: i32,
    by: i32,
    img_width: i32,
) {
    //println!("decode_strip");

    let base_i: i32 = by * img_width + bx;
    unsafe {
        for x in 0..8 {
            let strip: u8 = *(input.add(x));

            for y in 0..4 {
                // bitmaps are stored mirrored in memory, so we need to read it
                // starting from the bottom right:
                let i = (3 - y) * img_width + 7 - x as i32;
                *output.add((base_i + i) as usize) = (strip >> (y % 4)) & 1;
                *output.add((base_i + i) as usize) |= ((strip >> (y % 4 + 4)) & 1) << 1;
            }
        }
    }
}

// preloads sprites and tiles
pub fn preload_images(g: &mut game) {
    //println!("preload_images");

    // sprites and tiles are images that are stored in sprite/tile rom.
    // in memory, those images are represented using vertical "strips"
    // of 8*4px, each strip being 8 bytes long (each pixel is stored on two
    // bits)
    let LEN_STRIP_BYTES: i32 = 8;

    // tiles are 8*8px images. in memory, they are composed of two strips.
    let NB_PIXELS_PER_TILE: i32 = 8 * 8;
    let TILE_WIDTH: i32 = 8;
    let NB_TILES: i32 = 256;

    //memset(p->tiles, 0, NB_TILES * NB_PIXELS_PER_TILE);
    g.p.tiles = [0; 256 * 8 * 8];
    unsafe {
        for i in 0..NB_TILES {
            let tile: *mut u8 = &mut g.p.tiles[(i * NB_PIXELS_PER_TILE) as usize];
            let rom: *mut u8 = &mut g.p.tile_rom[(i * (LEN_STRIP_BYTES * 2)) as usize];

            decode_strip(g, rom.add(0), tile, 0, 4, TILE_WIDTH);
            decode_strip(g, rom.add(8), tile, 0, 0, TILE_WIDTH);
        }
    }

    // sprites are 16*16px images. in memory, they are composed of 8 strips.
    let NB_PIXELS_PER_SPRITE: i32 = 16 * 16;
    let SPRITE_WIDTH: i32 = 16;
    let NB_SPRITES: i32 = 64;

    //memset(p->sprites, 0, NB_SPRITES * NB_PIXELS_PER_SPRITE);
    g.p.sprites = [0; 64 * 16 * 16];
    unsafe {
        for i in 0..NB_SPRITES {
            let sprite: *mut u8 = &mut g.p.sprites[(i * NB_PIXELS_PER_SPRITE) as usize];
            let rom: *mut u8 = &mut g.p.sprite_rom[(i * (LEN_STRIP_BYTES * 8)) as usize];

            decode_strip(g, rom.add(0 * 8), sprite, 8, 12, SPRITE_WIDTH);
            decode_strip(g, rom.add(1 * 8), sprite, 8, 0, SPRITE_WIDTH);
            decode_strip(g, rom.add(2 * 8), sprite, 8, 4, SPRITE_WIDTH);
            decode_strip(g, rom.add(3 * 8), sprite, 8, 8, SPRITE_WIDTH);

            decode_strip(g, rom.add(4 * 8), sprite, 0, 12, SPRITE_WIDTH);
            decode_strip(g, rom.add(5 * 8), sprite, 0, 0, SPRITE_WIDTH);
            decode_strip(g, rom.add(6 * 8), sprite, 0, 4, SPRITE_WIDTH);
            decode_strip(g, rom.add(7 * 8), sprite, 0, 8, SPRITE_WIDTH);
        }
    }
}

pub fn draw_tile(game: &mut game, tile_no: u8, pal: &mut [u8; 4], x: u16, y: u16) {
    //println!("draw_tile");

    if x < 0 || x >= PAC_SCREEN_WIDTH as u16 {
        return;
    }

    for i in 0..8 * 8 {
        let px: i32 = i % 8;
        let py: i32 = i / 8;

        let color: u8 = game.p.tiles[tile_no as usize * 64 + i as usize];
        let screenbuf_pos: i32 = (y as i32 + py) * PAC_SCREEN_WIDTH as i32 + (x as i32 + px);

        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        get_color(game, pal[color as usize], &mut r, &mut g, &mut b);
        game.p.screen_buffer[screenbuf_pos as usize * 3 + 0] = r;
        game.p.screen_buffer[screenbuf_pos as usize * 3 + 1] = g;
        game.p.screen_buffer[screenbuf_pos as usize * 3 + 2] = b;
    }
}

pub fn draw_sprite(
    game: &mut game,
    sprite_no: u8,
    pal: &mut [u8; 4],
    x: i16,
    y: i16,
    flip_x: u8,
    flip_y: u8,
) {
    //println!("draw_sprite");

    if x <= -16 || x > PAC_SCREEN_WIDTH as i16 {
        return;
    }

    let base_i: i32 = y as i32 * PAC_SCREEN_WIDTH as i32 + x as i32;

    for i in 0..16 * 16 {
        let px: i32 = i % 16;
        let py: i32 = i / 16;

        let color: u8 = game.p.sprites[sprite_no as usize * 256 + i as usize];

        // color 0 is transparent
        if pal[color as usize] == 0 {
            continue;
        }

        let x_pos: i32;
        if flip_x != 0 {
            x_pos = 15 - px;
        } else {
            x_pos = px;
        }

        let y_pos: i32;
        if flip_y != 0 {
            y_pos = 15 - py;
        } else {
            y_pos = py;
        }

        let screenbuf_pos: i32 = base_i + y_pos * PAC_SCREEN_WIDTH as i32 + x_pos;

        if x as i32 + x_pos < 0 || x as i32 + x_pos >= PAC_SCREEN_WIDTH as i32 {
            continue;
        }

        let mut r = game.p.screen_buffer[screenbuf_pos as usize * 3 + 0];
        let mut g = game.p.screen_buffer[screenbuf_pos as usize * 3 + 1];
        let mut b = game.p.screen_buffer[screenbuf_pos as usize * 3 + 2];

        get_color(game, pal[color as usize], &mut r, &mut g, &mut b);

        game.p.screen_buffer[screenbuf_pos as usize * 3 + 0] = r;
        game.p.screen_buffer[screenbuf_pos as usize * 3 + 1] = g;
        game.p.screen_buffer[screenbuf_pos as usize * 3 + 2] = b;
    }
}

pub fn pac_draw(g: &mut game) {
    //println!("pac_draw");

    // 1. writing tiles according to VRAM

    let VRAM_SCREEN_BOT: u16 = 0x4000;
    let VRAM_SCREEN_MID: u16 = 0x4000 + 64;
    let VRAM_SCREEN_TOP: u16 = 0x4000 + 64 + 0x380;

    let mut x: i32;
    let mut y: i32;
    let mut i: i32;
    let mut palette: [u8; 4] = [0; 4];

    // bottom of screen:
    x = 31;
    y = 34;
    i = VRAM_SCREEN_BOT as i32;
    while x != 31 || y != 36 {
        let tile_no: u8 = rb(&mut g.p.cpu.userdata, i as u16);
        let palette_no: u8 = rb(&mut g.p.cpu.userdata, i as u16 + 0x400);

        get_palette(g, palette_no, &mut palette);
        draw_tile(g, tile_no, &mut palette, (x as u16 - 2) * 8, y as u16 * 8);

        i += 1;
        if x == 0 {
            x = 31;
            y += 1;
        } else {
            x -= 1;
        }
    }

    // middle of the screen:
    x = 29;
    y = 2;
    i = VRAM_SCREEN_MID as i32;
    while x != 1 || y != 2 {
        let tile_no: u8 = rb(&mut g.p.cpu.userdata, i as u16);
        let palette_no: u8 = rb(&mut g.p.cpu.userdata, i as u16 + 0x400);

        get_palette(g, palette_no, &mut palette);
        draw_tile(g, tile_no, &mut palette, (x as u16 - 2) * 8, y as u16 * 8);

        i += 1;
        if y == 33 {
            y = 2;
            x -= 1;
        } else {
            y += 1;
        }
    }

    // top of the screen:
    x = 31;
    y = 0;
    i = VRAM_SCREEN_TOP as i32;
    while x != 31 || y != 2 {
        let tile_no: u8 = rb(&mut g.p.cpu.userdata, i as u16);
        let palette_no: u8 = rb(&mut g.p.cpu.userdata, i as u16 + 0x400);

        get_palette(g, palette_no, &mut palette);
        draw_tile(g, tile_no, &mut palette, (x as u16 - 2) * 8, y as u16 * 8);

        i += 1;
        if x == 0 {
            x = 31;
            y += 1;
        } else {
            x -= 1;
        }
    }

    // 2. drawing the 8 sprites (in reverse order)
    let VRAM_SPRITES_INFO: u16 = 0x4FF0;
    for s in (0..=7).rev() {
        // the screen coordinates of a sprite start on the lower right corner
        // of the main screen:
        let x: i16 = (PAC_SCREEN_WIDTH as i16) - (g.p.sprite_pos[s * 2] as i16) + 15;
        let y: i16 = (PAC_SCREEN_HEIGHT as i16) - (g.p.sprite_pos[s * 2 + 1] as i16) - 16;

        let sprite_info: u8 = rb(&mut g.p.cpu.userdata, VRAM_SPRITES_INFO + (s as u16 * 2));
        let palette_no: u8 = rb(
            &mut g.p.cpu.userdata,
            VRAM_SPRITES_INFO + (s as u16 * 2) + 1,
        );

        let flip_x: u8 = (sprite_info >> 1) & 1;
        let flip_y: u8 = (sprite_info >> 0) & 1;
        let sprite_no: u8 = sprite_info >> 2;

        get_palette(g, palette_no, &mut palette);
        draw_sprite(g, sprite_no, &mut palette, x, y, flip_x, flip_y);
    }
}

// generates audio for one frame
pub fn sound_update(g: &mut game) {
    //println!("sound_update");

    if g.p.sound_enabled == 0 || g.p.mute_audio {
        return;
    }

    // update the WSG (filling the audio buffer)
    wsg_play(
        &mut g.p.sound_chip,
        &mut g.p.audio_buffer,
        g.p.audio_buffer_len,
    );

    // resampling the 96kHz audio stream from the WSG into a 44.1kHz one
    let d: f32 = WSG_SAMPLE_RATE as f32 / g.p.sample_rate as f32;
    for i in 0..g.p.sample_rate / PAC_FPS as i32 {
        let pos: i32 = (d * i as f32) as i32;
        (g.p.push_sample)(g, g.p.audio_buffer[pos as usize]);
    }
}

pub fn pac_init(g: &mut game) -> i32 {
    //println!("pac_init");

    z80_init(&mut g.p.cpu);
    //game-->pacman-->z80-->userdata-->game_ptr
    g.p.cpu.userdata.game_ptr = g;
    g.p.cpu.read_byte = rb;
    g.p.cpu.write_byte = wb;
    g.p.cpu.port_in = port_in;
    g.p.cpu.port_out = port_out;

    // loading rom files
    let mut r: i32 = 0;

    r += load_file("roms/pacman.6e", &mut g.p.rom[0..0x1000], 0x1000);
    r += load_file("roms/pacman.6f", &mut g.p.rom[0x1000..0x2000], 0x1000);
    r += load_file("roms/pacman.6h", &mut g.p.rom[0x2000..0x3000], 0x1000);
    r += load_file("roms/pacman.6j", &mut g.p.rom[0x3000..0x4000], 0x1000);

    r += load_file("roms/82s123.7f", &mut g.p.color_rom, 32);

    r += load_file("roms/82s126.4a", &mut g.p.palette_rom, 0x100);

    r += load_file("roms/pacman.5e", &mut g.p.tile_rom, 0x1000);

    r += load_file("roms/pacman.5f", &mut g.p.sprite_rom, 0x1000);

    r += load_file("roms/82s126.1m", &mut g.p.sound_rom1, 0x100);

    r += load_file("roms/82s126.3m", &mut g.p.sound_rom2, 0x100);

    preload_images(g);
    //g.p.update_screen = NULL;

    // audio
    wsg_init(&mut g.p.sound_chip, g.p.sound_rom1);
    g.p.audio_buffer_len = (WSG_SAMPLE_RATE / PAC_FPS) as i32;
    g.p.audio_buffer.resize(g.p.audio_buffer_len as usize, 0);
    g.p.sample_rate = 44100;
    g.p.mute_audio = false;

    return r;
}

pub fn pac_quit(g: &mut game) {
    //println!("pac_quit");

    //free(p->audio_buffer);
}

// updates emulation for "ms" milliseconds.
pub fn pac_update(g: &mut game, ms: u32) {
    //println!("pac_update");

    // machine executes exactly PAC_CLOCK_SPEED cycles every second,
    // so we need to execute "ms * PAC_CLOCK_SPEED / 1000"
    let mut count: i32 = 0;
    while count < (ms * PAC_CLOCK_SPEED) as i32 / 1000 {
        let cyc: i32 = g.p.cpu.cyc as i32;
        z80_step(&mut g.p.cpu);
        let elapsed: i32 = g.p.cpu.cyc as i32 - cyc;
        count += elapsed;

        if g.p.cpu.cyc >= PAC_CYCLES_PER_FRAME as u64 {
            g.p.cpu.cyc -= PAC_CYCLES_PER_FRAME as u64;

            // trigger vblank if enabled:
            if g.p.vblank_enabled != 0 {
                // p->vblank_enabled = 0;
                z80_gen_int(&mut g.p.cpu, g.p.int_vector);

                pac_draw(g);
                (g.p.update_screen)(g);
                sound_update(g);
            }
        }
    }
}

// invincibility patch (from http://cheat.retrogames.com)
pub fn pac_cheat_invincibility(g: &mut game) {
    //println!("pac_cheat_invincibility");

    g.p.rom[0x1774 + 3] = 0x32;
    g.p.rom[0x1774 + 2] = 0x3c;
    g.p.rom[0x1774 + 1] = 0xe0;
    g.p.rom[0x1774 + 0] = 0xc3;

    g.p.rom[0x3cdf + 3] = 0x04;
    g.p.rom[0x3cdf + 2] = 0x20;
    g.p.rom[0x3cdf + 1] = 0xa7;
    g.p.rom[0x3cdf + 0] = 0x00;

    g.p.rom[0x3ce3 + 3] = 0x17;
    g.p.rom[0x3ce3 + 2] = 0x64;
    g.p.rom[0x3ce3 + 1] = 0xc3;
    g.p.rom[0x3ce3 + 0] = 0xaf;

    g.p.rom[0x3ce7 + 3] = 0x17;
    g.p.rom[0x3ce7 + 2] = 0x77;
    g.p.rom[0x3ce7 + 1] = 0xc3;
    g.p.rom[0x3ce7 + 0] = 0xaf;

    println!("applied invincibility patch");
}
