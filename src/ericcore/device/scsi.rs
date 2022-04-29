
extern crate winapi;

use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
use winapi::um::winnt::{
    GENERIC_READ, GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_ATTRIBUTE_NORMAL
    , HANDLE
};

use winapi::um::winioctl::{STORAGE_PROPERTY_QUERY, IOCTL_STORAGE_QUERY_PROPERTY, PropertyStandardQuery, StorageDeviceProperty};
use winapi::um::ioapiset::DeviceIoControl;
use winapi::ctypes::c_void;
use winapi::shared::ntddscsi::{IOCTL_SCSI_PASS_THROUGH_DIRECT, SCSI_PASS_THROUGH_DIRECT};

const CDB_LEN: u8 = 12;
const TIMEOUT: u32 = 30;
const BusTypeUsb: u32 = 7;

pub struct SPTD{
    sptd: SCSI_PASS_THROUGH_DIRECT,
    //filler: u32,
    sense_buf: [u8; 32]
}

pub struct ScsiCmd{
    pub cdb: [u8; 16],
    pub direction: u8,
    pub data_len: u32
}

pub fn open_device(device_path: &String) -> HANDLE {
    let wide: Vec<u16> = OsStr::new(device_path).encode_wide().chain(once(0)).collect();
    let handle = unsafe {
        CreateFileW(wide.as_ptr(), 
            GENERIC_READ | GENERIC_WRITE, 
            FILE_SHARE_READ | FILE_SHARE_WRITE, 
            null_mut(),
            OPEN_EXISTING, 
            FILE_ATTRIBUTE_NORMAL, 
            null_mut()
        )
    };
    return handle;
}

pub fn get_bus_type(handle: HANDLE) -> u32 {
    let mut dev_desc = [(0u32); 10];
    let mut dw_out: u32 = 0;
    let _: i32 = unsafe {
        
        let mut query =  STORAGE_PROPERTY_QUERY {
            PropertyId: StorageDeviceProperty,
            QueryType: PropertyStandardQuery,
            AdditionalParameters: [0]
        };
        
        let state_ptr: *mut c_void = &mut query as *mut _ as *mut c_void;
        let output_ptr: *mut c_void = &mut dev_desc as *mut _ as *mut c_void;

        DeviceIoControl(
                    handle,
                    IOCTL_STORAGE_QUERY_PROPERTY, 
                    state_ptr, 
                    std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                    output_ptr, 
                    40 as u32,
                    &mut dw_out,
                    null_mut()
    )};
    return dev_desc[7];
}

pub fn scsi_pass_through_direct_base(handle: HANDLE, cdb: [u8; 16], xfer_len: u32, direction: u8, buffer: &mut [u8], time_out: u32) -> u8 {
    let output_ptr: *mut c_void = buffer.as_ptr() as *mut _;
    let mut scsi =  SPTD {
        sptd: SCSI_PASS_THROUGH_DIRECT{
            Lun: 0,
            PathId: 0,
            TargetId: 0,
            ScsiStatus: 0,
            SenseInfoLength: 0,
            
            Cdb: cdb,
            CdbLength: CDB_LEN,
            Length: std::mem::size_of::<SCSI_PASS_THROUGH_DIRECT>() as u16,
            SenseInfoOffset: memoffset::offset_of!(SPTD, sense_buf) as u32,
            TimeOutValue: time_out,
            DataIn: direction,
            DataTransferLength: xfer_len,
            DataBuffer: output_ptr
        },
        //filler: 0,
        sense_buf: [0; 32]
    };

    let mut dw_out: u32 = 0;
    let _: i32 = unsafe {
        let scsi_ptr: *mut c_void = &mut scsi as *mut _ as *mut c_void;
        DeviceIoControl(
                    handle,
                    IOCTL_SCSI_PASS_THROUGH_DIRECT, 
                    scsi_ptr, 
                    std::mem::size_of::<SPTD>() as u32,
                    scsi_ptr, 
                    std::mem::size_of::<SPTD>() as u32,
                    &mut dw_out,
                    null_mut()
    )};
    return scsi.sptd.ScsiStatus;
}

pub fn scsi_pass_through_direct(handle: HANDLE, cmd: &ScsiCmd, buffer: &mut [u8]) -> u8 {
    return scsi_pass_through_direct_base(handle, cmd.cdb, cmd.data_len, cmd.direction, buffer, TIMEOUT);
}

pub fn gen_device_string() -> Vec<String>{
    let mut devices: Vec<String> = Vec::new();
    for i in 0..32 {
        devices.push(format!("{}{}", "\\\\.\\PhysicalDrive", i));
    }
    return devices;
}

pub fn get_handle_colls(devices: &Vec<String>) -> Vec<(&String, HANDLE)> {
    let mut handle_colls = Vec::new();
    for d in devices {
        let h: HANDLE = open_device(d);
        let bus_type = get_bus_type(h);
        if bus_type == BusTypeUsb {
            handle_colls.push((d, h));
        }
    }
    return handle_colls;
}



