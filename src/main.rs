use io_uring::{opcode, types, IoUring};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::os::unix::io::AsRawFd;

fn main() -> io::Result<()> {
    // Open the NVMe device file
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/nvram")?; // Adjust the path to your NVMe device

    // Create a new io_uring instance
    let mut ring = IoUring::new(8)?;

    // Prepare the buffer to write
    let buf = "Hello NVMe!".as_bytes();

    // Get the file descriptor
    let fd = file.as_raw_fd();

    // Prepare the write operation using opcode::Writev (vectorized write)
    let entry = opcode::Write::new(
        types::Fd(fd),
        buf.as_ptr(),
        buf.len() as u32,
    )
    .build()
    .user_data(0x42); // user_data can be used to identify the operation later

    // Get the submission queue
    unsafe {
        ring.submission()
            .push(&entry)
            .expect("submission queue is full");
    }

    // Submit the operation and wait for its completion
    ring.submit_and_wait(1)?;

    // Retrieve the completion event
    let cqe = ring.completion().next().expect("Completion queue entry is missing");

    // Check the result
    if cqe.result() >= 0 {
        println!("Write to NVMe device successful.");
    } else {
        eprintln!("Error writing to NVMe device: {}", cqe.result());
    }

    Ok(())
}
