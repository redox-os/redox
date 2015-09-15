use programs::common::*;

pub struct Application;

impl SessionItem for Application {
    fn main(&mut self, url: URL){
        let mut window = Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(640, 480), "Example Game".to_string());
        let start_time = Duration::monotonic();
        let mut last_time = start_time;
        let mut running = true;
        while running {
            let current_time = Duration::monotonic();
            let run_time = current_time - start_time;
            let frame_time = current_time - last_time;
            last_time = current_time;

            loop {
                match window.poll() {
                    EventOption::Key(key_event) => {
                        if key_event.pressed && key_event.scancode == K_ESC {
                            running = false;
                            break;
                        }
                    },
                    EventOption::None => break,
                    _ => ()
                }
            }

            let content = &mut window.content;
            content.set(Color::new(255, 255, 255));

            { //Draw
                let mut y = 0;
                content.text(Point::new(0, y), &"Running Time".to_string(), Color::new(50, 50, 50));
                y += 16;
                content.text(Point::new(8, y), &String::from_num_signed(run_time.secs as isize), Color::new(50, 50, 50));
                y += 16;
                content.text(Point::new(8, y), &String::from_num(run_time.nanos as usize), Color::new(50, 50, 50));
                y += 32;

                content.text(Point::new(0, y), &"Frame Time".to_string(), Color::new(50, 50, 50));
                y += 16;
                content.text(Point::new(8, y), &String::from_num_signed(frame_time.secs as isize), Color::new(50, 50, 50));
                y += 16;
                content.text(Point::new(8, y), &String::from_num(frame_time.nanos as usize), Color::new(50, 50, 50));
                y += 32;
            }

            content.flip();

            RedrawEvent {
                redraw: REDRAW_ALL
            }.trigger();

            sys_yield();
        }
    }
}

impl Application {
    pub fn new() -> Application {
        Application
    }
}
