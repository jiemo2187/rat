use std::{
    env, fs,
    io::{Read, Seek, SeekFrom},
};

use log::{info, warn};
use rat::{FsInfoSector, PartitionBootSector};

#[test]
fn partition_boot_sector() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .try_init();

    let current_dir = env::current_dir().unwrap();
    let img_dir = current_dir.join("fat32_image.img");
    let mut file = fs::File::open(img_dir).unwrap();
    let mut data = [0; 512];
    file.read(&mut data).unwrap();
    let pbs = bincode::deserialize::<PartitionBootSector>(&data[..]).unwrap();
    info!(
        "Creating System Identifier: {:?}",
        String::from_utf8(pbs.creating_system_identifier.to_vec()).unwrap()
    );
    info!("Sector Size: 0x{:x}", pbs.sector_size);
    info!("Sectors per Cluster: {}", pbs.sectors_per_cluster);
    info!("Reserved Sector Count: {}", pbs.reserved_sector_count);
    info!("Number of FATs: {}", pbs.number_of_fats);
    info!(
        "Number of Root-directory Entries: {}",
        pbs.number_of_root_directory_entries
    );
    info!("Total Sectors: {}", pbs.total_sectors);
    assert_eq!(0xf8, pbs.medium_identifier);
    info!("Sectors per FAT: {}", pbs.sectors_per_fat);
    info!("Sectors per Track: {}", pbs.sectors_per_track);
    info!("Number of Sides: {}", pbs.number_of_sides);
    info!("Number of Hidden Sectors: {}", pbs.number_of_hidden_sectors);
    info!("Total Sectors: {}", pbs.total_sectors2);
    info!(
        "Sectors per FAT for FAT32: {}",
        pbs.sectors_per_fat_for_fat32
    );
    info!("Extension Flag: {}", pbs.extension_flag);
    assert_eq!(0x0000, pbs.fs_version);
    info!("Root Cluster: {}", pbs.root_cluster);
    info!("FS Info: {}", pbs.fs_info);
    info!("Backup Boot Sector: {}", pbs.backup_boot_sector);
    assert_eq!(0x80, pbs.physical_disk_number);
    assert_eq!(0x29, pbs.extended_boot_record_signature);
    info!("Volume ID Number: {}", pbs.volume_id_number);
    info!(
        "Volume Label: {:?}",
        String::from_utf8(pbs.volume_label.to_vec()).unwrap()
    );
    info!(
        "File System Type: {:?}",
        String::from_utf8(pbs.file_system_type.to_vec()).unwrap()
    );
    assert_eq!([0x55, 0xaa], pbs.signature_word);
}

#[test]
fn back_partition_boot_sector() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .try_init();

    let current_dir = env::current_dir().unwrap();
    let img_dir = current_dir.join("fat32_image.img");
    let mut file = fs::File::open(img_dir).unwrap();
    let mut pbs_data = [0; 512];
    file.read(&mut pbs_data).unwrap();
    let pbs = bincode::deserialize::<PartitionBootSector>(&pbs_data[..]).unwrap();
    let sector_size = pbs.sector_size;
    let backup_boot_sector = pbs.backup_boot_sector;
    // 跳转到 Partition Boot Sector
    file.seek(SeekFrom::Start((backup_boot_sector * sector_size).into()))
        .unwrap();
    let mut backup_pbs_data = vec![0; sector_size.into()];
    file.read(&mut backup_pbs_data).unwrap();
    assert_eq!(backup_pbs_data, pbs_data);
}

#[test]
fn fs_info_sector() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .try_init();

    let current_dir = env::current_dir().unwrap();
    let img_dir = current_dir.join("fat32_image.img");
    let mut file = fs::File::open(img_dir).unwrap();
    let mut pbs_data = [0; 512];
    file.read(&mut pbs_data).unwrap();
    let pbs = bincode::deserialize::<PartitionBootSector>(&pbs_data[..]).unwrap();
    let sector_size = pbs.sector_size;
    let fs_info = pbs.fs_info;
    // 跳转到 FS Info Sector
    file.seek(SeekFrom::Start((fs_info * sector_size).into()))
        .unwrap();

    let mut fs_info_data = vec![0; sector_size.into()];
    file.read(&mut fs_info_data).unwrap();
    let fis = bincode::deserialize::<FsInfoSector>(&fs_info_data[..]).unwrap();

    assert_eq!([0x52, 0x52, 0x61, 0x41], fis.lead_signature);
    info!("free cluster count: {}", fis.free_cluster_count);
    assert_eq!([0x72, 0x72, 0x41, 0x61], fis.struct_signature);
    info!("next free cluster: {}", fis.next_free_cluster);
    assert_eq!([0x00, 0x00, 0x55, 0xaa], fis.tail_signature);
}

