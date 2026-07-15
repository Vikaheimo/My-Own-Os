use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};

pub trait VirtualFilesystem {
    fn root(&self) -> Arc<dyn VfsDirectory>;

    fn resolve_path(&self, path: &VfsPath) -> VfsResult<VfsEntry>;
}

pub struct VfsPath {
    absolute: bool,
    components: Vec<Box<str>>,
}

impl VfsPath {
    pub fn parse(path: &str) -> Self {
        let absolute = path.starts_with('/');
        let components = path
            .split('/')
            .filter(|element| !element.is_empty())
            .map(Box::from)
            .collect();

        Self {
            absolute,
            components,
        }
    }

    pub fn is_absolute(&self) -> bool {
        self.absolute
    }

    pub fn components(&self) -> &[Box<str>] {
        &self.components
    }
}

#[derive(Debug, Clone)]
pub struct VirtualFileMetadata {
    pub is_dir: bool,
    pub name: String,
}

pub trait FsError: core::fmt::Debug + core::fmt::Display {}
impl<T> FsError for T where T: core::fmt::Debug + core::fmt::Display {}

#[derive(Debug)]
pub enum VfsError {
    NotFound,
    NotADirectory,
    NotAFile,
    AlreadyExists,
    PermissionDenied,
    IoError,
    OffsetOutsideData,
    FsSpecific(Box<dyn FsError>),
}

impl core::fmt::Display for VfsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            VfsError::NotFound => "Not found",
            VfsError::NotADirectory => "Not a directory",
            VfsError::NotAFile => "Not a file",
            VfsError::AlreadyExists => "Already exists",
            VfsError::PermissionDenied => "Permission denied",
            VfsError::IoError => "I/O error",
            VfsError::OffsetOutsideData => "Offset outside data range",
            VfsError::FsSpecific(inner) => {
                return write!(f, "Filesystem-specific error: {}", inner);
            }
        };

        write!(f, "{msg}")
    }
}

impl core::error::Error for VfsError {}

pub type VfsResult<T> = Result<T, VfsError>;

#[derive(Clone)]
pub enum VfsEntry {
    File(Arc<dyn VfsFile>),
    Directory(Arc<dyn VfsDirectory>),
}

impl VfsEntry {
    pub fn metadata(&self) -> VirtualFileMetadata {
        match self {
            VfsEntry::File(file) => file.metadata(),
            VfsEntry::Directory(directory) => directory.metadata(),
        }
    }

    pub fn into_file(self) -> Result<Arc<dyn VfsFile>, VfsError> {
        self.try_into()
    }

    pub fn into_directory(self) -> Result<Arc<dyn VfsDirectory>, VfsError> {
        self.try_into()
    }
}

impl TryInto<Arc<dyn VfsFile>> for VfsEntry {
    type Error = VfsError;

    fn try_into(self) -> Result<Arc<dyn VfsFile>, Self::Error> {
        match self {
            VfsEntry::File(file) => Ok(file),
            VfsEntry::Directory(_) => Err(VfsError::NotAFile),
        }
    }
}

impl TryInto<Arc<dyn VfsDirectory>> for VfsEntry {
    type Error = VfsError;

    fn try_into(self) -> Result<Arc<dyn VfsDirectory>, Self::Error> {
        match self {
            VfsEntry::File(_) => Err(VfsError::NotADirectory),
            VfsEntry::Directory(directory) => Ok(directory),
        }
    }
}

pub trait VfsNode {
    fn metadata(&self) -> VirtualFileMetadata;
}

pub trait VfsFile: VfsNode {
    fn read(&self, offset: usize, buffer: &mut [u8]) -> VfsResult<usize>;

    fn write(&self, offset: usize, buffer: &[u8]) -> VfsResult<usize>;
}

pub trait VfsDirectory: VfsNode {
    fn create(&self, metadata: VirtualFileMetadata) -> VfsResult<VfsEntry>;

    fn remove(&self, name: &str) -> VfsResult<()>;

    fn find(&self, name: &str) -> VfsResult<VfsEntry>;

    fn list_files(&self) -> VfsResult<Vec<VfsEntry>>;
}
