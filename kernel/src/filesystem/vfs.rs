use alloc::{string::String, sync::Arc, vec::Vec};

pub trait VirtualFilesystem {
    type FilesystemError;
    fn root(&self) -> Arc<dyn VfsNode<FilesystemError = Self::FilesystemError>>;
}

#[derive(Debug, Clone)]
pub struct VirtualFileMetadata {
    pub is_dir: bool,
    pub name: String,
}

pub type VfsPointer<E> = Arc<dyn VfsNode<FilesystemError = E>>;

pub trait VfsNode {
    type FilesystemError;

    fn metadata(&self) -> VirtualFileMetadata;

    fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<usize, Self::FilesystemError>;

    fn write(&self, offset: usize, buffer: &[u8]) -> Result<usize, Self::FilesystemError>;

    fn create(
        &self,
        metadata: VirtualFileMetadata,
    ) -> Result<VfsPointer<Self::FilesystemError>, Self::FilesystemError>;

    fn find(
        &self,
        name: &str,
    ) -> Result<Option<VfsPointer<Self::FilesystemError>>, Self::FilesystemError>;

    fn list_files(&self) -> Result<Vec<VfsPointer<Self::FilesystemError>>, Self::FilesystemError>;
}
