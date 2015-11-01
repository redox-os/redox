use alloc::boxed::Box;

use core::{cmp, mem};
use core::simd::*;

use graphics::color::Color;
use graphics::point::Point;
use graphics::size::Size;
use fs::File;
use io::*;
use vec::Vec;
use syscall::{sys_alloc, sys_unalloc};

/// A display
pub struct Display {
    pub offscreen: usize,
    pub onscreen: usize,
    pub size: usize,
    pub bytesperrow: usize,
    pub width: usize,
    pub height: usize,
    pub root: bool,
}

impl Display {
    /// Create a new display
    pub fn new(width: usize, height: usize) -> Box<Self> {
        unsafe {
            let bytesperrow = width * 4;
            let memory_size = bytesperrow * height;

            let ret = box Display {
                offscreen: sys_alloc(memory_size),
                onscreen: sys_alloc(memory_size),
                size: memory_size,
                bytesperrow: bytesperrow,
                width: width,
                height: height,
                root: false,
            };

            // if sys_alloc returns 0 then draw green to the screen
            if ret.offscreen == 0 {
                if let Some(mut disp) = File::open("display://") {
                       let colors:Vec<u32> = vec![0xFF00FF00; 640*480];
                       unsafe { 
                            let u8s = mem::transmute::<&[u32],&[u8]>(&colors[..]); 
                            disp.write(u8s);
                            disp.sync();
                            disp.seek(SeekFrom::Start(0));
                        }
                }
            }
            ret.set(Color::rgb(0, 0, 0));
            ret.flip();

            ret
        }
    }

    /* Optimized { */
    pub unsafe fn set_run(data: u32, dst: usize, len: usize) {
        let mut i = 0;
        //Only use 16 byte transfer if possible
        if len - (dst + i) % 16 >= mem::size_of::<u32x4>() {
            //Align 16
            while (dst + i) % 16 != 0 && len - i >= mem::size_of::<u32>() {
                *((dst + i) as *mut u32) = data;
                i += mem::size_of::<u32>();
            }
            //While 16 byte transfers
            let simd: u32x4 = u32x4(data, data, data, data);
            while len - i >= mem::size_of::<u32x4>() {
                *((dst + i) as *mut u32x4) = simd;
                i += mem::size_of::<u32x4>();
            }
        }
        //Everything after last 16 byte transfer
        while len - i >= mem::size_of::<u32>() {
            *((dst + i) as *mut u32) = data;
            i += mem::size_of::<u32>();
        }
    }

    pub unsafe fn copy_run(src: usize, dst: usize, len: usize) {
        let mut i = 0;
        //Only use 16 byte transfer if possible
        if (src + i) % 16 == (dst + i) % 16 {
            //Align 16
            while (dst + i) % 16 != 0 && len - i >= mem::size_of::<u32>() {
                *((dst + i) as *mut u32) = *((src + i) as *const u32);
                i += mem::size_of::<u32>();
            }
            //While 16 byte transfers
            while len - i >= mem::size_of::<u32x4>() {
                *((dst + i) as *mut u32x4) = *((src + i) as *const u32x4);
                i += mem::size_of::<u32x4>();
            }
        }
        //Everything after last 16 byte transfer
        while len - i >= mem::size_of::<u32>() {
            *((dst + i) as *mut u32) = *((src + i) as *const u32);
            i += mem::size_of::<u32>();
        }
    }

    /// Set the color
    pub fn set(&self, color: Color) {
        unsafe {
            Display::set_run(color.data, self.offscreen, self.size);
        }
    }

    /// Scroll the display
    pub fn scroll(&self, rows: usize) {
        if rows > 0 && rows < self.height {
            let offset = rows * self.bytesperrow;
            unsafe {
                Display::copy_run(self.offscreen + offset,
                                  self.offscreen,
                                  self.size - offset);
                Display::set_run(0, self.offscreen + self.size - offset, offset);
            }
        }
    }

    /// Flip the display
    pub fn flip(&self) {
        unsafe {
            let self_mut: *mut Self = mem::transmute(self);
            mem::swap(&mut (*self_mut).offscreen,
                      &mut (*self_mut).onscreen);
        }
    }

