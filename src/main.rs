use std::{
    arch::asm,
    ffi::OsString,
    os::{fd::AsRawFd, unix::ffi::OsStringExt},
};

const IOCTL: u64 = 16;
const DRM_BASE_GROUP: u32 = 100;
const GET_VERSION_NR: u32 = 0x00;

const NUM_BITS: u32 = 8;
const GROUP_BITS: u32 = 8;
const SIZE_BITS: u32 = 14;
const DIR_BITS: u32 = 2;

const GROUP_SHIFT: u32 = NUM_BITS;
const SIZE_SHIFT: u32 = GROUP_SHIFT + GROUP_BITS;
const DIR_SHIFT: u32 = SIZE_SHIFT + SIZE_BITS;

const MASK_8_BIT: u32 = (1 << 8) - 1;
const SIZE_MASK: u32 = (1 << SIZE_BITS) - 1;
const DIR_MASK: u32 = (1 << DIR_BITS) - 1;

const READWRITE: u32 = 1 | 2;

#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct DrmVersion {
    pub version_major: core::ffi::c_int,
    pub version_minor: core::ffi::c_int,
    pub version_patchlevel: core::ffi::c_int,
    pub name_len: usize,
    pub name: *mut core::ffi::c_char,
    pub date_len: usize,
    pub date: *mut core::ffi::c_char,
    pub desc_len: usize,
    pub desc: *mut core::ffi::c_char,
}

impl Default for DrmVersion {
    fn default() -> Self {
        let mut s = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}

fn main() {
    let card = std::fs::File::options()
        .write(true)
        .open("/dev/dri/card1")
        .unwrap();

    let fd = card.as_raw_fd();

    let size: usize = core::mem::size_of::<DrmVersion>();
    println!("size: {:?}", size);
    let get_version_opcode = ((DRM_BASE_GROUP & MASK_8_BIT) << GROUP_SHIFT)
        | (GET_VERSION_NR & MASK_8_BIT)
        | ((READWRITE & DIR_MASK) << DIR_SHIFT)
        | ((size as u32 & SIZE_MASK) << SIZE_SHIFT); // group + num + dir + size

    unsafe {
        let mut drm_version_struct = DrmVersion {
            ..DrmVersion::default()
        };
        let r0: i64;
        asm!(
        "syscall",
        inlateout("rax") IOCTL => r0,
        in("rdi") fd,
        in("rsi") get_version_opcode,
        in("rdx") &drm_version_struct,
        );
        if r0 < 0 {
            println!("ahhhhhh");
        }
        println!("drm_version_struct: {:#?}", drm_version_struct);

        let r0: i64;
        let mut name_buf: Vec<i8> = vec![0; drm_version_struct.name_len];
        let mut date_buf: Vec<i8> = vec![0; drm_version_struct.date_len];
        let mut desc_buf: Vec<i8> = vec![0; drm_version_struct.desc_len];

        drm_version_struct.name = name_buf.as_mut_ptr();
        drm_version_struct.date = date_buf.as_mut_ptr();
        drm_version_struct.desc = desc_buf.as_mut_ptr();

        asm!(
        "syscall",
        inlateout("rax") IOCTL => r0,
        in("rdi") fd,
        in("rsi") get_version_opcode,
        in("rdx") &mut drm_version_struct,
        );
        if r0 < 0 {
            println!("failed with error: {}", -r0);
        }
        println!("name_buf raw: {:?}", name_buf);
        println!("date_buf raw: {:?}", date_buf);
        let mut from = std::mem::ManuallyDrop::new(name_buf);
        let name_vec = OsString::from_vec(Vec::from_raw_parts(
            from.as_mut_ptr() as *mut u8,
            drm_version_struct.name_len,
            drm_version_struct.name_len,
        ));

        println!("desc_buf raw: {:?}", desc_buf);
        let mut from = std::mem::ManuallyDrop::new(desc_buf);
        let desc_vec = OsString::from_vec(Vec::from_raw_parts(
            from.as_mut_ptr() as *mut u8,
            drm_version_struct.desc_len,
            drm_version_struct.desc_len,
        ));

        let mut from = std::mem::ManuallyDrop::new(date_buf);
        let date_vec = OsString::from_vec(Vec::from_raw_parts(
            from.as_mut_ptr() as *mut u8,
            drm_version_struct.date_len,
            drm_version_struct.date_len,
        ));
        println!("drm_version_struct: {:#?}", drm_version_struct);
        println!("drm_version_struct.name: {:#?}", name_vec);
        println!("drm_version_struct.date: {:#?}", date_vec);
        println!("drm_version_struct.desc: {:#?}", desc_vec);
    }
}
