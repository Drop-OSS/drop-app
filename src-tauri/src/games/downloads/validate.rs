use std::{
    fs::File,
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
};

use log::debug;
use md5::Context;

use crate::{
    download_manager::util::{
        download_thread_control_flag::{DownloadThreadControl, DownloadThreadControlFlag},
        progress_object::ProgressHandle,
    },
    error::application_download_error::ApplicationDownloadError,
    games::downloads::manifest::DropValidateContext,
};

pub fn validate_game_chunk(
    ctx: &DropValidateContext,
    control_flag: &DownloadThreadControl,
    progress: ProgressHandle,
) -> Result<bool, ApplicationDownloadError> {
    debug!(
        "Starting chunk validation {}, {}, {} #{}",
        ctx.path.display(), ctx.index, ctx.offset, ctx.checksum
    );
    // If we're paused
    if control_flag.get() == DownloadThreadControlFlag::Stop {
        progress.set(0);
        return Ok(false);
    }

    let Ok(mut source) = File::open(&ctx.path) else {
        return Ok(false);
    };

    if ctx.offset != 0 {
        source
            .seek(SeekFrom::Start(ctx.offset.try_into().unwrap()))
            .expect("Failed to seek to file offset");
    }

    let mut hasher = md5::Context::new();

    let completed =
        validate_copy(&mut source, &mut hasher, ctx.length, control_flag, progress).unwrap();
    if !completed {
        return Ok(false);
    }

    let res = hex::encode(hasher.compute().0);
    if res != ctx.checksum {
        return Ok(false);
    }

    debug!(
        "Successfully finished verification #{}, copied {} bytes",
        ctx.checksum, ctx.length
    );

    Ok(true)
}

fn validate_copy(
    source: &mut File,
    dest: &mut Context,
    size: usize,
    control_flag: &DownloadThreadControl,
    progress: ProgressHandle,
) -> Result<bool, io::Error> {
    let copy_buf_size = 512;
    let mut copy_buf = vec![0; copy_buf_size];
    let mut buf_writer = BufWriter::with_capacity(1024 * 1024, dest);
    let mut total_bytes = 0;

    loop {
        if control_flag.get() == DownloadThreadControlFlag::Stop {
            buf_writer.flush()?;
            return Ok(false);
        }

        let mut bytes_read = source.read(&mut copy_buf)?;
        total_bytes += bytes_read;

        // If we read over (likely), truncate our read to
        // the right size
        if total_bytes > size {
            let over = total_bytes - size;
            bytes_read -= over;
            total_bytes = size;
        }

        buf_writer.write_all(&copy_buf[0..bytes_read])?;
        progress.add(bytes_read);

        if total_bytes >= size {
            break;
        }
    }
    buf_writer.flush()?;
    Ok(true)
}
