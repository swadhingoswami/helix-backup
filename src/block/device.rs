use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};

#[cfg(unix)]
use std::os::unix::fs::FileExt;

pub struct BlockDevice {
    file: File,
    block_size: u32,
    device_size: u64,
    path: String,
    writable: bool,
}

impl BlockDevice {
    pub fn open(path: &str, block_size: u32) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(false)
            .open(path)
            .with_context(|| format!("Cannot open block device: {}", path))?;

        let device_size = Self::probe_device_size(&file, path)?;

        log::info!(
            "Opened block device: {} ({} bytes, block size: {})",
            path,
            device_size,
            block_size
        );

        Ok(Self {
            file,
            block_size,
            device_size,
            path: path.to_string(),
            writable: false,
        })
    }

    pub fn open_for_write(path: &str, block_size: u32) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .with_context(|| format!("Cannot open target device for writing: {}", path))?;

        let device_size = Self::probe_device_size(&file, path)?;

        Ok(Self {
            file,
            block_size,
            device_size,
            path: path.to_string(),
            writable: true,
        })
    }

    fn probe_device_size(file: &File, _path: &str) -> Result<u64> {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::io::AsRawFd;

            const BLKGETSIZE64: libc::c_ulong = 0x8008_1272;
            let mut size: u64 = 0;
            let ret = unsafe { libc::ioctl(file.as_raw_fd(), BLKGETSIZE64, &mut size as *mut _) };
            if ret == 0 {
                return Ok(size);
            }
        }

        let metadata = file.metadata()?;
        Ok(metadata.len())
    }

    pub fn read_block(&self, block_num: u64) -> Result<Vec<u8>> {
        let offset = block_num * self.block_size as u64;
        let mut buffer = vec![0u8; self.block_size as usize];

        #[cfg(unix)]
        {
            self.file
                .read_exact_at(&mut buffer, offset)
                .with_context(|| {
                    format!("Failed to read block {} at offset {}", block_num, offset)
                })?;
        }

        #[cfg(not(unix))]
        {
            let mut file = &self.file;
            file.seek(SeekFrom::Start(offset))?;
            file.read_exact(&mut buffer)?;
        }

        Ok(buffer)
    }

    pub fn write_block(&self, block_num: u64, data: &[u8]) -> Result<()> {
        if !self.writable {
            anyhow::bail!("Device not opened for writing");
        }

        let offset = block_num * self.block_size as u64;

        #[cfg(unix)]
        {
            self.file.write_all_at(data, offset).with_context(|| {
                format!("Failed to write block {} at offset {}", block_num, offset)
            })?;
        }

        #[cfg(not(unix))]
        {
            let mut file = &self.file;
            file.seek(SeekFrom::Start(offset))?;
            file.write_all(data)?;
        }

        Ok(())
    }

    pub fn block_count(&self) -> Result<u64> {
        Ok(self.device_size / self.block_size as u64)
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    pub fn device_size(&self) -> u64 {
        self.device_size
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn flush(&self) -> Result<()> {
        self.file.sync_all()?;
        Ok(())
    }
}

impl std::fmt::Debug for BlockDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockDevice")
            .field("path", &self.path)
            .field("block_size", &self.block_size)
            .field("device_size", &self.device_size)
            .field("block_count", &(self.device_size / self.block_size as u64))
            .field("writable", &self.writable)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_write_blocks() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();

        let dev = BlockDevice::open_for_write(path, 4096).unwrap();
        let data = vec![0xAB; 4096];
        dev.write_block(0, &data).unwrap();
        dev.flush().unwrap();

        let read_dev = BlockDevice::open(path, 4096).unwrap();
        let read_data = read_dev.read_block(0).unwrap();
        assert_eq!(read_data[0], 0xAB);
        assert_eq!(read_data.len(), 4096);
    }

    #[test]
    fn test_block_count() {
        let tmp = NamedTempFile::new().unwrap();
        tmp.as_file().set_len(8192).unwrap();
        let path = tmp.path().to_str().unwrap();

        let dev = BlockDevice::open(path, 4096).unwrap();
        assert_eq!(dev.block_count().unwrap(), 2);
    }
}
