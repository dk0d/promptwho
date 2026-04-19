pub trait SupportsVectors {
    fn vector_enabled(&self) -> bool;
}

pub trait SupportsSyncMetadata {
    fn sync_enabled(&self) -> bool;
}
