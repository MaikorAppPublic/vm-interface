mod mem_cmdr;
mod sound;

use crate::mem_cmdr::MemoryCommander;
use crate::sound::CpalPlayer;
use cpal::traits::StreamTrait;
use cpal::Stream;
use maikor_platform::constants::{
    ATLAS_TILE_HEIGHT, ATLAS_TILE_WIDTH, LAYER_COUNT, SCREEN_HEIGHT, SCREEN_PIXELS, SCREEN_WIDTH,
    SPRITE_COUNT, TILE_HEIGHT, TILE_WIDTH,
};
use maikor_platform::input;
use maikor_platform::mem::address::{ATLAS1, ATLAS2, ATLAS3, ATLAS4};
use maikor_platform::mem::interrupt_flags::IRQ_CONTROLLER;
use maikor_platform::mem::{address, sizes};
use maikor_platform::models::{Byteable, LayerHeader, Sprite};
use maikor_platform::registers::FLG_DEFAULT;
use maikor_vm_core::VM;
use nanorand::{Rng, WyRand};
use std::ops::Add;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub const PIXEL_SIZE: usize = 4;
pub const SCREEN_BYTES: usize = SCREEN_PIXELS * PIXEL_SIZE;
pub const ATLAS_TILE_PIXELS: usize = ATLAS_TILE_WIDTH * ATLAS_TILE_HEIGHT;
pub const TRANSPARENT: [u8; 3] = [0, 0, 0];
pub const CYCLES_PER_FRAME: usize = 200000;

pub struct VMHost {
    pub vm: VM,
    pub keep_alive: Arc<AtomicBool>,
    pub stream_handle: Option<JoinHandle<()>>,
    pub cmdr: MemoryCommander,
    pub fill_color: [u8; 3],
    pub rng: WyRand,
    pub input_state: Input,
    pub on_save_invalidated: Box<fn(usize)>,
    pub on_halt: Box<fn(Option<String>)>,
    pub next_frame: Instant,
}

#[derive(Default, Debug)]
pub struct Input {
    pub a: bool,
    pub b: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub x: bool,
    pub y: bool,
    pub start: bool,
    pub cached: Option<[u8; 2]>,
}

impl Input {
    fn as_bytes(&mut self) -> [u8; 2] {
        if let Some(value) = self.cached {
            value
        } else {
            let value = [
                if self.up { input::mask::UP } else { 0 }
                    | if self.down { input::mask::DOWN } else { 0 },
                if self.start { input::mask::START } else { 0 },
            ];
            self.cached = Some(value);
            value
        }
    }
}

impl VMHost {
    #[allow(clippy::new_without_default)]
    pub fn new(
        on_save_invalidated: Box<fn(usize)>,
        on_halt: Box<fn(Option<String>)>,
    ) -> Result<Self, String> {
        match CpalPlayer::get() {
            Some((player, keep_alive, handle)) => Ok(Self {
                vm: VM::new(Box::new(player)),
                keep_alive,
                stream_handle: Some(handle),
                cmdr: MemoryCommander::default(),
                fill_color: [0, 0, 0],
                rng: WyRand::new(),
                input_state: Input::default(),
                on_save_invalidated,
                on_halt,
                next_frame: Instant::now(),
            }),
            None => Err(String::from("Unable to create audio player")),
        }
    }
}

impl VMHost {
    pub fn reset(&mut self) {
        self.vm.registers.fill(0);
        self.vm.registers[8] = FLG_DEFAULT;
        self.vm.memory[address::RAM as usize..(address::RAM + sizes::RAM_BANK) as usize]
            .as_mut()
            .fill(0);
        for bank in &mut self.vm.ram_banks {
            bank.fill(0);
        }
        self.vm.error = None;
        self.vm.pc = 0;
        self.vm.halted = false;
        self.vm.op_executed = 0;
        self.vm.cycles_executed = 0;
        self.vm.save_dirty_flag.fill(false);
        self.vm.sound.reset();
        self.next_frame = Instant::now();
    }
}

