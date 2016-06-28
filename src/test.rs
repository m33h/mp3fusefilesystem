use std::process::Command;
use std::collections::BTreeMap;


//#[test]
//fn open_file_with_given_dir(){
//    let cmd = Command::new("vlc")
//                        .arg("/home/michal/RustProjects/aaa.mp3")
//                        .output()
//                        .unwrap_or_else(|e|{panic!("Aaaa!")});
//
//}


#[test]
fn map_test(){
    let mut map = BTreeMap::new();
    map.insert(1, "world");
    let key = "hello";
    println!("{}",map[&1]);
}


//rustc --test main.rs -> compile tests, main fn replaced with test runner
