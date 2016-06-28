extern crate fuse;
extern crate libc;
extern crate time;
extern crate id3;
extern crate regex;


use std::env;
use std::path::Path;
use libc::ENOENT;
use time::Timespec;
use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyWrite, ReplyEntry, ReplyEmpty, ReplyAttr, ReplyOpen, ReplyDirectory};
use std::process::Command;
use std::fs;
use id3::Tag;
use regex::Regex;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;


const TTL: Timespec = Timespec { sec: 1, nsec: 0 };              

const CREATE_TIME: Timespec = Timespec { sec: 1381237736, nsec: 0 };

const DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: CREATE_TIME,
    mtime: CREATE_TIME,
    ctime: CREATE_TIME,
    crtime: CREATE_TIME,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};


const TXT_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 65536,
    blocks: 1,
    atime: CREATE_TIME,
    mtime: CREATE_TIME,
    ctime: CREATE_TIME,
    crtime: CREATE_TIME,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

fn getFileAttr(_ino: u64, _size: u64) -> FileAttr{
    let a: FileAttr = FileAttr{
        ino: _ino,
        size: _size,
        blocks: 1,
        atime: CREATE_TIME,
        mtime: CREATE_TIME,
        ctime: CREATE_TIME,
        crtime: CREATE_TIME,
        kind: FileType::RegularFile,
        perm: 0o644,
        nlink: 1,
        uid: 501,
        gid: 20,
        rdev: 0,
        flags: 0,
    };

    return a;
}


struct HelloFS;

impl Filesystem for HelloFS {
    fn lookup (&mut self, _req: &Request, parent: u64, name: &Path, reply: ReplyEntry) {

        let str_path = name.file_name().unwrap().to_str().unwrap();
        let mp3dir = env::args().nth(2).unwrap();

        let mut map = BTreeMap::new();
        let mut map2 = BTreeMap::new();

        let mut n: u64 = 3;
        match fs::read_dir(mp3dir){
            Err(why) => println!("! {:?}", why.kind()),
            Ok(paths) => for path in paths {
                let str_path = path.unwrap().path();
                let re = Regex::new(r"(.*)\.mp3").unwrap();

                if re.is_match(str_path.to_str().expect("aa")){
                    let tag = Tag::read_from_path(str_path.clone()).unwrap();
                    let artist = tag.artist().unwrap().to_string();
                    let album = tag.get("TALB").unwrap().content.text();
                    let title = tag.title().unwrap();
                    map.insert(artist.clone()+" ["+album+"] - "+title+".mp3",str_path.clone());
                    map2.insert(artist.clone()+" ["+album+"] - "+title+".mp3",n);
                    n+=1;
                }

            }
        }

//        if map.contains_key(name.to_str().unwrap()) {
//            let nr: &u64 = map2.get(name.to_str().unwrap()).unwrap();
//            println!("{}", nr);
//        }


        if parent == 1  {
            if name.to_str().unwrap()=="library"{
                reply.entry(&TTL, &getFileAttr(2,65536), 0);
            }
            else{
                if map.contains_key(name.to_str().unwrap()){//.file_name().unwrap().to_str().unwrap()
                    let nr: &u64 = map2.get(name.to_str().unwrap()).unwrap();
                    let u64nr: u64 = *nr;
                    reply.entry(&TTL, &getFileAttr(u64nr,65536), 0);
                }
            }
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr (&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &DIR_ATTR),
            2 => reply.attr(&TTL, &TXT_ATTR),
            i => reply.attr(&TTL, &getFileAttr(i,65536)),
        }
    }

    fn read (&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, _size: u32, reply: ReplyData) {
        if ino==2{
            unsafe{
                let mut abc: String = String::new();
                match fs::read_dir(env::args().nth(2).unwrap()){
                    Err(why) => println!("! {:?}", why.kind()),
                    Ok(paths) => for path in paths {
                        let str_path = path.unwrap().path();
                        let re = Regex::new(r"(.*)\.mp3").unwrap();

                        if re.is_match(str_path.to_str().expect("aa")){
                            let tag = Tag::read_from_path(str_path.clone()).unwrap();
                            let artist = tag.artist().unwrap().to_string();
                            abc.push_str("artist: ");
                            abc.push_str(tag.artist().unwrap());
                            abc.push_str("\n");

                            abc.push_str("title: ");
                            let title = tag.title().unwrap();
                            abc.push_str(title);
                            abc.push_str("\n");

                            abc.push_str("album: ");
                            let album = tag.album().unwrap();
                            abc.push_str(album);
                            abc.push_str("\n\n");

                        }
                    }
                }
                reply.data(&abc.as_bytes());
            }
        }

        else if ino > 2 {
                let mut map = BTreeMap::new();
                let mut i: u64 = 3;

                match fs::read_dir(env::args().nth(2).unwrap()){
                    Err(why) => println!("! {:?}", why.kind()),
                    Ok(paths) => for path in paths {
                        let str_path = path.unwrap().path();

                        let re = Regex::new(r"(.*)\.mp3").unwrap();
                        if re.is_match(str_path.to_str().expect("aa")){
                            map.insert(i,str_path.clone());
                            i+=1;

                        }

                    }
                }


                let mut abc: String = String::new();

                let path = Path::new(map.get(&ino).unwrap().as_path().to_str().unwrap());
                let display = path.display();

                let mut file = match File::open(&path) {
                    // The `description` method of `io::Error` returns a string that
                    // describes the error
                    Err(why) => panic!("couldn't open {}: {}", display,
                                                   why.description()),
                    Ok(file) => file,
                };


//                println!("inode: {}, offset: {}", ino, offset);
                let mut buffer = [0; 65536];
                file.read(&mut buffer[..]);


                reply.data(&buffer);

        } else {
            reply.error(ENOENT);
        }
    }

    fn open(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        reply.opened(_ino,_flags);
    }


    fn readdir (&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, mut reply: ReplyDirectory) {
        let mp3dir = env::args().nth(2).unwrap();
        let mut map = BTreeMap::new();

        match fs::read_dir(mp3dir){
            Err(why) => println!("! {:?}", why.kind()),
            Ok(paths) => for path in paths {
                let str_path = path.unwrap().path();
                let re = Regex::new(r"(.*)\.mp3").unwrap();

                if re.is_match(str_path.to_str().expect("aa")){
                    let tag = Tag::read_from_path(str_path.clone()).unwrap();
                    let artist = tag.artist().unwrap().to_string();
                    let album = tag.get("TALB").unwrap().content.text();
                    let title = tag.title().unwrap();

                    map.insert(artist+" ["+album+"] - "+title+".mp3",str_path.clone());
                }
            }
        }

        if ino == 1 {
            if offset == 0 {

                reply.add(1, 0, FileType::Directory, ".");
                reply.add(1, 1, FileType::Directory, "..");
                reply.add(5, 5, FileType::RegularFile, "library");

                let mut j = 3;
                for i in map.keys(){
                    reply.add(j, j, FileType::RegularFile, i);
                    j+=1;
                }
            }
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }

}



fn main () {
    let mountpoint = env::args_os().nth(1).unwrap();

    fuse::mount(HelloFS, &mountpoint, &[]);

}

//example usage:
//mount:   cargo run /home/michal/RustProjects/rust-fuse /home/michal/Muzyka
//unmount: fusermount -u /home/michal/RustProjects/rust-fuse
//force unmount: sudo umount -l  /home/michal/RustProjects/rust-fuse
