use anyhow::{Result,ensure};
use std::{fs,path::{Path,PathBuf}};
#[derive(Clone)]
pub struct AlbumBay {pub bay: usize,pub genre: usize,pub count: usize,pub first: usize,pub path: PathBuf}
pub fn parse(root: &Path,text: &str) -> Result<Vec<AlbumBay>> {
    let mut bays=Vec::new();let mut first=0;
    for line in text.lines() {
        let fields: Vec<_>=line.split('\t').collect();ensure!(fields.len()==5,"Invalid album shelf manifest");
        let bay=fields[0].parse::<usize>()?;let genre=fields[1].parse::<usize>()?;
        let count=fields[2].parse::<usize>()?;let offset=fields[3].parse::<usize>()?;
        ensure!(bay==bays.len() && bay<crate::store_album_racks::MAX_BAYS && genre<crate::store_album_signs::GENRES && (1..=54).contains(&count) && offset==first,"Invalid album shelf allocation");
        ensure!(fields[4]==format!("albums-bay-{bay}.bin"),"Invalid album shelf path");
        bays.push(AlbumBay {bay,genre,count,first,path:root.join(fields[4])});first+=count;
    }
    Ok(bays)
}
pub fn read(path: &Path) -> Result<Vec<AlbumBay>> {
    ensure!(fs::metadata(path)?.len()<8192,"Album manifest exceeds capacity");
    parse(path.parent().unwrap(),&fs::read_to_string(path)?)
}
#[cfg(test)]
mod tests {
    #[test]
    fn album_sections_reject_path_escape_duplicate_slots_and_wrong_selection_offsets() {
        let root=std::path::Path::new("/private/session");
        assert!(super::parse(root,"0\t0\t54\t0\talbums-bay-0.bin\n1\t15\t1\t54\talbums-bay-1.bin").is_ok());
        for bad in ["0\t0\t1\t0\t../albums-bay-0.bin","0\t16\t1\t0\talbums-bay-0.bin",
            "0\t0\t1\t2\talbums-bay-0.bin","0\t0\t55\t0\talbums-bay-0.bin",
            "0\t0\t1\t0\talbums-bay-0.bin\n0\t0\t1\t1\talbums-bay-0.bin"] {assert!(super::parse(root,bad).is_err());}
    }
}
