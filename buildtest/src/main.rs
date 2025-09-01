use std::thread;
use std::time::Duration;

fn main() {
    println!("Starting bad code...");

    let mut data = Vec::new();

    // Spawn extra threads to cause context switching + allocations
    for i in 0..4 {
        thread::spawn(move || {
            loop {
                // allocate a big String every iteration
                let s = format!("Thread {} - {}", i, "x".repeat(10_000));
                // push into a Vec that grows infinitely -> memory spike
                let mut v = Vec::new();
                for _ in 0..1000 {
                    v.push(s.clone());
                }
                // sleep a bit so CPU doesn't fully lock
                thread::sleep(Duration::from_millis(10));
            }
        });
    }

    // main thread also does bad allocations
    loop {
        // repeatedly allocate / drop strings
        for i in 0..10_00 {
            let s = format!("Main loop alloc {}", i);
            data.push(s);
        }

        // clear but don't shrink -> keeps allocated capacity
        data.clear();

        // waste some CPU cycles
        let mut sum = 0;
        for i in 0..100_0 {
            sum += i % 7;
        }

        break;
        // println!("sum = {}", sum);
        // thread::sleep(Duration::from_millis(100));
    }
}
