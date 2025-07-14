use std::{
    fs::File,
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    sync::{mpsc::Sender, Arc},
};

use log::{debug, error, info};
use md5::Context;
use rayon::ThreadPoolBuilder;

use crate::{
    database::db::borrow_db_checked,
    download_manager::{
        download_manager::DownloadManagerSignal,
        util::{
            download_thread_control_flag::{DownloadThreadControl, DownloadThreadControlFlag},
            progress_object::{ProgressHandle, ProgressObject},
        },
    },
    error::application_download_error::ApplicationDownloadError,
    games::downloads::{drop_data::DropData, manifest::DropDownloadContext},
    remote::{auth::generate_authorization_header, requests::make_request},
};

pub fn game_validate_logic(
    dropdata: &DropData,
    contexts: Vec<DropDownloadContext>,
    progress: Arc<ProgressObject>,
    sender: Sender<DownloadManagerSignal>,
    control_flag: &DownloadThreadControl,
) -> Result<bool, ApplicationDownloadError> {
    progress.reset(contexts.len());
    let max_download_threads = borrow_db_checked().settings.max_download_threads;

    debug!(
        "validating game: {} with {} threads",
        dropdata.game_id, max_download_threads
    );
    let pool = ThreadPoolBuilder::new()
        .num_threads(max_download_threads)
        .build()
        .unwrap();

    debug!("{:#?}", contexts);
    let invalid_chunks = Arc::new(boxcar::Vec::new());
    pool.scope(|scope| {
        let client = &reqwest::blocking::Client::new();
        for (index, context) in contexts.iter().enumerate() {
            let client = client.clone();

            let current_progress = progress.get(index);
            let progress_handle = ProgressHandle::new(current_progress, progress.clone());
            let invalid_chunks_scoped = invalid_chunks.clone();
            let sender = sender.clone();

            let request = match make_request(
                &client,
                &["/api/v1/client/chunk"],
                &[
                    ("id", &context.game_id),
                    ("version", &context.version),
                    ("name", &context.file_name),
                    ("chunk", &context.index.to_string()),
                ],
                |r| r.header("Authorization", generate_authorization_header()),
            ) {
                Ok(request) => request,
                Err(e) => {
                    sender
                        .send(DownloadManagerSignal::Error(
                            ApplicationDownloadError::Communication(e),
                        ))
                        .unwrap();
                    continue;
                }
            };

            scope.spawn(move |_| {
                match validate_game_chunk(context, control_flag, progress_handle) {
                    Ok(true) => {
                        debug!(
                            "Finished context #{} with checksum {}",
                            index, context.checksum
                        );
                    }
                    Ok(false) => {
                        debug!(
                            "Didn't finish context #{} with checksum {}",
                            index, &context.checksum
                        );
                        invalid_chunks_scoped.push(context.checksum.clone());
                    }
                    Err(e) => {
                        error!("{}", e);
                        sender.send(DownloadManagerSignal::Error(e)).unwrap();
                    }
                }
            });
        }
    });

    // If there are any contexts left which are false
    if !invalid_chunks.is_empty() {
        info!(
            "validation of game id {} failed for chunks {:?}",
            dropdata.game_id.clone(),
            invalid_chunks
        );
        return Ok(false);
    }

    Ok(true)
}

pub fn validate_game_chunk(
    ctx: &DropDownloadContext,
    control_flag: &DownloadThreadControl,
    progress: ProgressHandle,
) -> Result<bool, ApplicationDownloadError> {
    debug!(
        "Starting chunk validation {}, {}, {} #{}",
        ctx.file_name, ctx.index, ctx.offset, ctx.checksum
    );
    // If we're paused
    if control_flag.get() == DownloadThreadControlFlag::Stop {
        progress.set(0);
        return Ok(false);
    }

    let mut source = File::open(&ctx.path).unwrap();

    if ctx.offset != 0 {
        source
            .seek(SeekFrom::Start(ctx.offset))
            .expect("Failed to seek to file offset");
    }

    let mut hasher = md5::Context::new();

    let completed =
        validate_copy(&mut source, &mut hasher, ctx.length, control_flag, progress).unwrap();
    if !completed {
        return Ok(false);
    };

    let res = hex::encode(hasher.compute().0);
    if res != ctx.checksum {
        println!(
            "Checksum failed. Correct: {}, actual: {}",
            &ctx.checksum, &res
        );
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
