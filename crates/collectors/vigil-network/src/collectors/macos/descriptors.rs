use std::io;

use crate::parsers::SOCKET_INFORMATION_BYTES;

const LIST_DESCRIPTORS: libc::c_int = 1;

const SOCKET_INFORMATION: libc::c_int = 3;

const DESCRIPTOR_BYTES: usize = 8;

const SPARE_DESCRIPTORS: usize = 32;

pub(super) fn open_descriptors(pid: u32) -> io::Result<Vec<u8>> {
    let pid = libc::c_int::try_from(pid).map_err(|_| io::Error::from_raw_os_error(libc::ESRCH))?;

    let asked = unsafe { libc::proc_pidinfo(pid, LIST_DESCRIPTORS, 0, std::ptr::null_mut(), 0) };
    let Ok(asked) = usize::try_from(asked) else {
        return Err(io::Error::last_os_error());
    };
    if asked == 0 {
        return match io::Error::last_os_error().raw_os_error() {
            Some(0) | None => Ok(Vec::new()),
            Some(_) => Err(io::Error::last_os_error()),
        };
    }

    let mut buffer = vec![0u8; asked + SPARE_DESCRIPTORS * DESCRIPTOR_BYTES];
    let written = unsafe {
        libc::proc_pidinfo(
            pid,
            LIST_DESCRIPTORS,
            0,
            buffer.as_mut_ptr().cast(),
            libc::c_int::try_from(buffer.len()).unwrap_or(libc::c_int::MAX),
        )
    };
    let Ok(written) = usize::try_from(written) else {
        return Err(io::Error::last_os_error());
    };
    if written == 0 {
        return Err(io::Error::last_os_error());
    }

    buffer.truncate(written - written % DESCRIPTOR_BYTES);
    Ok(buffer)
}

pub(super) fn socket_information(pid: u32, descriptor: i32) -> io::Result<Vec<u8>> {
    let pid = libc::c_int::try_from(pid).map_err(|_| io::Error::from_raw_os_error(libc::ESRCH))?;
    let mut buffer = vec![0u8; SOCKET_INFORMATION_BYTES];

    let written = unsafe {
        libc::proc_pidfdinfo(
            pid,
            descriptor,
            SOCKET_INFORMATION,
            buffer.as_mut_ptr().cast(),
            libc::c_int::try_from(buffer.len()).unwrap_or(libc::c_int::MAX),
        )
    };
    match usize::try_from(written) {
        Ok(written) if written == SOCKET_INFORMATION_BYTES => Ok(buffer),
        Ok(0) | Err(_) => Err(io::Error::last_os_error()),
        Ok(_) => Err(io::Error::from(io::ErrorKind::InvalidData)),
    }
}
