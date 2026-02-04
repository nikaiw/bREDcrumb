#[derive(Debug, Clone)]
pub struct CodeCave {
    pub file_offset: usize,
    pub size: usize,
    pub virtual_address: Option<u64>,
    pub section_name: Option<String>,
}

pub struct CaveFinder;

impl CaveFinder {
    /// Find code caves (sequences of null bytes) in the given data
    pub fn find_caves(data: &[u8], min_size: usize) -> Vec<CodeCave> {
        let mut caves = Vec::new();
        let mut cave_start: Option<usize> = None;
        let mut null_count = 0;

        for (i, &byte) in data.iter().enumerate() {
            if byte == 0 {
                if cave_start.is_none() {
                    cave_start = Some(i);
                }
                null_count += 1;
            } else {
                if let Some(start) = cave_start {
                    if null_count >= min_size {
                        caves.push(CodeCave {
                            file_offset: start,
                            size: null_count,
                            virtual_address: None,
                            section_name: None,
                        });
                    }
                }
                cave_start = None;
                null_count = 0;
            }
        }

        // Check for cave at end of data
        if let Some(start) = cave_start {
            if null_count >= min_size {
                caves.push(CodeCave {
                    file_offset: start,
                    size: null_count,
                    virtual_address: None,
                    section_name: None,
                });
            }
        }

        // Sort by size descending
        caves.sort_by(|a, b| b.size.cmp(&a.size));
        caves
    }

    /// Find caves within a specific range
    pub fn find_caves_in_range(
        data: &[u8],
        start: usize,
        end: usize,
        min_size: usize,
    ) -> Vec<CodeCave> {
        if start >= data.len() || end > data.len() || start >= end {
            return Vec::new();
        }

        let slice = &data[start..end];
        let mut caves = Self::find_caves(slice, min_size);

        // Adjust offsets to be relative to original data
        for cave in &mut caves {
            cave.file_offset += start;
        }

        caves
    }

    /// Find the best cave that can fit the given size
    pub fn find_best_cave(data: &[u8], needed_size: usize) -> Option<CodeCave> {
        let caves = Self::find_caves(data, needed_size);
        caves.into_iter().next()
    }

    /// Get the largest cave found
    pub fn largest_cave_size(data: &[u8]) -> usize {
        Self::find_caves(data, 1)
            .first()
            .map(|c| c.size)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_caves() {
        let data = vec![0x41, 0x00, 0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x43];
        let caves = CaveFinder::find_caves(&data, 2);

        assert_eq!(caves.len(), 2);
        assert_eq!(caves[0].file_offset, 1);
        assert_eq!(caves[0].size, 4);
        assert_eq!(caves[1].file_offset, 6);
        assert_eq!(caves[1].size, 2);
    }

    #[test]
    fn test_find_best_cave() {
        let data = vec![0x41, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00, 0x00, 0x43];
        let cave = CaveFinder::find_best_cave(&data, 3);

        assert!(cave.is_some());
        let cave = cave.unwrap();
        assert_eq!(cave.file_offset, 4);
        assert_eq!(cave.size, 4);
    }
}