    /// Draw a rectangle
    pub fn rect(&self, point: Point, size: Size, color: Color) {
        let data = color.data;
        let alpha = (color.data & 0xFF000000) >> 24;

        if alpha > 0 {
            let start_y = cmp::max(0, cmp::min(self.height as isize - 1, point.y)) as usize;
            let end_y =
                cmp::max(0, cmp::min(self.height as isize - 1, point.y + size.height as isize)) as usize;

            let start_x = cmp::max(0, cmp::min(self.width as isize - 1, point.x)) as usize * 4;
            let len = cmp::max(0, cmp::min(self.width as isize - 1, point.x + size.width as isize)) as usize *
                      4 - start_x;

            if alpha >= 255 {
                for y in start_y..end_y {
                    unsafe {
                        Display::set_run(data,
                                         self.offscreen + y * self.bytesperrow + start_x,
                                         len);
                    }
                }
            } else {
                let n_alpha = 255 - alpha;
                let r = (((data >> 16) & 0xFF) * alpha) >> 8;
                let g = (((data >> 8) & 0xFF) * alpha) >> 8;
                let b = ((data & 0xFF) * alpha) >> 8;
                let premul = (r << 16) | (g << 8) | b;
                for y in start_y..end_y {
                    unsafe {
                        Display::set_run_alpha(premul,
                                               n_alpha,
                                               self.offscreen + y * self.bytesperrow + start_x,
                                               len);
                    }
                }
            }
        }
    }

    /// Set the color of a pixel
    pub fn pixel(&self, point: Point, color: Color) {
        unsafe {
            if point.x >= 0 && point.x < self.width as isize && point.y >= 0 &&
               point.y < self.height as isize {
                *((self.offscreen + point.y as usize * self.bytesperrow + point.x as usize * 4) as *mut u32) = color.data;
            }
        }
    }

    /// Draw an line (without antialiasing) with width 1
    /// (using Bresenham's algorithm)
    pub fn line(&self, point_a: Point, point_b: Point, color: Color) {
        // Calculate delta
        let delta = point_b - point_a;

        if delta.x == 0 { // Handle case where delta x = 0
            // Set offset
            let mut y = point_a.y;

            // While the endpoint isn't reached
            while y != point_b.y {
                // Set pixel
                self.pixel(Point::new(point_a.x, y), color);
                // Increase
                y += 1;
            }
        } else {
            // Find error and error change
            let mut error = 0.0;
            // This is a bit annoying... but libcore does not have .abs() defined on f64 ;-(
            let delta_error = {
                let delta_error_signed = (delta.y as f64) / (delta.x as f64);
                if delta_error_signed < 0.0 {
                    -delta_error_signed
                } else {
                    delta_error_signed
                }
            };

            let mut y = 0;

            for x in point_a.x..point_b.x {
                // Draw pixel
                self.pixel(Point::new(x, y), color);

                // Update error
                error += delta_error;

                while error >= 0.5 {
                    // Draw pixel
                    self.pixel(Point::new(x, y), color);

                    // Update y
                    y += if delta.y > 0 {
                        1
                    } else {
                        -1
                    };

                    // Decrease error
                    error -= 1.0;
                }
            }
        }
    }
    // TODO: Antialiased lines
    // TODO: Lines with other width

    /* Commented because std::f64 is required
    fn line_aa(&self, mut point_a: Point, mut point_b: Point, color: RgbColor) {
        // TODO: Xiaolin Wu line drawing
        use core::mem::swap;

        let steep = (point_b.y - point_a.y).abs() > (point_b.x - point_a.x);

        if steep {
            // Swap the x and y
            swap(&mut point_a.x, &mut point_a.y);
            swap(&mut point_b.x, &mut point_b.y);
        }

        if point_a.x > point_b.x {
            // Swap point_a and point_b
            swap(&mut point_a.x, &mut point_b.x);
            swap(&mut point_a.y, &mut point_b.y);
        }

        // Calculate delta
        let dx = point_b.x - point_a.x;
        let dy = point_b.y - point_a.y;
        // Calculate gradient
        let gradient = (dy as f64) / (dx as f64);

        // First endpoint
        let x_end = point_b.x.round();
        let y_end = point_b.y + gradient * (x_end - point_b.x);

        let x_pxl2 = x_end;
        let y_pxl2 = y_end.floor();

        if steep {
            self
        };

    }
    */

