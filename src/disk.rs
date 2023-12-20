use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct BlockId<'a> {
    filename: &'a str,
    blknum: i32,
}

impl<'a> BlockId<'a> {
    pub fn new(filename: &'a str, blknum: i32) -> Self {
        Self{
            filename,
            blknum,
        }
    }
    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn number(&self) -> i32 {
        self.blknum
    }
}


#[derive(Debug)]
pub struct Page {
    byte_buffer: Vec<u8>,
}

impl Page {

    pub fn new(blocksize: usize) -> Self {
        Self { byte_buffer: vec![0;blocksize] }
    }

    pub fn from_byte(byte_array: Vec<u8>) -> Self {
        Self { byte_buffer: byte_array}
    }

    pub fn get_int(&self, offset: usize) -> i32 {
        let data = &self.byte_buffer[offset..offset+4];
        i32::from_be_bytes(data.try_into().unwrap())
    }

    pub fn set_int(&mut self, offset: usize, num: i32) {
        self.byte_buffer[offset..offset+4].copy_from_slice(&num.to_be_bytes())
    }

    pub fn get_byte(&self, offset: usize) -> Vec<u8> {
        let length = self.get_int(offset) as usize;
        self.byte_buffer[offset+4..offset+4+length].to_vec()
    }

    pub fn set_byte(&mut self, offset: usize, bytes: &[u8]) {
        self.set_int(offset, bytes.len() as i32);
        self.byte_buffer[offset+4..offset+4+bytes.len()].copy_from_slice(bytes)
    }

    pub fn get_string(&self, offset: usize) -> String {
        let byte = self.get_byte(offset);
        String::from_utf8(byte).unwrap()
    }

    pub fn set_string(&mut self, offset: usize, s: &str) {
        self.set_byte(offset, s.as_bytes());
    }

    pub fn max_length(strlen: usize) -> usize {
        let bytes_per_char = 1;
        std::mem::size_of::<i32>() + strlen * bytes_per_char
    }

    pub fn contents(&self) -> &[u8] {
        &self.byte_buffer
    }

    pub fn contents_mut(&mut self) -> &mut [u8] {
        &mut self.byte_buffer
    }
}


pub struct FileMgr {
    db_directory: PathBuf,
    blocksize: usize,
    is_new: bool,
    open_files: HashMap<String, File>
}


impl FileMgr {
    pub fn new(db_directory: &Path, blocksize: usize) -> io::Result<Self> {
        let is_new = !db_directory.exists();
        if is_new {
            std::fs::create_dir_all(db_directory)?;
        }

        let mgr = FileMgr {
            db_directory: db_directory.to_path_buf(),
            blocksize,
            is_new,
            open_files: HashMap::new(),
        };

        if is_new {
            for entry in std::fs::read_dir(db_directory)? {
                let path = entry?.path();
                if path.file_name()
                    .map_or(false, |name| name.to_string_lossy().starts_with("temp")) {
                        std::fs::remove_file(path)?;
                    }
            }
        }

        Ok(mgr)
    }

    pub fn read(&mut self, blk: &BlockId, p: &mut Page) -> io::Result<()> {
        let blocksize = self.blocksize as u64;
        let file = self.get_file(&blk.filename)?;
        file.seek(SeekFrom::Start(blk.number() as u64 * blocksize))?;
        file.read_exact(&mut p.contents_mut())?;
        Ok(())
    }

    pub fn write(&mut self, blk: &BlockId, p: &Page) -> io::Result<()> {
        let blocksize = self.blocksize as u64;
        let file = self.get_file(&blk.filename)?;
        file.seek(SeekFrom::Start(blk.number() as u64 * blocksize))?;
        file.write_all(&p.contents())?;
        Ok(())
    }

    fn get_file(&mut self, filename: &str) -> io::Result<&mut File> {
        if !self.open_files.contains_key(&filename.to_string()) {
            let file_path = self.db_directory.join(filename);
            let file = OpenOptions::new().read(true).write(true).create(true).open(file_path)?;
            self.open_files.insert(filename.to_string(), file);
        }
        Ok(self.open_files.get_mut(filename).unwrap())
    }
}