#[test]
fn back_fs_info_sector() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .try_init();

    let current_dir = env::current_dir().unwrap();
    let img_dir = current_dir.join("fat32_image.img");
    let mut file = fs::File::open(img_dir).unwrap();
    let mut pbs_data = [0; 512];
    file.read(&mut pbs_data).unwrap();
    let pbs = bincode::deserialize::<PartitionBootSector>(&pbs_data[..]).unwrap();
    let sector_size = pbs.sector_size;
    info!("Backup Boot Sector: {}", pbs.backup_boot_sector);
    let backup_boot_sector = pbs.backup_boot_sector;
    let backup_fs_info = backup_boot_sector + 1;
    // 跳转到 FS Info Sector
    file.seek(SeekFrom::Start((backup_fs_info * sector_size).into()))
        .unwrap();
    let mut backup_fs_info_data = vec![0; sector_size.into()];
    file.read(&mut backup_fs_info_data).unwrap();

    let fis = bincode::deserialize::<FsInfoSector>(&backup_fs_info_data[..]).unwrap();
    if [0x52, 0x52, 0x61, 0x41] != fis.lead_signature {
        warn!("No Backup FS Info Sector");
    }
}

#[test]
fn file_allocation_table_compare() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .try_init();

    let current_dir = env::current_dir().unwrap();
    let img_dir = current_dir.join("fat32_image.img");
    let mut file = fs::File::open(img_dir).unwrap();
    let mut data = [0; 512];
    file.read(&mut data).unwrap();
    let pbs = bincode::deserialize::<PartitionBootSector>(&data[..]).unwrap();

    info!("Reserved Sector Count: {}", pbs.reserved_sector_count);
    info!(
        "Sectors per FAT for FAT32: {}",
        pbs.sectors_per_fat_for_fat32
    );

    let sector_size = pbs.sector_size;
    // 预留扇区后面就是 File Allocation Table1
    let f1 = pbs.reserved_sector_count;
    let fat_size = (pbs.sectors_per_fat_for_fat32 as usize) * (sector_size as usize);
    let mut fat1_data = vec![0u8; fat_size];
    file.seek(SeekFrom::Start((f1 * sector_size).into()))
        .unwrap();
    file.read(&mut fat1_data).unwrap();
    let mut fat2_data = vec![0u8; fat_size];
    file.read(&mut fat2_data).unwrap();
    assert_eq!(fat1_data, fat2_data);
}

#[test]
fn file_allocation_table() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .try_init();

    let current_dir = env::current_dir().unwrap();
    let img_dir = current_dir.join("fat32_image.img");
    let mut file = fs::File::open(img_dir).unwrap();
    let mut data = [0; 512];
    file.read(&mut data).unwrap();
    let pbs = bincode::deserialize::<PartitionBootSector>(&data[..]).unwrap();

    info!("Reserved Sector Count: {}", pbs.reserved_sector_count);
    info!(
        "Sectors per FAT for FAT32: {}",
        pbs.sectors_per_fat_for_fat32
    );

    let sector_size = pbs.sector_size;
    // 预留扇区后面就是 File Allocation Table1
    let f1 = pbs.reserved_sector_count;
    let fat_size = (pbs.sectors_per_fat_for_fat32 as usize) * (sector_size as usize);
    let mut fat1_data = vec![0u8; fat_size];
    file.seek(SeekFrom::Start((f1 * sector_size).into()))
        .unwrap();
    file.read(&mut fat1_data).unwrap();

    let cluster = bincode::deserialize::<u32>(&fat1_data[0..4]).unwrap();

    let value = pbs.fat_entry_value(cluster);
    info!("cluster: 0x{:x}, value : {:?}", cluster, value);
}
