use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;

/// Global VFS instance.
pub static VFS: Mutex<FileSystem> = Mutex::new(FileSystem::new());

/// Inode types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InodeType {
    File,
    Directory,
    Device,
}

/// An inode: metadata for a file or directory.
#[derive(Debug, Clone)]
pub struct Inode {
    pub id: usize,
    pub name: String,
    pub itype: InodeType,
    pub size: usize,
    /// File data (for regular files).
    pub data: Vec<u8>,
    /// Children inode IDs (for directories).
    pub children: Vec<usize>,
    /// Parent inode ID.
    pub parent: usize,
    /// Permissions (rwx as octal, e.g., 0o755).
    pub mode: u16,
}

/// The virtual filesystem.
pub struct FileSystem {
    inodes: Vec<Inode>,
    next_id: usize,
}

impl FileSystem {
    pub const fn new() -> Self {
        Self {
            inodes: Vec::new(),
            next_id: 0,
        }
    }

    /// Initialize with root directory and default structure.
    pub fn init(&mut self) {
        // Root directory (inode 0)
        self.create_inode("/", InodeType::Directory, 0, 0o755);

        // Standard directories
        let root = 0;
        self.mkdir(root, "bin");
        self.mkdir(root, "etc");
        self.mkdir(root, "home");
        self.mkdir(root, "tmp");
        self.mkdir(root, "dev");
        self.mkdir(root, "proc");
        self.mkdir(root, "sys");
        self.mkdir(root, "var");

        // /etc files
        let etc = self.resolve_path("/etc").unwrap();
        self.create_file(etc, "hostname", b"opsys");
        self.create_file(etc, "os-release",
            b"NAME=OpSys\nVERSION=0.1.0\nID=opsys\nPRETTY_NAME=\"OpSys AI-Optimized Microkernel\"\n");
        self.create_file(etc, "motd",
            b"Welcome to OpSys v0.1.0\nAI-Optimized Microkernel Operating System\n");

        // /home/user
        let home = self.resolve_path("/home").unwrap();
        let user = self.mkdir(home, "user");
        self.create_file(user, ".bashrc", b"# OpSys shell config\nexport PS1='$ '\n");
        self.create_file(user, "hello.txt", b"Hello from OpSys!\nThis file lives in the tmpfs.\n");
        self.create_file(user, "readme.txt",
            b"OpSys - AI-Optimized Microkernel OS\n\nBuilt from scratch in Rust.\n7 phases complete.\n");

        // /proc virtual files (populated at read time)
        let proc_dir = self.resolve_path("/proc").unwrap();
        self.create_file(proc_dir, "version", b"OpSys v0.1.0 x86_64 Microkernel (Rust)");
        self.create_file(proc_dir, "cpuinfo", b"vendor: AuthenticAMD\narch: x86_64\nfeatures: sse sse2\n");
    }

    fn create_inode(&mut self, name: &str, itype: InodeType, parent: usize, mode: u16) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.inodes.push(Inode {
            id,
            name: String::from(name),
            itype,
            size: 0,
            data: Vec::new(),
            children: Vec::new(),
            parent,
            mode,
        });
        id
    }

    /// Create a directory under parent. Returns the new inode ID.
    pub fn mkdir(&mut self, parent: usize, name: &str) -> usize {
        let id = self.create_inode(name, InodeType::Directory, parent, 0o755);
        if parent < self.inodes.len() {
            self.inodes[parent].children.push(id);
        }
        id
    }

    /// Create a file under parent with initial data. Returns the inode ID.
    pub fn create_file(&mut self, parent: usize, name: &str, data: &[u8]) -> usize {
        let id = self.create_inode(name, InodeType::File, parent, 0o644);
        self.inodes[id].data = data.to_vec();
        self.inodes[id].size = data.len();
        if parent < self.inodes.len() {
            self.inodes[parent].children.push(id);
        }
        id
    }

    /// Resolve a path string to an inode ID.
    pub fn resolve_path(&self, path: &str) -> Option<usize> {
        if path == "/" { return Some(0); }
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = 0usize; // root

        for part in parts {
            let inode = self.inodes.get(current)?;
            if inode.itype != InodeType::Directory { return None; }
            let mut found = false;
            for &child_id in &inode.children {
                if let Some(child) = self.inodes.get(child_id) {
                    if child.name == part {
                        current = child_id;
                        found = true;
                        break;
                    }
                }
            }
            if !found { return None; }
        }
        Some(current)
    }

    /// Read a file's contents.
    pub fn read_file(&self, inode_id: usize) -> Option<&[u8]> {
        let inode = self.inodes.get(inode_id)?;
        if inode.itype != InodeType::File { return None; }
        Some(&inode.data)
    }

    /// Write data to a file (overwrite).
    pub fn write_file(&mut self, inode_id: usize, data: &[u8]) -> bool {
        if let Some(inode) = self.inodes.get_mut(inode_id) {
            if inode.itype != InodeType::File { return false; }
            inode.data = data.to_vec();
            inode.size = data.len();
            return true;
        }
        false
    }

    /// List children of a directory.
    pub fn list_dir(&self, inode_id: usize) -> Option<Vec<(String, InodeType, usize)>> {
        let inode = self.inodes.get(inode_id)?;
        if inode.itype != InodeType::Directory { return None; }
        let mut entries = Vec::new();
        for &child_id in &inode.children {
            if let Some(child) = self.inodes.get(child_id) {
                entries.push((child.name.clone(), child.itype, child.size));
            }
        }
        Some(entries)
    }

    /// Get inode info.
    pub fn stat(&self, inode_id: usize) -> Option<&Inode> {
        self.inodes.get(inode_id)
    }

    /// Get total filesystem stats.
    pub fn stats(&self) -> (usize, usize, usize) {
        let files = self.inodes.iter().filter(|i| i.itype == InodeType::File).count();
        let dirs = self.inodes.iter().filter(|i| i.itype == InodeType::Directory).count();
        let bytes: usize = self.inodes.iter().map(|i| i.data.len()).sum();
        (files, dirs, bytes)
    }

    /// Remove a file (not directories).
    pub fn remove(&mut self, path: &str) -> bool {
        let id = match self.resolve_path(path) {
            Some(id) => id,
            None => return false,
        };
        if self.inodes[id].itype != InodeType::File { return false; }
        let parent = self.inodes[id].parent;
        self.inodes[parent].children.retain(|&c| c != id);
        // Mark as empty (don't actually remove to keep IDs stable)
        self.inodes[id].data.clear();
        self.inodes[id].size = 0;
        self.inodes[id].name = String::from("(deleted)");
        true
    }
}
