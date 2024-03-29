use std::env;

use rat::{Fat32, DEFAULT_SECTOR_SIZE};

#[test]
fn init_test() {
    let current_dir = env::current_dir().unwrap();
    let img_dir = current_dir.join("fat32_image.img");
    let fat32 = Fat32::init(&img_dir).unwrap();
    assert_eq!(DEFAULT_SECTOR_SIZE, fat32.sector_size().into());
}
