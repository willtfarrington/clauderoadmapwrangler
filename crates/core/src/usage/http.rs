//! The meter's one HTTP request (EP-17): a GET through WinHTTP, Windows' own HTTP stack, so that
//! TLS, the certificate store and the proxy settings are the system's and no client crate joins
//! the lock (the owner's choice). The header block, which carries the token, is a [`Secret`]: it
//! is copied once into the wide string WinHTTP takes, which is overwritten as soon as the request
//! is sent. A loopback host (the tests' listener) is reached without a proxy and over plain HTTP.

use std::time::Duration;

use super::Secret;

/// The most of an answer read; a longer one is refused.
pub const BODY_CAP: usize = 1 << 20;

/// Each step's limit (resolving, connecting, sending, receiving): a poll ends within about 30 s.
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// An answer: its status, the wait a `Retry-After` in seconds asks for, and its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub retry_after: Option<Duration>,
    pub body: Vec<u8>,
}

/// Where a request goes: `http://` or `https://`, a host, a port and a path with its query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    pub secure: bool,
    pub host: String,
    pub port: u16,
    pub path: String,
}

impl Url {
    /// `None` for anything but an `http` or `https` URL with a host.
    pub fn parse(text: &str) -> Option<Url> {
        let text = text.trim();
        let (secure, rest) = if let Some(rest) = text.strip_prefix("https://") {
            (true, rest)
        } else {
            (false, text.strip_prefix("http://")?)
        };
        let (authority, path) = match rest.find('/') {
            Some(at) => rest.split_at(at),
            None => (rest, "/"),
        };
        let (host, port) = match authority.rsplit_once(':') {
            Some((host, port)) => (host, port.parse().ok()?),
            None => (authority, if secure { 443 } else { 80 }),
        };
        if host.is_empty() || host.contains('@') {
            return None;
        }
        Some(Url {
            secure,
            host: host.to_owned(),
            port,
            path: path.to_owned(),
        })
    }

    /// A host on this machine, reached without a proxy.
    pub fn is_loopback(&self) -> bool {
        matches!(self.host.as_str(), "127.0.0.1" | "localhost" | "[::1]")
    }
}

/// GETs `url` with `headers` (`Name: value` lines, each ended by CRLF) as `agent`, each step
/// within `timeout`. The error is in words, never with a header's text.
#[cfg(windows)]
pub fn get(
    url: &Url,
    headers: &Secret,
    agent: &str,
    timeout: Duration,
) -> Result<Response, String> {
    use windows::Win32::Networking::WinHttp::{
        WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_ACCESS_TYPE_NO_PROXY, WINHTTP_FLAG_SECURE,
        WINHTTP_OPEN_REQUEST_FLAGS, WINHTTP_QUERY_RETRY_AFTER, WINHTTP_QUERY_STATUS_CODE,
        WinHttpConnect, WinHttpOpen, WinHttpOpenRequest, WinHttpQueryDataAvailable,
        WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest, WinHttpSetTimeouts,
    };
    use windows::core::{HSTRING, PCWSTR};

    let step = i32::try_from(timeout.as_millis()).unwrap_or(i32::MAX);
    let access = if url.is_loopback() {
        WINHTTP_ACCESS_TYPE_NO_PROXY
    } else {
        WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY
    };
    // SAFETY (each block below): WinHTTP is given handles it returned, which `Handle` closes once,
    // the children before their parents (the locals drop in reverse order), and buffers that
    // outlive the call.
    let session = Handle(unsafe {
        WinHttpOpen(
            &HSTRING::from(agent),
            access,
            PCWSTR::null(),
            PCWSTR::null(),
            0,
        )
    });
    session.check("open a session")?;
    unsafe { WinHttpSetTimeouts(session.0, step, step, step, step) }
        .map_err(|e| failed("set its time limits", &e))?;
    let connection = Handle(unsafe {
        WinHttpConnect(session.0, &HSTRING::from(url.host.as_str()), url.port, 0)
    });
    connection.check("reach the host")?;
    let flags = if url.secure {
        WINHTTP_FLAG_SECURE
    } else {
        WINHTTP_OPEN_REQUEST_FLAGS(0)
    };
    let request = Handle(unsafe {
        WinHttpOpenRequest(
            connection.0,
            &HSTRING::from("GET"),
            &HSTRING::from(url.path.as_str()),
            PCWSTR::null(),
            PCWSTR::null(),
            std::ptr::null(),
            flags,
        )
    });
    request.check("open the request")?;
    let mut wide: Vec<u16> = headers.expose().encode_utf16().collect();
    let sent = unsafe { WinHttpSendRequest(request.0, Some(&wide), None, 0, 0, 0) };
    super::wipe_wide(&mut wide);
    sent.map_err(|e| failed("send the request", &e))?;
    unsafe { WinHttpReceiveResponse(request.0, std::ptr::null_mut()) }
        .map_err(|e| failed("receive the answer", &e))?;
    let status = number(&request, WINHTTP_QUERY_STATUS_CODE)
        .and_then(|code| u16::try_from(code).ok())
        .ok_or("the answer has no status")?;
    // WinHTTP turns the status into a number, but not a `Retry-After`.
    let retry_after = text(&request, WINHTTP_QUERY_RETRY_AFTER)
        .and_then(|text| text.trim().parse::<u64>().ok())
        .map(Duration::from_secs);
    let mut body = Vec::new();
    loop {
        let mut available = 0u32;
        unsafe { WinHttpQueryDataAvailable(request.0, &mut available) }
            .map_err(|e| failed("read the answer", &e))?;
        if available == 0 {
            break;
        }
        let start = body.len();
        let take = (available as usize).min(BODY_CAP + 1 - start);
        body.resize(start + take, 0);
        let mut read = 0u32;
        unsafe {
            WinHttpReadData(
                request.0,
                body[start..].as_mut_ptr().cast(),
                take as u32,
                &mut read,
            )
        }
        .map_err(|e| failed("read the answer", &e))?;
        body.truncate(start + read as usize);
        if body.len() > BODY_CAP {
            return Err(format!("the answer is over {} KB", BODY_CAP / 1024));
        }
        if read == 0 {
            break;
        }
    }
    Ok(Response {
        status,
        retry_after,
        body,
    })
}

