use crate::filesystem::vfs::{self, VfsEntry, VfsError, VfsNode, VfsPath};
use alloc::{collections::btree_map::BTreeMap, string::String, sync::Arc, vec::Vec};
use spin::Mutex;

pub type TempFilesystemError = &'static str;

pub struct TempFilesystem {
    root: Arc<TempFsDirectory>,
}

impl Default for TempFilesystem {
    fn default() -> Self {
        let name = String::from("/");
        let root_node = TempFsDirectory::new(name);
        assert!(
            root_node.metadata().is_dir,
            "Root node must be a directory!"
        );

        Self {
            root: Arc::new(root_node),
        }
    }
}

impl vfs::VirtualFilesystem for TempFilesystem {
    fn root(&self) -> Arc<dyn vfs::VfsDirectory> {
        self.root.clone()
    }

    fn resolve_path(&self, path: &VfsPath) -> vfs::VfsResult<VfsEntry> {
        // We ignore non absolute paths for now
        if !path.is_absolute() {
            return Err(vfs::VfsError::NotFound);
        }
        let mut current = VfsEntry::Directory(self.root());
        for component in path.components() {
            let dir = current.into_directory()?;
            let next = dir.find(component)?;

            current = next;
        }

        Ok(current)
    }
}

struct TempFsFile {
    name: String,
    data: Mutex<Vec<u8>>,
}

impl TempFsFile {
    pub fn new(name: String) -> Self {
        Self {
            name,
            data: Mutex::new(Vec::new()),
        }
    }
}

impl vfs::VfsNode for TempFsFile {
    fn metadata(&self) -> vfs::VirtualFileMetadata {
        vfs::VirtualFileMetadata {
            is_dir: false,
            name: self.name.clone(),
        }
    }
}

impl vfs::VfsFile for TempFsFile {
    #[allow(clippy::indexing_slicing)]
    fn read(&self, offset: usize, buffer: &mut [u8]) -> vfs::VfsResult<usize> {
        let data = self.data.lock();

        if offset >= data.len() {
            return Err(vfs::VfsError::OffsetOutsideData);
        }

        let end = usize::min(offset + buffer.len(), data.len());
        let size = end - offset;

        buffer[..size].copy_from_slice(&data[offset..end]);

        Ok(size)
    }

    #[allow(clippy::indexing_slicing)]
    fn write(&self, offset: usize, buffer: &[u8]) -> vfs::VfsResult<usize> {
        let mut data = self.data.lock();

        if offset >= buffer.len() {
            return Err(vfs::VfsError::OffsetOutsideData);
        }

        let required_len = offset + buffer.len();
        if data.len() < required_len {
            data.resize(required_len, 0);
        }

        let end = offset + buffer.len();

        data[offset..end].copy_from_slice(buffer);

        Ok(buffer.len())
    }
}

struct TempFsDirectory {
    name: String,
    children: Mutex<BTreeMap<String, vfs::VfsEntry>>,
}

impl TempFsDirectory {
    pub fn new(name: String) -> Self {
        Self {
            name,
            children: Mutex::new(BTreeMap::new()),
        }
    }
}

impl vfs::VfsNode for TempFsDirectory {
    fn metadata(&self) -> vfs::VirtualFileMetadata {
        vfs::VirtualFileMetadata {
            is_dir: true,
            name: self.name.clone(),
        }
    }
}

impl vfs::VfsDirectory for TempFsDirectory {
    fn create(&self, metadata: vfs::VirtualFileMetadata) -> vfs::VfsResult<VfsEntry> {
        let mut children = self.children.lock();
        let already_exists = children.contains_key(&metadata.name);
        if already_exists {
            return Err(vfs::VfsError::AlreadyExists);
        }

        let name = metadata.name.clone();
        let new_child = if metadata.is_dir {
            VfsEntry::Directory(Arc::new(TempFsDirectory::new(metadata.name)))
        } else {
            VfsEntry::File(Arc::new(TempFsFile::new(metadata.name)))
        };
        children.insert(name, new_child.clone());

        Ok(new_child)
    }

    fn find(&self, name: &str) -> vfs::VfsResult<VfsEntry> {
        self.children
            .lock()
            .get(name)
            .cloned()
            .ok_or(VfsError::NotFound)
    }

    fn list_files(&self) -> vfs::VfsResult<Vec<VfsEntry>> {
        let files = self.children.lock().values().cloned().collect();
        Ok(files)
    }

    fn remove(&self, name: &str) -> vfs::VfsResult<()> {
        let mut children = self.children.lock();
        children.remove(name).ok_or(VfsError::NotFound)?;
        Ok(())
    }
}
