use windows::{
    core::{Result, HSTRING, PWSTR},
    Win32::{
        Foundation::BOOLEAN,
        System::Performance::{
            PdhAddCounterW, PdhAddEnglishCounterW, PdhCloseQuery, PdhExpandWildCardPathW,
            PdhGetCounterInfoW, PdhOpenQueryW, PDH_COUNTER_INFO_W, PDH_MORE_DATA,
        },
    },
};

// PDH Error Values:
// PDH_ACCESS_DENIED                           0xC0000BDB
// PDH_ASYNC_QUERY_TIMEOUT                     0x800007DB
// PDH_BINARY_LOG_CORRUPT                      0xC0000BF7
// PDH_CALC_NEGATIVE_DENOMINATOR               0x800007D6
// PDH_CALC_NEGATIVE_TIMEBASE                  0x800007D7
// PDH_CALC_NEGATIVE_VALUE                     0x800007D8
// PDH_CANNOT_CONNECT_MACHINE                  0xC0000BC3
// PDH_CANNOT_CONNECT_WMI_SERVER               0xC0000BE8
// PDH_CANNOT_READ_NAME_STRINGS                0xC0000BC8
// PDH_CANNOT_SET_DEFAULT_REALTIME_DATASOURCE  0x800007DC
// PDH_COUNTER_ALREADY_IN_QUERY                0xC0000BF6
// PDH_CSTATUS_BAD_COUNTERNAME                 0xC0000BC0
// PDH_CSTATUS_INVALID_DATA                    0xC0000BBA
// PDH_CSTATUS_ITEM_NOT_VALIDATED              0x800007D3
// PDH_CSTATUS_NEW_DATA                        0x00000001
// PDH_CSTATUS_NO_COUNTER                      0xC0000BB9
// PDH_CSTATUS_NO_COUNTERNAME                  0xC0000BBF
// PDH_CSTATUS_NO_INSTANCE                     0x800007D1
// PDH_CSTATUS_NO_MACHINE                      0x800007D0
// PDH_CSTATUS_NO_OBJECT                       0xC0000BB8
// PDH_CSTATUS_VALID_DATA                      0x00000000
// PDH_DATA_SOURCE_IS_LOG_FILE                 0xC0000BCE
// PDH_DATA_SOURCE_IS_REAL_TIME                0xC0000BCF
// PDH_DIALOG_CANCELLED                        0x800007D9
// PDH_END_OF_LOG_FILE                         0x800007DA
// PDH_ENTRY_NOT_IN_LOG_FILE                   0xC0000BCD
// PDH_FILE_ALREADY_EXISTS                     0xC0000BD2
// PDH_FILE_NOT_FOUND                          0xC0000BD1
// PDH_FUNCTION_NOT_FOUND                      0xC0000BBE
// PDH_INCORRECT_APPEND_TIME                   0xC0000BFB
// PDH_INSUFFICIENT_BUFFER                     0xC0000BC2
// PDH_INVALID_ARGUMENT                        0xC0000BBD
// PDH_INVALID_BUFFER                          0xC0000BC1
// PDH_INVALID_DATA                            0xC0000BC6
// PDH_INVALID_DATASOURCE                      0xC0000BDD
// PDH_INVALID_HANDLE                          0xC0000BBC
// PDH_INVALID_INSTANCE                        0xC0000BC5
// PDH_INVALID_PATH                            0xC0000BC4
// PDH_INVALID_SQLDB                           0xC0000BDE
// PDH_INVALID_SQL_LOG_FORMAT                  0xC0000BF5
// PDH_LOGSVC_NOT_OPENED                       0xC0000BD9
// PDH_LOGSVC_QUERY_NOT_FOUND                  0xC0000BD8
// PDH_LOG_FILE_CREATE_ERROR                   0xC0000BC9
// PDH_LOG_FILE_OPEN_ERROR                     0xC0000BCA
// PDH_LOG_FILE_TOO_SMALL                      0xC0000BDC
// PDH_LOG_SAMPLE_TOO_SMALL                    0xC0000BF8
// PDH_LOG_TYPE_NOT_FOUND                      0xC0000BCB
// PDH_LOG_TYPE_RETIRED_BIN                    0x00000003
// PDH_LOG_TYPE_TRACE_GENERIC                  0x00000005
// PDH_LOG_TYPE_TRACE_KERNEL                   0x00000004
// PDH_MAX_COUNTER_NAME                        0x00000400
// PDH_MAX_COUNTER_PATH                        0x00000800
// PDH_MAX_DATASOURCE_PATH                     0x00000400
// PDH_MAX_INSTANCE_NAME                       0x00000400
// PDH_MAX_SCALE                               0x00000007
// PDH_MEMORY_ALLOCATION_FAILURE               0xC0000BBB
// PDH_MIN_SCALE                               0xFFFFFFF9
// PDH_MORE_DATA                               0x800007D2
// PDH_NOEXPANDCOUNTERS                        0x00000001
// PDH_NOEXPANDINSTANCES                       0x00000002
// PDH_NOT_IMPLEMENTED                         0xC0000BD3
// PDH_NO_COUNTERS                             0xC0000BDF
// PDH_NO_DATA                                 0x800007D5
// PDH_NO_DIALOG_DATA                          0xC0000BC7
// PDH_NO_MORE_DATA                            0xC0000BCC
// PDH_OS_EARLIER_VERSION                      0xC0000BFA
// PDH_OS_LATER_VERSION                        0xC0000BF9
// PDH_PLA_COLLECTION_ALREADY_RUNNING          0xC0000BE9
// PDH_PLA_COLLECTION_NOT_FOUND                0xC0000BEB
// PDH_PLA_ERROR_ALREADY_EXISTS                0xC0000BEE
// PDH_PLA_ERROR_FILEPATH                      0xC0000BF0
// PDH_PLA_ERROR_NAME_TOO_LONG                 0xC0000BF4
// PDH_PLA_ERROR_NOSTART                       0xC0000BED
// PDH_PLA_ERROR_SCHEDULE_ELAPSED              0xC0000BEC
// PDH_PLA_ERROR_SCHEDULE_OVERLAP              0xC0000BEA
// PDH_PLA_ERROR_TYPE_MISMATCH                 0xC0000BEF
// PDH_PLA_SERVICE_ERROR                       0xC0000BF1
// PDH_PLA_VALIDATION_ERROR                    0xC0000BF2
// PDH_PLA_VALIDATION_WARNING                  0x80000BF3
// PDH_QUERY_PERF_DATA_TIMEOUT                 0xC0000BFE
// PDH_REFRESHCOUNTERS                         0x00000004
// PDH_RETRY                                   0x800007D4
// PDH_SQL_ALLOCCON_FAILED                     0xC0000BE1
// PDH_SQL_ALLOC_FAILED                        0xC0000BE0
// PDH_SQL_ALTER_DETAIL_FAILED                 0xC0000BFD
// PDH_SQL_BIND_FAILED                         0xC0000BE7
// PDH_SQL_CONNECT_FAILED                      0xC0000BE6
// PDH_SQL_EXEC_DIRECT_FAILED                  0xC0000BE2
// PDH_SQL_FETCH_FAILED                        0xC0000BE3
// PDH_SQL_MORE_RESULTS_FAILED                 0xC0000BE5
// PDH_SQL_ROWCOUNT_FAILED                     0xC0000BE4
// PDH_STRING_NOT_FOUND                        0xC0000BD4
// PDH_UNABLE_MAP_NAME_FILES                   0x80000BD5
// PDH_UNABLE_READ_LOG_HEADER                  0xC0000BD0
// PDH_UNKNOWN_LOGSVC_COMMAND                  0xC0000BD7
// PDH_UNKNOWN_LOG_FORMAT                      0xC0000BD6
// PDH_UNMATCHED_APPEND_COUNTER                0xC0000BFC
// PDH_WBEM_ERROR                              0xC0000BDA

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct PDH_FUNCTION(pub u32);

