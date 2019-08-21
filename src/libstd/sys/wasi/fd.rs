#![allow(dead_code)]

use crate::io::{self, IoSlice, IoSliceMut, SeekFrom};
use crate::mem;
use crate::net::Shutdown;
use super::err2io;
use ::wasi::wasi_unstable as wasi;

#[derive(Debug)]
pub struct WasiFd {
    fd: wasi::Fd,
}

fn iovec<'a>(a: &'a mut [IoSliceMut<'_>]) -> &'a [wasi::IoVec] {
    assert_eq!(
        mem::size_of::<IoSliceMut<'_>>(),
        mem::size_of::<wasi::IoVec>()
    );
    assert_eq!(
        mem::align_of::<IoSliceMut<'_>>(),
        mem::align_of::<wasi::IoVec>()
    );
    /// SAFETY: `IoSliceMut` and `IoVec` have exactly the same memory layout
    unsafe { mem::transmute(a) }
}

fn ciovec<'a>(a: &'a [IoSlice<'_>]) -> &'a [wasi::CIoVec] {
    assert_eq!(
        mem::size_of::<IoSlice<'_>>(),
        mem::size_of::<wasi::CIoVec>()
    );
    assert_eq!(
        mem::align_of::<IoSlice<'_>>(),
        mem::align_of::<wasi::CIoVec>()
    );
    /// SAFETY: `IoSlice` and `CIoVec` have exactly the same memory layout
    unsafe { mem::transmute(a) }
}

impl WasiFd {
    pub unsafe fn from_raw(fd: wasi::Fd) -> WasiFd {
        WasiFd { fd }
    }

    pub fn into_raw(self) -> wasi::Fd {
        let ret = self.fd;
        mem::forget(self);
        ret
    }

    pub fn as_raw(&self) -> wasi::Fd {
        self.fd
    }

    pub fn datasync(&self) -> io::Result<()> {
        wasi::fd_datasync(self.fd).map_err(err2io)
    }

    pub fn pread(&self, bufs: &mut [IoSliceMut<'_>], offset: u64) -> io::Result<usize> {
        wasi::fd_pread(self.fd, iovec(bufs), offset).map_err(err2io)
    }

    pub fn pwrite(&self, bufs: &[IoSlice<'_>], offset: u64) -> io::Result<usize> {
        wasi::fd_pwrite(self.fd, ciovec(bufs), offset).map_err(err2io)
    }

    pub fn read(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        wasi::fd_read(self.fd, iovec(bufs)).map_err(err2io)
    }

    pub fn write(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        wasi::fd_write(self.fd, ciovec(bufs)).map_err(err2io)
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        let (whence, offset) = match pos {
            SeekFrom::Start(pos) => (wasi::WHENCE_SET, pos as i64),
            SeekFrom::End(pos) => (wasi::WHENCE_END, pos),
            SeekFrom::Current(pos) => (wasi::WHENCE_CUR, pos),
        };
        wasi::fd_seek(self.fd, offset, whence).map_err(err2io)
    }

    pub fn tell(&self) -> io::Result<u64> {
        wasi::fd_tell(self.fd).map_err(err2io)
    }

    // FIXME: __wasi_fd_fdstat_get

    pub fn set_flags(&self, flags: wasi::FdFlags) -> io::Result<()> {
        wasi::fd_fdstat_set_flags(self.fd, flags).map_err(err2io)
    }

    pub fn set_rights(&self, base: wasi::Rights, inheriting: wasi::Rights) -> io::Result<()> {
        wasi::fd_fdstat_set_rights(self.fd, base, inheriting).map_err(err2io)
    }

    pub fn sync(&self) -> io::Result<()> {
        wasi::fd_sync(self.fd).map_err(err2io)
    }

    pub fn advise(&self, offset: u64, len: u64, advice: wasi::Advice) -> io::Result<()> {
        wasi::fd_advise(self.fd, offset, len, advice).map_err(err2io)
    }

    pub fn allocate(&self, offset: u64, len: u64) -> io::Result<()> {
        wasi::fd_allocate(self.fd, offset, len).map_err(err2io)
    }

    pub fn create_directory(&self, path: &[u8]) -> io::Result<()> {
        wasi::path_create_directory(self.fd, path).map_err(err2io)
    }

    pub fn link(
        &self,
        old_flags: wasi::LookupFlags,
        old_path: &[u8],
        new_fd: &WasiFd,
        new_path: &[u8],
    ) -> io::Result<()> {
        wasi::path_link(self.fd, old_flags, old_path, new_fd.fd, new_path)
            .map_err(err2io)
    }

    pub fn open(
        &self,
        dirflags: wasi::LookupFlags,
        path: &[u8],
        oflags: wasi::OFlags,
        fs_rights_base: wasi::Rights,
        fs_rights_inheriting: wasi::Rights,
        fs_flags: wasi::FdFlags,
    ) -> io::Result<WasiFd> {
        wasi::path_open(
            self.fd,
            dirflags,
            path,
            oflags,
            fs_rights_base,
            fs_rights_inheriting,
            fs_flags,
        ).map(|fd| unsafe { WasiFd::from_raw(fd) }).map_err(err2io)
    }

    pub fn readdir(&self, buf: &mut [u8], cookie: wasi::DirCookie) -> io::Result<usize> {
        wasi::fd_readdir(self.fd, buf, cookie).map_err(err2io)
    }

    pub fn readlink(&self, path: &[u8], buf: &mut [u8]) -> io::Result<usize> {
        wasi::path_readlink(self.fd, path, buf).map_err(err2io)
    }

    pub fn rename(&self, old_path: &[u8], new_fd: &WasiFd, new_path: &[u8]) -> io::Result<()> {
        wasi::path_rename(self.fd, old_path, new_fd.fd, new_path)
            .map_err(err2io)
    }

    pub fn filestat_get(&self) -> io::Result<wasi::FileStat> {
        wasi::fd_filestat_get(self.fd).map_err(err2io)
    }

    pub fn filestat_set_times(
        &self,
        atim: wasi::Timestamp,
        mtim: wasi::Timestamp,
        fstflags: wasi::FstFlags,
    ) -> io::Result<()> {
        wasi::fd_filestat_set_times(self.fd, atim, mtim, fstflags)
            .map_err(err2io)
    }

    pub fn filestat_set_size(&self, size: u64) -> io::Result<()> {
        wasi::fd_filestat_set_size(self.fd, size).map_err(err2io)
    }

    pub fn path_filestat_get(
        &self,
        flags: wasi::LookupFlags,
        path: &[u8],
    ) -> io::Result<wasi::FileStat> {
        wasi::path_filestat_get(self.fd, flags, path).map_err(err2io)
    }

    pub fn path_filestat_set_times(
        &self,
        flags: wasi::LookupFlags,
        path: &[u8],
        atim: wasi::Timestamp,
        mtim: wasi::Timestamp,
        fstflags: wasi::FstFlags,
    ) -> io::Result<()> {
        wasi::path_filestat_set_times(
            self.fd,
            flags,
            path,
            atim,
            mtim,
            fstflags,
        ).map_err(err2io)
    }

    pub fn symlink(&self, old_path: &[u8], new_path: &[u8]) -> io::Result<()> {
        wasi::path_symlink(old_path, self.fd, new_path).map_err(err2io)
    }

    pub fn unlink_file(&self, path: &[u8]) -> io::Result<()> {
        wasi::path_unlink_file(self.fd, path).map_err(err2io)
    }

    pub fn remove_directory(&self, path: &[u8]) -> io::Result<()> {
        wasi::path_remove_directory(self.fd, path).map_err(err2io)
    }

    pub fn sock_recv(
        &self,
        ri_data: &mut [IoSliceMut<'_>],
        ri_flags: wasi::RiFlags,
    ) -> io::Result<(usize, wasi::RoFlags)> {
        wasi::sock_recv(self.fd, iovec(ri_data), ri_flags).map_err(err2io)
    }

    pub fn sock_send(&self, si_data: &[IoSlice<'_>], si_flags: wasi::SiFlags) -> io::Result<usize> {
        wasi::sock_send(self.fd, ciovec(si_data), si_flags).map_err(err2io)
    }

    pub fn sock_shutdown(&self, how: Shutdown) -> io::Result<()> {
        let how = match how {
            Shutdown::Read => wasi::SHUT_RD,
            Shutdown::Write => wasi::SHUT_WR,
            Shutdown::Both => wasi::SHUT_WR | wasi::SHUT_RD,
        };
        wasi::sock_shutdown(self.fd, how).map_err(err2io)
    }
}

impl Drop for WasiFd {
    fn drop(&mut self) {
        // FIXME: can we handle the return code here even though we can't on
        // unix?
        let _ = wasi::fd_close(self.fd);
    }
}
