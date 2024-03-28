use std::{env, fs, io::Read};

use rat::PartitionArea;

#[test]
fn signature_word_55aa() {
    let current_dir = env::current_dir().unwrap();
    let img_dir = current_dir.join("fat32_image.img");
    let mut file = fs::File::open(img_dir).unwrap();
    let mut data = [0; 512];
    file.read(&mut data).unwrap();
    let partition_area = bincode::deserialize::<PartitionArea>(&data[..]).unwrap();
    assert_eq!(partition_area.signature_word._55aa, [0x55, 0xaa]);
}
