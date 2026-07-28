use core::panic::PanicInfo;

use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};
use toolkit::{Directory, File, FilePath, Sleeper, U16CStackString, print, println};

use crate::{types::{AncestorPath, AncestorPaths, DirectoryNode}};

pub fn search_via_nt_api() {
    let path = U16CStackString::<260>::from_str(r#"\??\C:\"#).unwrap();
    let mut directories: Vec<FilePath<260>> = Vec::new();
    
    let root = FilePath::<260>::from_slice(path.as_slice());
    directories.push(root);
    
    let mut node_map: BTreeMap<AncestorPath, DirectoryNode> = BTreeMap::new();
    let root_path = AncestorPath::from_slice(path.as_slice());
    node_map.insert(root_path, DirectoryNode::default());
    
    while let Some(file_path) = directories.pop() {
        let current = match Directory::open(file_path.as_slice()) {
            Ok(value) => value,
            Err(err) => {
                println!("{} {:?}", file_path, err);
                continue;
            },
        };
        
        let parent_path = AncestorPath::from_slice(file_path.as_slice());
        
        for entry in current.query() {
            let entry = match entry {
                Ok(e) => e,
                Err(err) => {
                    println!("{:?}", err);
                    continue;
                }
            };
            
            if entry.is_reparse_point() {
                continue;
            }
            
            if entry.is_directory() {
                if entry.is_self() || entry.is_parent() {
                    continue;
                }
                
                let child_path = AncestorPath::from_slice(entry.file_path().as_slice());
                
                if let Some(parent_node) = node_map.get_mut(&parent_path) {
                    parent_node.children.push(child_path.clone());
                }
                
                node_map.entry(child_path).or_default();
                
                let file_path = entry.file_path();
                directories.push(file_path);
                continue;
            }
            
            let size = entry.end_of_file().0;
            let file_path = entry.file_path();
            let path_slice = file_path.as_slice();
            let iter = AncestorPaths::new(path_slice);
            
            for ancestor in iter {
                if let Some(node) = node_map.get_mut(&ancestor) {
                    node.stats.file_count += 1;
                    node.stats.file_size += size;
                }
            }
        }
    }
    
    print_tree(&node_map, &AncestorPath::from_slice(path.as_slice()), 0);
}

fn print_tree(map: &BTreeMap<AncestorPath, DirectoryNode>, path: &AncestorPath, depth: usize) {
    let indent = "  ".repeat(depth);
    if let Some(node) = map.get(path) {
        println!("{}{} - {} files, {} bytes", 
                 indent, path, node.stats.file_count, node.stats.file_size);
        
        for child in &node.children {
            print_tree(map, child, depth + 1);
        }
    }
}