impl PDH_FUNCTION {
    #[inline]
    pub const fn is_ok(self) -> bool {
        self.0 == 0
    }
    #[inline]
    pub const fn is_err(self) -> bool {
        !self.is_ok()
    }
    #[inline]
    pub const fn to_hresult(self) -> ::windows::core::HRESULT {
        ::windows::core::HRESULT(self.0 as _)
    }
    #[inline]
    pub fn from_error(error: &::windows::core::Error) -> ::core::option::Option<Self> {
        Some(Self(error.code().0 as u32))
    }
    #[inline]
    pub fn ok(self) -> ::windows::core::Result<()> {
        self.to_hresult().ok()
    }
}
impl ::core::convert::From<PDH_FUNCTION> for ::windows::core::HRESULT {
    fn from(value: PDH_FUNCTION) -> Self {
        value.to_hresult()
    }
}
impl ::core::convert::From<PDH_FUNCTION> for ::windows::core::Error {
    fn from(value: PDH_FUNCTION) -> Self {
        ::windows::core::Error::new(value.to_hresult(), "")
    }
}

pub struct PerfQueryHandle(pub isize);

impl PerfQueryHandle {
    pub fn open_query() -> Result<Self> {
        let query_handle = unsafe {
            let mut query_handle = 0;
            PDH_FUNCTION(PdhOpenQueryW(None, 0, &mut query_handle)).ok()?;
            query_handle
        };
        Ok(Self(query_handle))
    }

