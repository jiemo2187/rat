use std::{
    env, fs,
    io::{Read, Seek, SeekFrom},
};

use rat::{FsInfoSector, PartitionBootSector};

#[test]
fn partition_boot_sector() {
    let current_dir = env::current_dir().unwrap();
    let img_dir = current_dir.join("fat32_image.img");
    let mut file = fs::File::open(img_dir).unwrap();
    let mut data = [0; 512];
    file.read(&mut data).unwrap();
    let pbs = bincode::deserialize::<PartitionBootSector>(&data[..]).unwrap();
    println!(
        "Creating System Identifier: {:?}",
        String::from_utf8(pbs.creating_system_identifier.to_vec()).unwrap()
    );
    println!("Sector Size: 0x{:x}", pbs.sector_size);
    println!("Sectors per Cluster: {}", pbs.sectors_per_cluster);
    println!("Reserved Sector Count: {}", pbs.reserved_sector_count);
    println!("Number of FATs: {}", pbs.number_of_fats);
    println!(
        "Number of Root-directory Entries: {}",
        pbs.number_of_root_directory_entries
    );
    println!("Total Sectors: {}", pbs.total_sectors);
    assert_eq!(0xf8, pbs.medium_identifier);
    println!("Sectors per FAT: {}", pbs.sectors_per_fat);
    println!("Sectors per Track: {}", pbs.sectors_per_track);
    println!("Number of Sides: {}", pbs.number_of_sides);
    println!("Number of Hidden Sectors: {}", pbs.number_of_hidden_sectors);
    println!("Total Sectors: {}", pbs.total_sectors2);
    println!(
        "Sectors per FAT for FAT32: {}",
        pbs.sectors_per_fat_for_fat32
    );
    println!("Extension Flag: {}", pbs.extension_flag);
    assert_eq!(0x0000, pbs.fs_version);
    println!("Root Cluster: {}", pbs.root_cluster);
    println!("FS Info: {}", pbs.fs_info);
    println!("Backup Boot Sector: {}", pbs.backup_boot_sector);
    assert_eq!(0x80, pbs.physical_disk_number);
    assert_eq!(0x29, pbs.extended_boot_record_signature);
    println!("Volume ID Number: {}", pbs.volume_id_number);
    println!(
        "Volume Label: {:?}",
        String::from_utf8(pbs.volume_label.to_vec()).unwrap()
    );
    println!(
        "File System Type: {:?}",
        String::from_utf8(pbs.file_system_type.to_vec()).unwrap()
    );
    assert_eq!([0x55, 0xaa], pbs.signature_word);
}

#[test]
fn fs_info_sector() {
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
    println!("free cluster count: {}", fis.free_cluster_count);
    assert_eq!([0x72, 0x72, 0x41, 0x61], fis.struct_signature);
    println!("next free cluster: {}", fis.next_free_cluster);
    assert_eq!([0x00, 0x00, 0x55, 0xaa], fis.tail_signature);
}
