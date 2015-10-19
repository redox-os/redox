use redox::*;

pub fn main() {
	match File::open("display://") {
		Some(ref mut file) => {
			let i:[u8;4] = [0, 128, 128, 128];
			let mut x = 0;
			loop {
				file.write(&i);
				x += 1;
				if x > 4096 {
					break;
				}
			}
			file.sync();
			let z = time::Duration::new(10,0);
			z.sleep();
		},
		None => { 
			loop {
				println!("Welp, that didn't work...");
			}
		}
	}
}
