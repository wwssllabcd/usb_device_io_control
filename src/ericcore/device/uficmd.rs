use super::scsi::{ScsiCmd};

use winapi::shared::ntddscsi::{SCSI_IOCTL_DATA_IN};

pub fn inquiry() -> ScsiCmd {
    let mut cmd = ScsiCmd{
        cdb: [0; 16]
        ,data_len: 0
        ,direction: SCSI_IOCTL_DATA_IN
    }; 
    cmd.cdb[0] = 0x12;
    cmd.cdb[4] = 0x24;
    cmd.data_len = 0x24;
    return cmd;
}