    pub fn close_query(&mut self) -> Result<()> {
        if self.0 != 0 {
            unsafe {
                PDH_FUNCTION(PdhCloseQuery(self.0)).ok()?;
            }
            self.0 = 0;
        }
        Ok(())
    }
}

impl Drop for PerfQueryHandle {
    fn drop(&mut self) {
        self.close_query().unwrap();
    }
}

pub fn add_perf_counters(
    query_handle: &PerfQueryHandle,
    wildcard_path: &str,
) -> Result<Vec<isize>> {
    let counter_handles = unsafe {
        let mut counter_handle = 0;
        PDH_FUNCTION(PdhAddEnglishCounterW(
            query_handle.0,
            &HSTRING::from(wildcard_path),
            0,
            &mut counter_handle,
        ))
        .ok()?;

        let mut buffer_size = 0;
        let result = PdhGetCounterInfoW(counter_handle, BOOLEAN(0), &mut buffer_size, None);
        assert_eq!(result, PDH_MORE_DATA);
        let mut buffer = vec![0u8; buffer_size as usize];
        PDH_FUNCTION(PdhGetCounterInfoW(
            counter_handle,
            BOOLEAN(0),
            &mut buffer_size,
            Some(buffer.as_mut_ptr() as *mut _),
        ))
        .ok()?;
        let header: *const PDH_COUNTER_INFO_W = buffer.as_ptr() as *const _;
        let header = header.as_ref().unwrap();
        let full_path = header.szFullPath.to_hstring()?;

        let mut buffer_size = 0;
        let result = PdhExpandWildCardPathW(
            None,
            &full_path,
            PWSTR(std::ptr::null_mut()),
            &mut buffer_size,
            0,
        );
        assert_eq!(result, PDH_MORE_DATA);
        let mut buffer = vec![0u16; buffer_size as usize];
        PDH_FUNCTION(PdhExpandWildCardPathW(
            None,
            &full_path,
            PWSTR(buffer.as_mut_ptr()),
            &mut buffer_size,
            0,
        ))
        .ok()?;

        let mut paths = Vec::new();
        let mut start = 0;
        for (i, char) in buffer.iter().enumerate() {
            if *char == 0 && i != start {
                let path = HSTRING::from_wide(&buffer[start..i])?;
                paths.push(path);
                start = i + 1;
            }
        }

        let mut counter_handles = Vec::new();
        for path in &paths {
            let mut counter_handle = 0;
            PDH_FUNCTION(PdhAddCounterW(query_handle.0, path, 0, &mut counter_handle)).ok()?;
            counter_handles.push(counter_handle);
        }

        counter_handles
    };
    Ok(counter_handles)
}
