use windows::{
    core::Result,
    Win32::{
        Foundation::LUID,
        Graphics::Dxgi::{IDXGIAdapter1, IDXGIFactory1, DXGI_ADAPTER_DESC1},
    },
};

use crate::windows_utils::dxgi::DxgiAdapterIter;

pub struct Adapter {
    pub name: String,
    pub luid: LUID,
}

impl Adapter {
    pub fn from_dxgi_adapter(adapter: &IDXGIAdapter1) -> Result<Self> {
        unsafe {
            let mut desc = DXGI_ADAPTER_DESC1::default();
            adapter.GetDesc1(&mut desc)?;
            let luid = desc.AdapterLuid;

            let name_utf16 = &desc.Description[..desc
                .Description
                .iter()
                .position(|x| *x == 0)
                .unwrap_or(desc.Description.len())];
            let name = String::from_utf16(name_utf16)?;

            Ok(Self { name, luid })
        }
    }

    pub fn from_dxgi_factory(factory: &IDXGIFactory1) -> Result<Vec<Self>> {
        let mut adapters = Vec::new();
        for dxgi_adapter in factory.iter_adapters() {
            let adapter = Self::from_dxgi_adapter(&dxgi_adapter)?;
            adapters.push(adapter);
        }
        Ok(adapters)
    }
}
