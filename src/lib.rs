mod models;

use crate::models::{LayerHeader, Sprite};
use maikor_vm_core::constants::graphics::{
    ATLAS_TILE_HEIGHT, ATLAS_TILE_WIDTH, LAYER_COUNT, SCREEN_PIXELS, SCREEN_WIDTH, SPRITE_COUNT,
    TILES_PER_ATLAS_ROW,
};
use maikor_vm_core::constants::mem::address::ATLAS1;
use maikor_vm_core::constants::mem::{address, sizes};
use maikor_vm_core::VM;

pub const PIXEL_SIZE: usize = 4;
pub const SCREEN_BYTES: usize = SCREEN_PIXELS * PIXEL_SIZE;

pub struct VMHost {
    pub vm: VM,
    pub fill_color: [u8; 3],
}

impl VMHost {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            vm: VM::new(),
            fill_color: [0, 0, 0],
        }
    }
}

impl VMHost {
    pub fn pop_test(&mut self) {
        self.vm.memory[address::PALETTES.0 as usize + 12] = 255;
        self.vm.memory[address::PALETTES.0 as usize + 13] = 255;
        self.vm.memory[address::PALETTES.0 as usize + 14] = 255;
        self.vm.memory[address::PALETTES.0 as usize + 3] = 255;
        self.vm.memory[address::PALETTES.0 as usize + 7] = 255;
        self.vm.memory[address::PALETTES.0 as usize + 11] = 255;
        self.vm.memory[address::SPRITE_TABLE.0 as usize] = 10;
        self.vm.memory[address::SPRITE_TABLE.0 as usize + 1] = 10;
        self.vm.memory[address::SPRITE_TABLE.0 as usize + 2] = 0;
        self.vm.memory[address::SPRITE_TABLE.0 as usize + 3] = 0;
        for i in 0..(ATLAS_TILE_WIDTH * ATLAS_TILE_HEIGHT) {
            let offset = i * 2;
            self.vm.memory[address::ATLAS1.0 as usize + offset] = 0x41;
            self.vm.memory[address::ATLAS1.0 as usize + offset + 1] = 0x23;
        }
    }
}

impl VMHost {
    pub fn run(&mut self) {
        while self.vm.pc < 50 {
            self.vm.step();
        }
    }

    pub fn render(&self, pixels: &mut [u8]) {
        self.clear_screen(pixels);
        self.render_backgrounds(pixels);
        self.render_sprites(pixels);
    }

    fn clear_screen(&self, pixels: &mut [u8]) {
        for i in 0..SCREEN_PIXELS {
            format_pixel(self.fill_color, i * 4, pixels);
        }
    }

    fn render_backgrounds(&self, _pixels: &mut [u8]) {
        for layer_id in 0..LAYER_COUNT {
            let header_addr = address::LAYER_HEADERS.0 as usize + (sizes::LAYERS_HEADER * layer_id);
            let _content_addr = address::LAYERS.0 as usize + (sizes::LAYERS_CONTENT * layer_id);
            let header =
                LayerHeader::new(&self.vm.memory[header_addr..header_addr + sizes::LAYERS_HEADER]);
            if header.is_visible {}
        }
    }

    fn render_sprites(&self, pixels: &mut [u8]) {
        for i in 0..SPRITE_COUNT {
            let addr = address::SPRITE_TABLE.0 as usize + (sizes::SPRITE * i);
            let sprite = Sprite::new(&self.vm.memory[addr..addr + sizes::SPRITE]);
            if sprite.id < 255 {
                let atlas_y = sprite.id / TILES_PER_ATLAS_ROW * ATLAS_TILE_HEIGHT;
                for y in 0..ATLAS_TILE_HEIGHT {
                    let atlas_row_idx = ATLAS1.0 as usize + (atlas_y * ATLAS_TILE_WIDTH);
                    for x in 0..ATLAS_TILE_WIDTH {
                        let pixels_idx = atlas_row_idx + x;
                        let first = (self.vm.memory[pixels_idx] & 0xF0) >> 4;
                        let second = self.vm.memory[pixels_idx] & 0x0F;
                        if first > 0 {
                            let first = self.get_palette_color(sprite.palette, first);
                            format_pixel(
                                first,
                                ((y + sprite.y) * SCREEN_WIDTH + (x * 2) + sprite.x) * 4,
                                pixels,
                            );
                        }
                        if second > 0 {
                            let second = self.get_palette_color(sprite.palette, second);
                            format_pixel(
                                second,
                                ((y + sprite.y) * SCREEN_WIDTH + (x * 2) + 1 + sprite.x) * 4,
                                pixels,
                            );
                        }
                    }
                }
            }
        }
    }

    fn get_palette_color(&self, palette: usize, color: u8) -> [u8; 3] {
        let palette_addr = address::PALETTES.0 as usize + sizes::PALETTE * palette;
        let color_addr = palette_addr + 3 * color as usize;
        let colours = &self.vm.memory[color_addr..color_addr + 3];
        [colours[0], colours[1], colours[2]]
    }
}

#[cfg(feature = "argb")]
fn format_pixel(colour: [u8; 3], start: usize, pixels: &mut [u8]) {
    pixels[start] = 255;
    pixels[start + 1] = colour[0];
    pixels[start + 2] = colour[1];
    pixels[start + 3] = colour[2];
}

#[cfg(feature = "rgba")]
fn format_pixel(colour: [u8; 3], start: usize, pixels: &mut [u8]) {
    pixels[start] = colour[0];
    pixels[start + 1] = colour[1];
    pixels[start + 2] = colour[2];
    pixels[start + 3] = 255;
}
