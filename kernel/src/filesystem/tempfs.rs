use alloc::{string::String, sync::Arc, vec::Vec};
use spin::Mutex;

use crate::filesystem::vfs;

pub type TempFilesystemError = &'static str;

#[derive(Debug)]
pub struct TempFilesystem {
    root: Arc<TempFilesystemNode>,
}

impl Default for TempFilesystem {
    fn default() -> Self {
        let default_metadata = vfs::VirtualFileMetadata {
            is_dir: true,
            name: String::from("/"),
        };
        let root_node = TempFilesystemNode::new(default_metadata);
        assert!(root_node.is_dir(), "Root node must be a directory!");

        Self {
            root: Arc::new(root_node),
        }
    }
}

impl vfs::VirtualFilesystem for TempFilesystem {
    fn root(&self) -> Arc<dyn vfs::VfsNode<FilesystemError = TempFilesystemError>> {
        self.root.clone()
    }

    type FilesystemError = TempFilesystemError;
}

#[derive(Debug)]
enum TempFsNodeKind {
    Directory {
        children: Mutex<Vec<Arc<TempFilesystemNode>>>,
    },
    File {
        data: Mutex<Vec<u8>>,
    },
}

impl TempFsNodeKind {
    pub fn new(is_directory: bool) -> Self {
        if is_directory {
            Self::new_directory()
        } else {
            Self::new_file()
        }
    }
    pub fn new_file() -> Self {
        Self::File {
            data: Mutex::new(Vec::new()),
        }
    }

    pub fn new_directory() -> Self {
        Self::Directory {
            children: Mutex::new(Vec::new()),
        }
    }
}

#[derive(Debug)]
struct TempFilesystemNode {
    kind: TempFsNodeKind,
    name: String,
}

impl TempFilesystemNode {
    pub fn new(metadata: vfs::VirtualFileMetadata) -> Self {
        Self {
            kind: TempFsNodeKind::new(metadata.is_dir),
            name: metadata.name,
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.kind, TempFsNodeKind::Directory { children: _ })
    }
}

impl vfs::VfsNode for TempFilesystemNode {
    type FilesystemError = TempFilesystemError;

    #[allow(clippy::indexing_slicing)]
    fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<usize, Self::FilesystemError> {
        match &self.kind {
            TempFsNodeKind::Directory { children: _ } => Err("Cannot read data from a directory!"),
            TempFsNodeKind::File { data: lock } => {
                let data = lock.lock();

                if offset >= data.len() {
                    return Err("Offset is outside of the file!");
                }

                let end = usize::min(offset + buffer.len(), data.len());
                let size = end - offset;

                buffer[..size].copy_from_slice(&data[offset..end]);

                Ok(size)
            }
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn write(&self, offset: usize, buffer: &[u8]) -> Result<usize, Self::FilesystemError> {
        match &self.kind {
            TempFsNodeKind::Directory { children: _ } => Err("Cannot write data to a directory!"),

            TempFsNodeKind::File { data: lock } => {
                let mut data = lock.lock();

                if offset >= buffer.len() {
                    return Err("Offset is outside of the file!");
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
    }

    fn find(
        &self,
        name: &str,
    ) -> Result<
        Option<Arc<dyn vfs::VfsNode<FilesystemError = Self::FilesystemError>>>,
        Self::FilesystemError,
    > {
        match &self.kind {
            TempFsNodeKind::Directory { children } => {
                let child = children
                    .lock()
                    .iter()
                    .find(|child| child.name == name)
                    .map(|c| {
                        c.clone() as Arc<dyn vfs::VfsNode<FilesystemError = Self::FilesystemError>>
                    });
                Ok(child)
            }
            TempFsNodeKind::File { data: _ } => Err("Cannot find files from file!"),
        }
    }

    fn create(
        &self,
        metadata: vfs::VirtualFileMetadata,
    ) -> Result<Arc<dyn vfs::VfsNode<FilesystemError = Self::FilesystemError>>, Self::FilesystemError>
    {
        match &self.kind {
            TempFsNodeKind::Directory { children: lock } => {
                let mut children = lock.lock();
                let already_exists = children
                    .iter()
                    .map(|node| &node.name)
                    .any(|name| *name == metadata.name);
                if already_exists {
                    return Err("File with that name already exists!");
                }

                let new_child = Arc::new(TempFilesystemNode::new(metadata));
                children.push(new_child.clone());

                Ok(new_child)
            }
            TempFsNodeKind::File { data: _ } => Err("Cannot create file inside file!"),
        }
    }

    fn metadata(&self) -> vfs::VirtualFileMetadata {
        vfs::VirtualFileMetadata {
            is_dir: self.is_dir(),
            name: self.name.clone(),
        }
    }

    fn list_files(
        &self,
    ) -> Result<Vec<vfs::VfsPointer<Self::FilesystemError>>, Self::FilesystemError> {
        match self.kind {
            TempFsNodeKind::Directory { ref children } => {
                let files: Vec<Arc<dyn vfs::VfsNode<FilesystemError = Self::FilesystemError>>> =
                    children
                        .lock()
                        .iter()
                        .map(|node| {
                            node.clone()
                                as Arc<dyn vfs::VfsNode<FilesystemError = Self::FilesystemError>>
                        })
                        .collect();
                Ok(files)
            }
            TempFsNodeKind::File { data: _ } => Err("Cannot iterate over a file!"),
        }
    }
}
