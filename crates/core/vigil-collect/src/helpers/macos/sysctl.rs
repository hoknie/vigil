use std::ffi::CString;
use std::io;
use std::ptr;

const ATTEMPTS: usize = 4;

pub fn sysctl_named(name: &str) -> io::Result<Vec<u8>> {
    let name = CString::new(name).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;

    for _ in 0..ATTEMPTS {
        let mut size: libc::size_t = 0;
        let asked = unsafe {
            libc::sysctlbyname(
                name.as_ptr(),
                ptr::null_mut(),
                &mut size,
                ptr::null_mut(),
                0,
            )
        };
        if asked != 0 {
            return Err(io::Error::last_os_error());
        }

        let mut buffer = vec![0u8; size];
        let read = unsafe {
            libc::sysctlbyname(
                name.as_ptr(),
                buffer.as_mut_ptr().cast(),
                &mut size,
                ptr::null_mut(),
                0,
            )
        };
        match read {
            0 => {
                buffer.truncate(size);
                return Ok(buffer);
            }
            _ if io::Error::last_os_error().raw_os_error() == Some(libc::ENOMEM) => continue,
            _ => return Err(io::Error::last_os_error()),
        }
    }

    Err(io::Error::from_raw_os_error(libc::ENOMEM))
}

pub fn sysctl_numbered(name: &[libc::c_int]) -> io::Result<Vec<u8>> {
    let mut name = name.to_vec();
    let length = libc::c_uint::try_from(name.len())
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;

    for _ in 0..ATTEMPTS {
        let mut size: libc::size_t = 0;
        let asked = unsafe {
            libc::sysctl(
                name.as_mut_ptr(),
                length,
                ptr::null_mut(),
                &mut size,
                ptr::null_mut(),
                0,
            )
        };
        if asked != 0 {
            return Err(io::Error::last_os_error());
        }

        size += size / 8;
        let mut buffer = vec![0u8; size];
        let read = unsafe {
            libc::sysctl(
                name.as_mut_ptr(),
                length,
                buffer.as_mut_ptr().cast(),
                &mut size,
                ptr::null_mut(),
                0,
            )
        };
        match read {
            0 => {
                buffer.truncate(size);
                return Ok(buffer);
            }
            _ if io::Error::last_os_error().raw_os_error() == Some(libc::ENOMEM) => continue,
            _ => return Err(io::Error::last_os_error()),
        }
    }

    Err(io::Error::from_raw_os_error(libc::ENOMEM))
}

pub fn sysctl_into(name: &[libc::c_int], buffer: &mut [u8]) -> io::Result<usize> {
    let mut name = name.to_vec();
    let length = libc::c_uint::try_from(name.len())
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
    let mut size: libc::size_t = buffer.len();

    let read = unsafe {
        libc::sysctl(
            name.as_mut_ptr(),
            length,
            buffer.as_mut_ptr().cast(),
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    match read {
        0 => Ok(size),
        _ => Err(io::Error::last_os_error()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_asked_for_by_its_name_is_read_whole() {
        let kind = sysctl_named("kern.ostype").expect("every Mac names its kernel");

        assert_eq!(kind, b"Darwin\0");
    }

    #[test]
    fn a_name_the_kernel_does_not_know_is_an_error_and_not_an_empty_value() {
        let error = sysctl_named("kern.no_such_value_anywhere").expect_err("unknown");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(sysctl_named("kern.os\0type").is_err());
    }

    #[test]
    fn a_value_asked_for_by_its_numbers_is_read_whole() {
        let kind = sysctl_numbered(&[libc::CTL_KERN, libc::KERN_OSTYPE]).expect("read");

        assert_eq!(kind, b"Darwin\0");
    }

    #[test]
    fn a_value_read_into_a_buffer_says_how_much_of_it_was_written() {
        let mut buffer = [0u8; 64];

        let written = sysctl_into(&[libc::CTL_KERN, libc::KERN_OSTYPE], &mut buffer).expect("read");

        assert_eq!(&buffer[..written], b"Darwin\0");
    }
}