impl VMHost {
    pub fn execute(&mut self) {
        if self.next_frame <= Instant::now() {
            self.next_frame = Instant::now().add(Duration::from_millis(16));
        } else {
            return;
        }
        let mut cycles: usize = 0;
        while cycles < CYCLES_PER_FRAME {
            let cost = self.vm.step();
            cycles += cost;
            self.vm.sound.do_cycle(cost as u32);
            self.check_for_input_changes();
            if self.vm.halted {
                let error = self.vm.error.clone();
                (self.on_halt)(error);
                self.keep_alive.store(false, Ordering::SeqCst);
                if let Some(handle) = self.stream_handle.take() {
                    let _ = handle.join();
                }
                return;
            }
            self.vm.memory[address::RAND as usize] = self.rng.generate();
            self.cmdr.update(&mut self.vm.memory);
        }
    }

    fn check_for_input_changes(&mut self) {
        let input_bytes = self.input_state.as_bytes();
        if self.vm.memory[address::INPUT as usize] != input_bytes[0]
            || self.vm.memory[address::INPUT as usize + 1] != input_bytes[1]
        {
            self.vm.memory[address::INPUT as usize] = input_bytes[0];
            self.vm.memory[address::INPUT as usize + 1] = input_bytes[1];
            self.vm.trigger_interrupt(IRQ_CONTROLLER);
        }
    }
}

impl VMHost {
    pub fn render(&self, pixels: &mut [u8]) {
        self.clear_screen(pixels);
        self.render_backgrounds(pixels);
        self.render_sprites(pixels);
        //self.render_controller_sprites(pixel);
    }

    fn clear_screen(&self, pixels: &mut [u8]) {
        for i in 0..SCREEN_PIXELS {
            format_pixel(self.fill_color, false, i * 4, pixels);
        }
    }

    fn render_backgrounds(&self, _pixels: &mut [u8]) {
        for layer_id in 0..LAYER_COUNT {
            let header_addr =
                address::LAYER_HEADERS as usize + (sizes::LAYERS_HEADER as usize * layer_id);
            let _content_addr =
                address::LAYERS as usize + (sizes::LAYERS_CONTENT as usize * layer_id);
            let header = LayerHeader::from_bytes(
                &self.vm.memory[header_addr..header_addr + sizes::LAYERS_HEADER as usize],
            );
            if header.enabled {}
        }
    }

    fn render_sprites(&self, pixels: &mut [u8]) {
        for i in 0..=SPRITE_COUNT {
            let addr = address::SPRITE_TABLE as usize + (sizes::SPRITE as usize * i);
            let sprite = Sprite::from_bytes(&self.vm.memory[addr..addr + sizes::SPRITE as usize]);
            if sprite.enabled {
                match (sprite.flip_h, sprite.flip_v) {
                    (false, false) => self.render_sprite(pixels, sprite),
                    (true, false) => self.render_sprite_horz_flipped(pixels, sprite),
                    (false, true) => self.render_sprite_vert_flipped(pixels, sprite),
                    (true, true) => self.render_sprite_horz_vert_flipped(pixels, sprite),
                }
            }
        }
    }

    fn render_sprite(&self, pixels: &mut [u8], sprite: Sprite) {
        let mut x = 0;
        let mut y = 0;
        for i in 0..ATLAS_TILE_PIXELS {
            let (color1, color2) = self.get_colors(&sprite, i);
            self.set_pixel(pixels, &sprite, x * 2, y, color1, color2);
            x += 1;
            if x >= ATLAS_TILE_WIDTH {
                x = 0;
                y += 1;
            }
        }
    }

    fn render_sprite_horz_vert_flipped(&self, pixels: &mut [u8], sprite: Sprite) {
        let mut x = 0;
        let mut y = 0;
        for i in 0..ATLAS_TILE_PIXELS {
            let (color1, color2) = self.get_colors(&sprite, i);
            self.set_pixel(
                pixels,
                &sprite,
                TILE_WIDTH - 2 - x * 2,
                TILE_HEIGHT - 1 - y,
                color2,
                color1,
            );
            x += 1;
            if x >= ATLAS_TILE_WIDTH {
                x = 0;
                y += 1;
            }
        }
    }

