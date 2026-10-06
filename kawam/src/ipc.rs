use crate::{kawa::{state::AppState}, socket_path};
use kawactl::{request::Request as IpcReq, Response};
use serde::Serialize;
use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    os::{
        fd::{AsRawFd, RawFd},
        unix::net::{UnixListener, UnixStream},
    },
    time::Duration,
};
use tracing::{error, info};

pub struct IpcServer {
    listener: UnixListener,
    pendings: Vec<KawaRequest>,
}

impl IpcServer {
    pub fn bind() -> io::Result<Self> {
        let path = socket_path();
        let _ = fs::remove_file(&path);
        let listener = UnixListener::bind(&path).expect("Unable to create IPC Socket");
        listener
            .set_nonblocking(true)
            .expect("Unable to set Socket non-blocking");
        info!("IPC Socket create at {:?}", path.as_path());

        Ok(Self {
            listener,
            pendings: Vec::<KawaRequest>::new(),
        })
    }

    pub fn raw_fd(&self) -> RawFd {
        self.listener.as_raw_fd()
    }

    /// Accept every connection until WouldBlock
    pub fn drain(&self) -> Vec<KawaRequest> {
        let mut out = Vec::new();
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => match KawaRequest::read(stream) {
                    Ok(req) => out.push(req),
                    Err(e) => error!("ipc: ignored request : {e}"),
                },
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    error!("ipc: accept : {e}");
                    break;
                }
            }
        }
        out
    }

    pub fn handle_connections(&mut self, state: &AppState) {
        self.pendings.extend(self.drain());
        // Borrow context and get needed informations
        // Safe because we are in the conditional poll event for IPC
        // So there must be no context borrowal at this state
        let context = state.context.borrow();


        for req in self.pendings.drain(..) {
            let data = serde_json::json!({
                "caca":"oui"
            });
            let _ = req.reply(&Response::Ok { data: data });
        }
        drop(context);
    }

}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = fs::remove_file(socket_path());
    }
}

#[derive(Debug)]
pub struct KawaRequest {
    pub req: IpcReq,
    stream: UnixStream,
}

impl KawaRequest {
    fn read(stream: UnixStream) -> io::Result<Self> {
        // We block the socket for the time of operation
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(Duration::from_millis(200)))?;
        stream.set_write_timeout(Some(Duration::from_millis(200)))?;

        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line)?;
        Ok(KawaRequest {
            req: serde_json::from_str(&line)?,
            stream,
        })
    }

    // Send a JSON to the IPC Client
    pub fn reply<T: Serialize>(mut self, msg: &T) -> io::Result<()> {
        serde_json::to_writer(&mut self.stream, msg)?; 
        self.stream.write_all(b"\n")
    }
}
