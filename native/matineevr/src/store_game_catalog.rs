use anyhow::{Result,ensure};
use std::{collections::HashSet,fs,path::{Path,PathBuf}};

pub struct GameBay {
    pub bay: usize,
    pub system: usize,
    pub count: usize,
    pub path: PathBuf,
}
pub fn parse(directory: &Path,text: &str) -> Result<Vec<GameBay>> {
    ensure!(text.len()<=4096,"Game shelf manifest exceeds capacity");
    let mut bays=Vec::new();let mut used=HashSet::new();
    for line in text.lines() {
        let fields: Vec<_>=line.split('\t').collect();
        ensure!(fields.len()==4,"Invalid game shelf record");
        let bay: usize=fields[0].parse()?;
        let system: usize=fields[1].parse()?;
        let count: usize=fields[2].parse()?;
        ensure!(bay<crate::store_game_racks::MAX_BAYS && used.insert(bay) && system<crate::store_game_signs::ROWS-1
            && (1..=crate::store_game_racks::CAPACITY).contains(&count),"Invalid game shelf allocation");
        let name=fields[3];
        ensure!(name.len()<=100 && name.starts_with("games-bay-") && name.ends_with(".bin")
            && name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b,b'-'|b'.')),"Invalid game shelf file");
        bays.push(GameBay {bay,system,count,path:directory.join(name)});
    }
    Ok(bays)
}
pub fn read(path: &Path) -> Result<Vec<GameBay>> {
    ensure!(fs::metadata(path)?.len()<=4096,"Game shelf manifest exceeds capacity");
    parse(path.parent().unwrap(),&fs::read_to_string(path)?)
}
#[cfg(test)]
mod tests {
    #[test]
    fn private_shelf_manifest_rejects_duplicate_bays_and_escaping_paths() {
        let root=std::path::Path::new("/private/session");
        assert!(super::parse(root,"0\t0\t54\tgames-bay-0.bin\n1\t10\t3\tgames-bay-1.bin").is_ok());
        for bad in ["0\t0\t55\tgames-bay-0.bin","32\t0\t1\tgames-bay-0.bin",
            "0\t11\t1\tgames-bay-0.bin","0\t0\t1\t../games-bay-0.bin",
            "0\t0\t1\tgames-bay-0.bin\n0\t1\t1\tgames-bay-1.bin"] {
            assert!(super::parse(root,bad).is_err());
        }
    }
}
