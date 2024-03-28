use byteorder::{ByteOrder, LittleEndian};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::mem::size_of;

/// Volume Structure
///
/// File System Layout
/// - Master Boot Record and Partition Table : PSN(0-8191)
/// - Partition Boot Sector and FS Info : PSN(8192-14353) LSN(0-6161)
/// - File Allocation Table : PSN(14354-16383) LSN(6162-8191)
/// - User Data : PSN(16384-8323071) LSN(8192-8314879)
///
/// PSN : Physical Sector Number
/// LSN : Logical Sector Number
///
/// Partition Area:
/// - Master Boot Record and Partition Table
///
/// Regular Area:
/// - System Area
///     - Partition Boot Sector and FS Info
///     - File Allocation Table
/// - User Area
///     - User Data

#[repr(align(512))]
#[derive(Debug, Serialize, Deserialize)]
pub struct PartitionArea {
    pub master_boot_record: MasterBootRecord,
    pub partition1: PartitionTable,
    pub partition2: PartitionTable,
    pub partition3: PartitionTable,
    pub partition4: PartitionTable,
    pub signature_word: SignatureWord,
}

impl PartitionArea {
    pub fn new(data: [u8; 512]) -> Self {
        let (mut start, mut end) = (0, size_of::<MasterBootRecord>());
        let boot: [u8; size_of::<MasterBootRecord>()] = data[start..end]
            .try_into()
            .expect("Invalid MasterBootRecord data");
        start = end;
        end = end + size_of::<PartitionTable>();
        let p1: [u8; size_of::<PartitionTable>()] = data[start..end]
            .try_into()
            .expect("Invalid PartitionTable data");
        start = end;
        end = end + size_of::<PartitionTable>();
        let p2: [u8; size_of::<PartitionTable>()] = data[start..end]
            .try_into()
            .expect("Invalid PartitionTable data");
        start = end;
        end = end + size_of::<PartitionTable>();
        let p3: [u8; size_of::<PartitionTable>()] = data[start..end]
            .try_into()
            .expect("Invalid PartitionTable data");
        start = end;
        end = end + size_of::<PartitionTable>();
        let p4: [u8; size_of::<PartitionTable>()] = data[start..end]
            .try_into()
            .expect("Invalid PartitionTable data");
        start = end;
        end = end + size_of::<SignatureWord>();
        let word: [u8; size_of::<SignatureWord>()] = data[start..end]
            .try_into()
            .expect("Invalid SignatureWord data");
        Self {
            master_boot_record: MasterBootRecord::new(boot),
            partition1: PartitionTable::new(p1),
            partition2: PartitionTable::new(p2),
            partition3: PartitionTable::new(p3),
            partition4: PartitionTable::new(p4),
            signature_word: SignatureWord::new(word),
        }
    }

    pub fn new_with_bincode(data: [u8; 512]) -> Self {
        bincode::deserialize::<Self>(&data[..]).unwrap()
    }
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct MasterBootRecord {
    // #[serde(with = "serde_bytes")]
    // #[serde(
    //     serialize_with = "<[_]>::serialize",
    //     deserialize_with = "<[_]>::deserialize"
    // )]
    #[serde(with = "BigArray")]
    pub not_restricted: [u8; 446],
}

impl MasterBootRecord {
    pub fn new(data: [u8; 446]) -> Self {
        Self {
            not_restricted: data,
        }
    }

    pub fn new_with_bincode(data: [u8; 446]) -> Self {
        bincode::deserialize::<Self>(&data[..]).unwrap()
    }
}

#[repr(align(16))]
#[derive(Debug, Serialize, Deserialize)]
pub struct PartitionTable {
    pub boot_indicator: u8,
    pub starting_head: u8,
    pub starting_sector: u16,
    pub system_id: u8,
    pub ending_head: u8,
    pub ending_sector: u16,
    pub relative_sector: u32,
    pub total_sector: u32,
}

impl PartitionTable {
    pub fn new(data: [u8; 16]) -> Self {
        Self {
            boot_indicator: data[0],
            starting_head: data[1],
            starting_sector: LittleEndian::read_u16(&data[2..4]),
            system_id: data[4],
            ending_head: data[5],
            ending_sector: LittleEndian::read_u16(&data[6..8]),
            relative_sector: LittleEndian::read_u32(&data[8..12]),
            total_sector: LittleEndian::read_u32(&data[12..16]),
        }
    }