#[cfg(not(windows))]
pub fn get(
    _url: &Url,
    _headers: &Secret,
    _agent: &str,
    _timeout: Duration,
) -> Result<Response, String> {
    Err("the meter's request is made through Windows' WinHTTP".to_owned())
}

/// A WinHTTP handle, closed when dropped.
#[cfg(windows)]
struct Handle(*mut core::ffi::c_void);

#[cfg(windows)]
impl Handle {
    /// The step's failure, in words, when WinHTTP returned no handle.
    fn check(&self, step: &str) -> Result<(), String> {
        if self.0.is_null() {
            return Err(failed(step, &windows::core::Error::from_thread()));
        }
        Ok(())
    }
}

#[cfg(windows)]
impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: the handle came from WinHTTP and is closed once.
            let _ = unsafe { windows::Win32::Networking::WinHttp::WinHttpCloseHandle(self.0) };
        }
    }
}

/// A header of the answer as text; `None` when it is absent.
#[cfg(windows)]
fn text(request: &Handle, level: u32) -> Option<String> {
    use windows::Win32::Networking::WinHttp::WinHttpQueryHeaders;
    use windows::core::PCWSTR;

    // The first call fails, saying how many bytes the value takes.
    let mut size = 0u32;
    // SAFETY: no buffer, so nothing is written but the size.
    let _ = unsafe {
        WinHttpQueryHeaders(
            request.0,
            level,
            PCWSTR::null(),
            None,
            &mut size,
            std::ptr::null_mut(),
        )
    };
    if size == 0 {
        return None;
    }
    let mut buffer = vec![0u16; (size as usize).div_ceil(2)];
    // SAFETY: the buffer holds `size` bytes.
    unsafe {
        WinHttpQueryHeaders(
            request.0,
            level,
            PCWSTR::null(),
            Some(buffer.as_mut_ptr().cast()),
            &mut size,
            std::ptr::null_mut(),
        )
    }
    .ok()?;
    buffer.truncate(size as usize / 2);
    Some(String::from_utf16_lossy(&buffer))
}

/// A header of the answer as a number; `None` when it is absent or not a number.
#[cfg(windows)]
fn number(request: &Handle, level: u32) -> Option<u32> {
    use windows::Win32::Networking::WinHttp::{WINHTTP_QUERY_FLAG_NUMBER, WinHttpQueryHeaders};
    use windows::core::PCWSTR;

    let mut value = 0u32;
    let mut size = std::mem::size_of::<u32>() as u32;
    // SAFETY: the buffer is a u32 of the size given.
    unsafe {
        WinHttpQueryHeaders(
            request.0,
            level | WINHTTP_QUERY_FLAG_NUMBER,
            PCWSTR::null(),
            Some((&mut value as *mut u32).cast()),
            &mut size,
            std::ptr::null_mut(),
        )
    }
    .ok()
    .map(|()| value)
}

#[cfg(windows)]
fn failed(step: &str, error: &windows::core::Error) -> String {
    let message = error.message();
    let message = message.trim().trim_end_matches('.');
    if message.is_empty() {
        format!("WinHTTP could not {step} ({:#010x})", error.code().0)
    } else {
        format!("WinHTTP could not {step}: {message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_read_their_parts() {
        assert_eq!(
            Url::parse("https://api.anthropic.com/api/oauth/usage"),
            Some(Url {
                secure: true,
                host: "api.anthropic.com".to_owned(),
                port: 443,
                path: "/api/oauth/usage".to_owned(),
            })
        );
        let local = Url::parse("http://127.0.0.1:5123/usage?x=1").unwrap();
        assert_eq!((local.secure, local.port), (false, 5123));
        assert_eq!(local.path, "/usage?x=1");
        assert!(local.is_loopback());
        assert_eq!(Url::parse("http://example.org").unwrap().path, "/");
        for wrong in [
            "ftp://host/",
            "https://",
            "http://:80/",
            "https://u@host/",
            "x",
        ] {
            assert_eq!(Url::parse(wrong), None, "{wrong}");
        }
    }
}
