use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

use crate::{vm_client::ClientMessage, vm_server::ServerMessage};

pub struct IO {
    reader: File,
    writer: File,
}

use nix::sys::stat::Mode;
use nix::unistd;
impl IO {
    pub fn new(p: String) -> Self {
        let mut consumer_mode = Mode::empty();
        consumer_mode.insert(Mode::S_IWGRP);
        consumer_mode.insert(Mode::S_IRGRP);
        consumer_mode.insert(Mode::S_IRUSR);
        consumer_mode.insert(Mode::S_IWUSR);
        consumer_mode.insert(Mode::S_IWOTH);
        consumer_mode.insert(Mode::S_IROTH);

        let mut server_mode = Mode::empty();
        server_mode.insert(Mode::S_IWGRP);
        server_mode.insert(Mode::S_IRGRP);
        server_mode.insert(Mode::S_IRUSR);
        server_mode.insert(Mode::S_IWUSR);
        server_mode.insert(Mode::S_IWOTH);
        server_mode.insert(Mode::S_IROTH);
        let mut write_path = p.clone();
        write_path.push_str("_read");
        let writer_path = Path::new(&write_path);

        unistd::mkfifo(writer_path, server_mode).expect("failed to create fifo");
        let mut read_path = p;
        read_path.push_str("_write");
        let reader_path = Path::new(&read_path);
        unistd::mkfifo(reader_path, server_mode).expect("failed to create fifo");

        let reader = std::fs::File::open(read_path).expect("pipe doesnt exist");
        let writer = OpenOptions::create(OpenOptions::new().write(true), false)
            .open(&write_path)
            .expect("failed to create");

        Self { reader, writer }
    }

    pub fn read(&mut self) -> ClientMessage {
        let mut len_bytes = [0u8; std::mem::size_of::<usize>()];
        self.reader
            .read_exact(&mut len_bytes)
            .expect("Bad interop format");
        let len = usize::from_ne_bytes(len_bytes);

        let mut buf = vec![0; len];
        self.reader
            .read_exact(&mut buf[..])
            .expect("Bad interop format");

        serde_json::from_slice(&buf).expect("Bad interop format")
    }
    pub fn write(&mut self, msg: &ServerMessage) {
        let msg = serde_json::to_vec(msg).expect("Failed to write to pipe");
        self.writer
            .write_all(&usize::to_ne_bytes(msg.len()))
            .expect("Failed to write to pipe");
        self.writer
            .write_all(&msg[..])
            .expect("Failed to write to pipe");
        self.writer.flush().expect("Failed to write to pipe")
    }
    pub fn write_with_fail(&mut self, msg: &ServerMessage) -> Result<(), io::Error> {
        let msg = serde_json::to_vec(msg).expect("Failed to write to pipe");
        self.writer.write_all(&usize::to_ne_bytes(msg.len()))?;
        self.writer.write_all(&msg[..])?;
        self.writer.flush()
    }
}
