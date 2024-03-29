use std::{
    error::Error,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use crate::{FatEntryValue, FsInfoSector, PartitionBootSector};

pub const DEFAULT_SECTOR_SIZE: usize = 512;

#[derive(Debug)]
pub struct Fat32 {
    file: File,
    pbs: PartitionBootSector,
    fs_info: FsInfoSector,
    fat1: FileAllocationTable,
    fat2: FileAllocationTable,
}

#[derive(Debug)]
pub struct FileAllocationTable {
    data: Vec<u8>,
    entry: Vec<FatEntry>,
}

#[derive(Debug)]
pub struct FatEntry {
    data: u32,
    value: FatEntryValue,
}

impl Fat32 {
    pub fn init(path: &dyn AsRef<Path>) -> Result<Fat32, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let pbs = get_pbs(&mut file)?;
        let fs_info = get_fs_info(&mut file, &pbs)?;
        let (fat1, fat2) = get_fats(&mut file, &pbs)?;
        let fat32 = Fat32 {
            file,
            pbs,
            fs_info,
            fat1,
            fat2,
        };
        Ok(fat32)
    }

    pub fn sector_size(&self) -> u16 {
        self.pbs.sector_size
    }

    pub fn total_sectors(&self) -> u32 {
        self.pbs.total_sectors2
    }

    pub fn sectors_per_cluster(&self) -> u32 {
        self.pbs.sectors_per_fat_for_fat32
    }

    pub fn cluster_size(&self) -> u32 {
        self.sectors_per_cluster() * (self.sector_size() as u32)
    }

    pub fn root_cluster(&self) -> u32 {
        self.pbs.root_cluster
    }

    pub fn fs_info(&self) -> u16 {
        self.pbs.fs_info
    }
}

fn get_fats(
    file: &mut File,
    pbs: &PartitionBootSector,
) -> Result<(FileAllocationTable, FileAllocationTable), Box<dyn Error>> {
    let sector_size = pbs.sector_size;
    let f1 = pbs.reserved_sector_count;
    let fat_size = (pbs.sectors_per_fat_for_fat32 as usize) * (sector_size as usize);
    let mut fat1_data = vec![0u8; fat_size];
    file.seek(SeekFrom::Start((f1 * sector_size).into()))?;
    file.read(&mut fat1_data)?;
    let mut fat2_data = vec![0u8; fat_size];
    file.read(&mut fat2_data)?;

    assert!(fat1_data.len() % 4 == 0);
    // assert!(fat2_data.len() % 4 == 0);
    assert_eq!(fat1_data.len(), fat2_data.len());

    let (fat1, fat2) = (
        FileAllocationTable {
            data: fat1_data,
            entry: vec![],
        },
        FileAllocationTable {
            data: fat2_data,
            entry: vec![],
        },
    );
    Ok((fat1, fat2))
}

fn get_fs_info(file: &mut File, pbs: &PartitionBootSector) -> Result<FsInfoSector, Box<dyn Error>> {
    let sector_size = pbs.sector_size;
    let fs_info = pbs.fs_info;
    file.seek(SeekFrom::Start((fs_info * sector_size).into()))?;
    let mut fs_info_data = vec![0; sector_size.into()];
    file.read(&mut fs_info_data)?;
    let fs_info = bincode::deserialize::<FsInfoSector>(&fs_info_data[..])?;
    Ok(fs_info)
}

fn get_pbs(file: &mut File) -> Result<PartitionBootSector, Box<dyn Error>> {
    let mut pbs_data = [0; 512];
    file.read(&mut pbs_data)?;
    let pbs = bincode::deserialize::<PartitionBootSector>(&pbs_data[..])?;
    Ok(pbs)
}
