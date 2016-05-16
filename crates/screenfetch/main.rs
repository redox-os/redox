#![deny(warnings)]

use std::fs::File;
use std::time::Instant;

fn main() {
    let uptime = Instant::now().inner().as_secs();

    let mut width = 0;
    let mut height = 0;
    if let Ok(display) = File::open("display:") {
        let path = display.path().map(|path| path.into_os_string().into_string().unwrap_or(String::new())).unwrap_or(String::new());
        let res = path.split(":").nth(1).unwrap_or("");
        width = res.split("/").nth(0).unwrap_or("").parse::<i32>().unwrap_or(0);
        height = res.split("/").nth(1).unwrap_or("").parse::<i32>().unwrap_or(0);
    }

    println!("\x1B[1;38;5;75m                `.-/+NMN+-.`                   \x1B[0m\x1B[1;38;5;75mroot\x1B[0m@\x1B[1;38;5;75mhostname\x1B[0m");
    println!("\x1B[1;38;5;75m           `:+oo+/-.-yds--/+oo+:`              \x1B[0m\x1B[1;38;5;75mOS:\x1B[0m redox-os");
    println!("\x1B[1;38;5;75m        `/ss/++::/+o++++o+/:```:ss/`           \x1B[0m\x1B[1;38;5;75mKernel:\x1B[0m redox");
    println!("\x1B[1;38;5;75m        `/ss/++::/+o++++o+/:```:ss/`           \x1B[0m\x1B[1;38;5;75mUptime:\x1B[0m {}s", uptime);
    println!("\x1B[1;38;5;75m      `+h+``oMMN+.````````.:+syyy:/h+`         \x1B[0m\x1B[1;38;5;75mShell:\x1B[0m ion");
    println!("\x1B[1;38;5;75m     /h/+mmm/://:+oo+//+oo+:. hNNh.`/h/        \x1B[0m\x1B[1;38;5;75mResolution:\x1B[0m {}x{}", width, height);
    println!("\x1B[1;38;5;75m    oy` ydds`/s+:`        `:+s/-.+Ndd-so       \x1B[0m\x1B[1;38;5;75mDE:\x1B[0m orbital");
    println!("\x1B[1;38;5;75m   os `yo  /y:                :y/.dmM- so      \x1B[0m\x1B[1;38;5;75mWM:\x1B[0m orbital");
    println!("\x1B[1;38;5;75m  :h  s+  os`   \x1B[0m smhhhyyy/  \x1B[1;38;5;75m   `so  +s  h:     \x1B[0m\x1B[1;38;5;75mFont:\x1B[0m unifont");
    println!("\x1B[1;38;5;75m  m. -h  /h     \x1B[0m yM    .oM+ \x1B[1;38;5;75m     h/  h- .m     \x1B[0m");
    println!("\x1B[1;38;5;75m  N  s+  d.     \x1B[0m yM     -Ms \x1B[1;38;5;75m     .d  +s  m     \x1B[0m");
    println!("\x1B[1;38;5;75m  h  y/  M      \x1B[0m yM :+sydy` \x1B[1;38;5;75m      M  /y  h     \x1B[0m");
    println!("\x1B[1;38;5;75m  M  oo  y/     \x1B[0m yM .yNy.   \x1B[1;38;5;75m     /y  oo  M     \x1B[0m");
    println!("\x1B[1;38;5;75m  y/ `m` .d.    \x1B[0m yM   :md-  \x1B[1;38;5;75m    .d.:hNy /y     \x1B[0m");
    println!("\x1B[1;38;5;75m  .d` :h:--h:   \x1B[0m +s    `ss` \x1B[1;38;5;75m   :h- oMNh`d.     \x1B[0m");
    println!("\x1B[1;38;5;75m   :d.-MMN:.oo:              :oo.+sd+..d:      \x1B[0m");
    println!("\x1B[1;38;5;75m    -d//oyy////so/:oyo..ydhos/. +MMM::d-       \x1B[0m");
    println!("\x1B[1;38;5;75m     `sy- yMMN. `./MMMo+dNm/ ./ss-./ys`        \x1B[0m");
    println!("\x1B[1;38;5;75m       .ss/++:+oo+//:-..:+ooo+-``:ss.          \x1B[0m");
    println!("\x1B[1;38;5;75m         `:ss/-` `.--::--.` `-/ss:`            \x1B[0m");
    println!("\x1B[1;38;5;75m             ./oooooooooooooo/.                \x1B[0m");
}
