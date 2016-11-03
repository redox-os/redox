pub use self::graphic::GraphicScreen;
pub use self::text::TextScreen;

use orbclient::Event;
use syscall::Result;

mod graphic;
mod text;

pub trait Screen {
    fn width(&self) -> usize;

    fn height(&self) -> usize;

    fn event(&mut self, flags: usize) -> Result<usize>;

    fn map(&self, offset: usize, size: usize) -> Result<usize>;

    fn input(&mut self, event: &Event);

    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn will_block(&self) -> bool;

    fn write(&mut self, buf: &[u8], sync: bool) -> Result<usize>;

    fn seek(&mut self, pos: usize, whence: usize) -> Result<usize>;

    fn sync(&mut self);

    fn redraw(&mut self);
}
