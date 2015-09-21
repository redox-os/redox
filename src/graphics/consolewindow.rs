use alloc::boxed::*;

use common::event::*;
use common::string::*;
use common::vec::*;

use graphics::color::*;
use graphics::point::*;
use graphics::size::*;
use graphics::window::*;

pub struct ConsoleChar {
    character: char,
    color: Color
}

pub struct ConsoleWindow {
    window: Box<Window>,
    output: Vec<ConsoleChar>,
    scroll: Point,
    wrap: bool
}

impl ConsoleWindow {
    pub fn new(point: Point, size: Size, title: String) -> Box<ConsoleWindow> {
        return box ConsoleWindow {
            window: Window::new(point, size, title),
            output: Vec::new(),
            scroll: Point::new(0, 0),
            wrap: true
        };
    }

    pub fn poll(&mut self) -> EventOption {
        return self.window.poll();
    }

    pub fn print(&mut self, string: &String, color: Color){
        for c in string.chars() {
            self.output.push(ConsoleChar{ character: c, color: color });
        }
    }

    pub fn redraw(&mut self){
        let scroll = self.scroll;

        let mut col = -scroll.x;
        let cols = self.window.content.width as isize / 8;
        let mut row = -scroll.y;
        let rows = self.window.content.height as isize / 16;

        {
            let content = &self.window.content;
            content.set(Color::new(0, 0, 0));

            for c in self.output.iter(){
                if self.wrap && col >= cols {
                    col = -scroll.x;
                    row += 1;
                }

                if c.character == '\n' {
                    col = -scroll.x;
                    row += 1;
                }else if c.character == '\t' {
                    col += 8 - col % 8;
                }else{
                    if col >= 0 && col < cols && row >= 0 && row < rows{
                        content.char(Point::new(8 * col, 16 * row), c.character, c.color);
                    }
                    col += 1;
                }
            }

            if col > -scroll.x {
                col = -scroll.x;
                row += 1;
            }

            if self.wrap && col >= cols {
                col = -scroll.x;
                row += 1;
            }

            content.flip();
            RedrawEvent { redraw: REDRAW_ALL }.to_event().trigger();
        }

        if row >= rows {
            self.scroll.y += row - rows + 1;

            self.redraw();
        }
    }
}