    /// Draw an image
    pub unsafe fn image(&self, point: Point, data: *const u32, size: Size) {
        let start_y = cmp::max(0, point.y) as usize;
        let end_y = cmp::min(self.height as isize, point.y + size.height as isize) as usize;

        let start_x = cmp::max(0, point.x) as usize;
        let len = cmp::min(self.width as isize, point.x + size.width as isize) as usize * 4 -
                  start_x * 4;
        let offscreen_offset = self.offscreen + start_x * 4;

        let bytesperrow = size.width * 4;
        let data_offset = data as usize - start_y * bytesperrow -
                          (point.x - start_x as isize) as usize * 4;

        for y in start_y..end_y {
            Display::copy_run(data_offset + y * bytesperrow,
                              offscreen_offset + y * self.bytesperrow,
                              len);
        }
    }
    /* } Optimized */

    /// Draw a image with opacity
    pub unsafe fn image_alpha(&self, point: Point, data: *const u32, size: Size) {
        let start_y = cmp::max(0, point.y) as usize;
        let end_y = cmp::min(self.height as isize, point.y + size.height as isize) as usize;

        let start_x = cmp::max(0, point.x) as usize;
        let len = cmp::min(self.width as isize, point.x + size.width as isize) as usize * 4 -
                  start_x * 4;
        let offscreen_offset = self.offscreen + start_x * 4;

        let bytesperrow = size.width * 4;
        let data_offset = data as usize - start_y * bytesperrow -
                          (point.x - start_x as isize) as usize * 4;

        for y in start_y..end_y {
            Display::copy_run_alpha(data_offset + y * bytesperrow,
                                    offscreen_offset + y * self.bytesperrow,
                                    len);
        }
    }

    //TODO: SIMD to optimize
    pub unsafe fn set_run_alpha(premul: u32, n_alpha: u32, dst: usize, len: usize) {
        let mut i = 0;
        while len - i >= mem::size_of::<u32>() {
            let orig = *((dst + i) as *const u32);
            let r = (((orig >> 16) & 0xFF) * n_alpha) >> 8;
            let g = (((orig >> 8) & 0xFF) * n_alpha) >> 8;
            let b = ((orig & 0xFF) * n_alpha) >> 8;
            *((dst + i) as *mut u32) = ((r << 16) | (g << 8) | b) + premul;
            i += mem::size_of::<u32>();
        }
    }

    //TODO: SIMD to optimize
    pub unsafe fn copy_run_alpha(src: usize, dst: usize, len: usize) {
        let mut i = 0;
        while len - i >= mem::size_of::<u32>() {
            let new = *((src + i) as *const u32);
            let alpha = (new >> 24) & 0xFF;
            if alpha > 0 {
                if alpha >= 255 {
                    *((dst + i) as *mut u32) = new;
                } else {
                    let n_r = (((new >> 16) & 0xFF) * alpha) >> 8;
                    let n_g = (((new >> 8) & 0xFF) * alpha) >> 8;
                    let n_b = ((new & 0xFF) * alpha) >> 8;

                    let orig = *((dst + i) as *const u32);
                    let n_alpha = 255 - alpha;
                    let o_r = (((orig >> 16) & 0xFF) * n_alpha) >> 8;
                    let o_g = (((orig >> 8) & 0xFF) * n_alpha) >> 8;
                    let o_b = ((orig & 0xFF) * n_alpha) >> 8;

                    *((dst + i) as *mut u32) = ((o_r << 16) | (o_g << 8) | o_b) +
                                               ((n_r << 16) | (n_g << 8) | n_b);
                }
            }
            i += mem::size_of::<u32>();
        }
    }

    /// Draw a char
    pub fn char(&self, font: &Vec<u8>, point: Point, character: char, color: Color) {
        let mut offset = (character as usize)*16;
        for row in 0..16 {
            let row_data = if offset < font.len() { font[offset+row] } else { 0 };
            for col in 0..8 {
                let pixel = (row_data >> (7 - col)) & 1;
                if pixel > 0 {
                    self.pixel(Point::new(point.x + col, point.y + row as isize), color);
                }
            }
        }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            if self.offscreen > 0 {
                sys_unalloc(self.offscreen);
                self.offscreen = 0;
            }
            if self.onscreen > 0 {
                sys_unalloc(self.onscreen);
                self.onscreen = 0;
            }
            self.size = 0;
            self.bytesperrow = 0;
            self.width = 0;
            self.height = 0;
            self.root = false;
        }
    }
}
