// tmpfs is implemented directly in vfs.rs as the default backing store.
// All data lives in RAM (Vec<u8> per inode).
// This module exists as a placeholder for when we separate tmpfs from the VFS.