    pub fn new_with_bincode(data: [u8; 16]) -> Self {
        bincode::deserialize::<Self>(&data[..]).unwrap()
    }
}

#[repr(align(2))]
#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureWord {
    // #[serde(with = "serde_bytes")]
    #[serde(with = "BigArray")]
    pub _55aa: [u8; 2],
}

impl SignatureWord {
    pub fn new(data: [u8; 2]) -> Self {
        Self { _55aa: data }
    }

    pub fn new_with_bincode(data: [u8; 2]) -> Self {
        bincode::deserialize::<Self>(&data[..]).unwrap()
    }
}

// /// M : The number of reserved sector count
// /// N : The number of sectors per FAT
// pub enum SystemAreaLayout {
//     PartitionBootSector = 0,
//     FsInfoSector = 1,
//     ReservedForBootSector = 2,
//     Reserved1,
//     PartitionBootSectorBackup = 6,
//     FsInfoSectorBackup = 7,
//     ReservedForBootSectorBackup = 8,
//     Reserved2, // 9 to M-1
//     FileAllocationTable1, // M to M+N-1
//     FileAllocationTable2, // M+N to M+2N-1
// }

#[repr(align(512))]
pub struct PartitionBootSector {
    pub jump_command: [u8; 3],
    pub creating_system_identifier: [u8; 8],
    pub sector_size: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub number_of_fats: u8,
    pub number_of_root_directory_entries: u16,
    pub total_sectors: u16,
    pub medium_identifier: u8, // F8h
    pub sectors_per_fat: u16,
    pub sectors_per_track: u16,
    pub number_of_sides: u16,
    pub number_of_hidden_sectors: u32,
    pub total_sectors2: u32,
    pub sectors_per_fat_for_fat32: u32,
    pub extension_flag: u16,
    pub fs_version: u16, // 0000h
    pub root_cluster: u32,
    pub fs_info: u16,
    pub backup_boot_sector: u16,
    pub reserved1: [u8; 12],                // All 00h
    pub physical_disk_number: u8,           // 80h
    pub reserved2: [u8; 1],                 // 00h
    pub extended_boot_record_signature: u8, // 29h
    pub volume_id_number: u32,
    pub volume_label: [u8; 11],
    pub file_system_type: [u8; 8],
    pub reserved3: [u8; 420],
    pub signature_word: [u8; 2], // 55h,AAh
}

/// FS Info Sector
#[repr(align(512))]
pub struct FsInfoSector {
    pub lead_signature: [u8; 4],   // 52h, 52h, 61h, 64h
    pub reserved1: [u8; 480],      // All 00h
    pub struct_signature: [u8; 4], // 72h, 72h, 41h, 61h
    pub free_cluster_count: u32,
    pub next_free_cluster: u32,
    pub reserved2: [u8; 12],
    pub tail_signature: [u8; 4], // 00h, 00h, 55h, AAh
}

/// File Allocation Table
#[repr(align(4))]

pub enum FatEntryValue {
    NotUsed = 0x00000000,
    // Allocated = 00000002h to MAX,
    // Reserved = MAX + 1 to FFFFFFF6h,
    Defective = 0xfffffff7,
    // Eoc = 0xfffffff8 ~ 0xffffffff,
}

/// File Directories
/// Directory Entry Fields
#[repr(align(32))]
pub struct DirectoryEntryField {
    pub name: [u8; 11],
    pub attributes: Attribute,
    pub reserved_for_nt: u8,
    pub created_time_tenth: u8,
    pub created_time: u16,
    pub created_date: u16,
    pub last_access_data: u16,
    pub starting_cluster_number_high: u16,
    pub time_recorded: u16,
    pub date_recorded: u16,
    pub starting_cluster_number_low: u16,
    pub file_length: u32,
}

#[repr(align(1))]
pub enum Attribute {
    ReadOnly = 0x01,
    Hidden = 0x02,
    System = 0x04,
    VolumeId = 0x08,
    Directory = 0x10,
    Archive = 0x20,
}

mod test {
    use super::*;
    #[test]
    fn partition_area_size_valid() {
        use std::mem::size_of;
        assert_eq!(2, size_of::<SignatureWord>());
        assert_eq!(16, size_of::<PartitionTable>());
        assert_eq!(446, size_of::<MasterBootRecord>());
        assert_eq!(512, size_of::<PartitionArea>());

        assert_eq!(512, size_of::<PartitionBootSector>());

        assert_eq!(512, size_of::<FsInfoSector>());

        assert_eq!(4, size_of::<FatEntryValue>());

        assert_eq!(32, size_of::<DirectoryEntryField>());
    }
}
