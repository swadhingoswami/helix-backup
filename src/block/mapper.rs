use std::collections::BTreeMap;

pub struct FileBlockMapper {
    block_size: u32,
    file_size: u64,
}

impl FileBlockMapper {
    pub fn new(block_size: u32) -> Self {
        Self {
            block_size,
            file_size: 0,
        }
    }

    pub fn file_to_block_range(&self, offset: u64, length: u64) -> (u64, u64) {
        let start_block = offset / self.block_size as u64;
        let end_block = (offset + length - 1) / self.block_size as u64;
        (start_block, end_block)
    }

    pub fn offset_in_block(&self, offset: u64) -> u64 {
        offset % self.block_size as u64
    }

    pub fn block_to_offset(&self, block_num: u64) -> u64 {
        block_num * self.block_size as u64
    }

    pub fn blocks_for_size(&self, size: u64) -> u64 {
        size.div_ceil(self.block_size as u64)
    }

    pub fn set_file_size(&mut self, size: u64) {
        self.file_size = size;
    }
}

pub struct ChangeMap {
    #[allow(dead_code)]
    block_size: u32,
    changed_blocks: BTreeMap<u64, Vec<u8>>,
}

impl ChangeMap {
    pub fn new(block_size: u32) -> Self {
        Self {
            block_size,
            changed_blocks: BTreeMap::new(),
        }
    }

    pub fn record_change(&mut self, block_number: u64, data: Vec<u8>) {
        self.changed_blocks.insert(block_number, data);
    }

    pub fn changed_block_count(&self) -> usize {
        self.changed_blocks.len()
    }

    pub fn total_changed_data_size(&self) -> u64 {
        self.changed_blocks.values().map(|v| v.len() as u64).sum()
    }

    pub fn get_changed_blocks(&self) -> &BTreeMap<u64, Vec<u8>> {
        &self.changed_blocks
    }

    pub fn into_changed_blocks(self) -> BTreeMap<u64, Vec<u8>> {
        self.changed_blocks
    }

    pub fn clear(&mut self) {
        self.changed_blocks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_range() {
        let mapper = FileBlockMapper::new(4096);
        assert_eq!(mapper.file_to_block_range(0, 100), (0, 0));
        assert_eq!(mapper.file_to_block_range(0, 4096), (0, 0));
        assert_eq!(mapper.file_to_block_range(0, 4097), (0, 1));
        assert_eq!(mapper.file_to_block_range(4096, 1), (1, 1));
    }

    #[test]
    fn test_blocks_for_size() {
        let mapper = FileBlockMapper::new(4096);
        assert_eq!(mapper.blocks_for_size(0), 0);
        assert_eq!(mapper.blocks_for_size(1), 1);
        assert_eq!(mapper.blocks_for_size(4096), 1);
        assert_eq!(mapper.blocks_for_size(4097), 2);
    }

    #[test]
    fn test_change_map() {
        let mut cm = ChangeMap::new(4096);
        cm.record_change(0, vec![0; 4096]);
        cm.record_change(5, vec![1; 4096]);
        assert_eq!(cm.changed_block_count(), 2);
    }
}
