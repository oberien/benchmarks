use std::thread;

fn main() {
    let t = thread::spawn(|| test());
    test();
    t.join();
}

fn test() {
    let time = std::time::Instant::now();
    for _ in 0..100_000 {
        unsafe {
            if (libc::fork() != 0) {
                libc::wait(std::ptr::null_mut());
            } else {
                libc::exit(0);
            }
        }
    }
    println!("{:?}", time.elapsed());
}