    fn render_sprite_horz_flipped(&self, pixels: &mut [u8], sprite: Sprite) {
        let mut x = 0;
        let mut y = 0;
        for i in 0..ATLAS_TILE_PIXELS {
            let (color1, color2) = self.get_colors(&sprite, i);
            self.set_pixel(pixels, &sprite, TILE_WIDTH - 2 - x * 2, y, color2, color1);
            x += 1;
            if x >= ATLAS_TILE_WIDTH {
                x = 0;
                y += 1;
            }
        }
    }

    fn render_sprite_vert_flipped(&self, pixels: &mut [u8], sprite: Sprite) {
        let mut x = 0;
        let mut y = 0;
        for i in 0..ATLAS_TILE_PIXELS {
            let (color1, color2) = self.get_colors(&sprite, i);
            self.set_pixel(pixels, &sprite, x * 2, TILE_HEIGHT - 1 - y, color1, color2);
            x += 1;
            if x >= ATLAS_TILE_WIDTH {
                x = 0;
                y += 1;
            }
        }
    }

    fn set_pixel(
        &self,
        pixels: &mut [u8],
        sprite: &Sprite,
        x: usize,
        y: usize,
        color1: [u8; 3],
        color2: [u8; 3],
    ) {
        let scr_x = (x + sprite.x).saturating_sub(8);
        let scr_y = (y + sprite.y).saturating_sub(8);
        if scr_x >= SCREEN_WIDTH || scr_y >= SCREEN_HEIGHT {
            return;
        }
        let idx = (scr_x + scr_y * SCREEN_WIDTH) * PIXEL_SIZE;
        if color1 != TRANSPARENT {
            format_pixel(color1, sprite.half_alpha, idx, pixels);
        }
        if color2 != TRANSPARENT {
            format_pixel(color2, sprite.half_alpha, idx + PIXEL_SIZE, pixels);
        }
    }

    fn get_colors(&self, sprite: &Sprite, pixel: usize) -> ([u8; 3], [u8; 3]) {
        let atlas = match sprite.atlas {
            0 => ATLAS1,
            1 => ATLAS2,
            2 => ATLAS3,
            4 => ATLAS4,
            _ => panic!("Impossible atlas value: {}", sprite.atlas),
        };
        let colors = self.vm.memory[atlas as usize + sprite.id * ATLAS_TILE_PIXELS + pixel];
        let first = (colors & 0xF0) >> 4;
        let second = colors & 0x0F;

        (
            self.get_palette_color(sprite.palette, first),
            self.get_palette_color(sprite.palette, second),
        )
    }

    fn get_palette_color(&self, palette: usize, color: u8) -> [u8; 3] {
        let palette_addr = address::PALETTES as usize + sizes::PALETTE as usize * palette;
        let color_addr = palette_addr + 3 * color as usize;
        let colours = &self.vm.memory[color_addr..color_addr + 3];
        [colours[0], colours[1], colours[2]]
    }
}

#[cfg(feature = "argb")]
fn format_pixel(colour: [u8; 3], half_alpha: bool, start: usize, pixels: &mut [u8]) {
    pixels[start] = 255;
    if half_alpha {
        pixels[start + 1] = mix(pixels[start + 1], colour[0]);
        pixels[start + 2] = mix(pixels[start + 2], colour[1]);
        pixels[start + 3] = mix(pixels[start + 3], colour[2]);
    } else {
        pixels[start + 1] = colour[0];
        pixels[start + 2] = colour[1];
        pixels[start + 3] = colour[2];
    }
}

#[cfg(feature = "rgba")]
fn format_pixel(colour: [u8; 3], half_alpha: bool, start: usize, pixels: &mut [u8]) {
    if half_alpha {
        pixels[start] = mix(pixels[start], colour[0]);
        pixels[start + 1] = mix(pixels[start + 1], colour[1]);
        pixels[start + 2] = mix(pixels[start + 2], colour[2]);
    } else {
        pixels[start] = colour[0];
        pixels[start + 1] = colour[1];
        pixels[start + 2] = colour[2];
    }
    pixels[start + 3] = 255;
}

#[inline(always)]
const fn mix(lhs: u8, rhs: u8) -> u8 {
    lhs.saturating_add(rhs / 3 * 2)
}
