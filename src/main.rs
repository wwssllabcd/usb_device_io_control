mod ericcore;
use ericcore::device::scsi::*;
use ericcore::device::uficmd::*;

fn main() {
    let devices= gen_device_string();
    let handle_colls = get_handle_colls(&devices);

    for x in handle_colls{
        let (d, h) = x;

        println!("{} is usb device, handle = {:x}", d, h as u32);

        let mut data_buf: [u8; 36] = [0; 36];
        let _status = scsi_pass_through_direct(h, &inquiry(), &mut data_buf);
        
        println!("====");

        let s = String::from_utf8(data_buf.to_vec()).expect("Found invalid UTF-8");
        println!("handle={:x}, s={}", h as u32, s);
    }
}