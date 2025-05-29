use windows::{
    core::Result,
    Win32::Graphics::Dxgi::{IDXGIAdapter1, IDXGIFactory1},
};

pub struct DxgiAdapterIterator<'a> {
    factory: &'a IDXGIFactory1,
    current_adapter_index: u32,
}

pub trait DxgiAdapterIter {
    fn iter_adapters<'a>(&'a self) -> DxgiAdapterIterator<'a>;
}

impl<'a> Iterator for DxgiAdapterIterator<'a> {
    type Item = IDXGIAdapter1;

    fn next(&mut self) -> Option<Self::Item> {
        let result = unsafe { self.factory.EnumAdapters1(self.current_adapter_index).ok() };
        if result.is_some() {
            self.current_adapter_index += 1;
        }

        result
    }
}

impl DxgiAdapterIter for IDXGIFactory1 {
    fn iter_adapters<'a>(&'a self) -> DxgiAdapterIterator<'a> {
        DxgiAdapterIterator {
            factory: self,
            current_adapter_index: 0,
        }
    }
}
