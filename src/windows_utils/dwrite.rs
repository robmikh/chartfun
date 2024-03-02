use windows::{core::Result, Win32::Graphics::DirectWrite::{DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE}};

pub fn create_dwrite_factory(factory_type: DWRITE_FACTORY_TYPE) -> Result<IDWriteFactory> {
    unsafe {
        DWriteCreateFactory(factory_type)
    }
}