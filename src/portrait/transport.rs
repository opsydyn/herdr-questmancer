//! Supply missing PTY pixel geometry from the managed pane's authoritative host.
use std::{os::fd::AsFd, time::Duration};

use rustix::termios::{Winsize, tcgetwinsize, tcsetwinsize};
use thiserror::Error;

use crate::{
    domain::PaneId,
    herdr::{
        client::{ClientError, HerdrClient},
        environment::HerdrEnvironment,
        protocol::PaneGraphicsInfo,
    },
};

const GEOMETRY_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Error)]
pub(super) enum GeometryError {
    #[error("Herdr portrait geometry request timed out")]
    Timeout,
    #[error(transparent)]
    Client(#[from] ClientError),
    #[error("portrait terminal geometry: {0}")]
    Terminal(#[from] rustix::io::Errno),
    #[error("Herdr portrait cell size is zero or exceeds PTY pixel bounds")]
    InvalidSize,
}

pub(super) async fn supply_missing_pixel_size(
    environment: &HerdrEnvironment,
    pane_id: &PaneId,
    terminal: &impl AsFd,
) -> Result<(), GeometryError> {
    if !needs_pixels(tcgetwinsize(terminal)?) {
        return Ok(());
    }
    let client = HerdrClient::new(environment.socket_path());
    let size = tokio::time::timeout(
        GEOMETRY_TIMEOUT,
        client.pane_graphics_info(pane_id.as_str()),
    )
    .await
    .map_err(|_| GeometryError::Timeout)??;
    // The pane may resize while the socket request is in flight. Preserve the
    // latest grid and any pixel dimensions already supplied by its owner.
    if let Some(completed) = completed_size(tcgetwinsize(terminal)?, size)? {
        tcsetwinsize(terminal, completed)?;
    }
    Ok(())
}

fn needs_pixels(size: Winsize) -> bool {
    size.ws_col > 0 && size.ws_row > 0 && (size.ws_xpixel == 0 || size.ws_ypixel == 0)
}

fn completed_size(
    mut current: Winsize,
    host: PaneGraphicsInfo,
) -> Result<Option<Winsize>, GeometryError> {
    if !needs_pixels(current) {
        return Ok(None);
    }
    let fill = |existing, cells, pixels: u32| {
        if existing != 0 {
            return Ok(existing);
        }
        pixels
            .checked_mul(u32::from(cells))
            .and_then(|value| u16::try_from(value).ok())
            .filter(|value| *value > 0)
            .ok_or(GeometryError::InvalidSize)
    };
    current.ws_xpixel = fill(current.ws_xpixel, current.ws_col, host.cell_width_px)?;
    current.ws_ypixel = fill(current.ws_ypixel, current.ws_row, host.cell_height_px)?;
    Ok(Some(current))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustix::{
        pty::{OpenptFlags, openpt},
        termios::{Winsize, tcgetwinsize, tcsetwinsize},
    };
    use serde_json::json;
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::UnixListener,
    };

    fn pty(size: Winsize) -> (std::os::fd::OwnedFd, std::os::fd::OwnedFd) {
        let master = openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).unwrap();
        rustix::pty::grantpt(&master).unwrap();
        rustix::pty::unlockpt(&master).unwrap();
        let name = rustix::pty::ptsname(&master, Vec::new()).unwrap();
        let terminal = rustix::fs::open(
            name,
            rustix::fs::OFlags::RDWR | rustix::fs::OFlags::NOCTTY,
            rustix::fs::Mode::empty(),
        )
        .unwrap();
        tcsetwinsize(&terminal, size).unwrap();
        (master, terminal)
    }

    fn zero_pixels() -> Winsize {
        Winsize {
            ws_row: 54,
            ws_col: 205,
            ws_xpixel: 0,
            ws_ypixel: 0,
        }
    }

    #[tokio::test]
    async fn managed_pane_gets_missing_pixels_from_herdr_without_resizing_its_grid() {
        let (_master, terminal) = pty(zero_pixels());
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("herdr.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let request: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["method"], "pane.graphics.info");
            assert_eq!(request["params"], json!({ "pane_id": "w8:pD" }));
            let response = json!({ "id": request["id"], "result": { "type": "pane_graphics_info", "cell_width_px": 8, "cell_height_px": 18, "pane_visible": false }});
            reader
                .get_mut()
                .write_all(format!("{response}\n").as_bytes())
                .await
                .unwrap();
        });
        supply_missing_pixel_size(
            &HerdrEnvironment::new(socket, "herdr"),
            &PaneId::new("w8:pD"),
            &terminal,
        )
        .await
        .unwrap();
        let after = tcgetwinsize(&terminal).unwrap();
        assert_eq!((after.ws_xpixel, after.ws_ypixel), (1640, 972));
        assert_eq!((after.ws_col, after.ws_row), (205, 54));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn existing_pixels_require_no_socket_request_or_change() {
        let original = Winsize {
            ws_xpixel: 1640,
            ws_ypixel: 972,
            ..zero_pixels()
        };
        let (_master, terminal) = pty(original);
        let missing = tempfile::tempdir().unwrap();
        supply_missing_pixel_size(
            &HerdrEnvironment::new(missing.path().join("absent.sock"), "herdr"),
            &PaneId::new("w1:p1"),
            &terminal,
        )
        .await
        .unwrap();
        let after = tcgetwinsize(&terminal).unwrap();
        assert_eq!(
            (after.ws_col, after.ws_row, after.ws_xpixel, after.ws_ypixel),
            (
                original.ws_col,
                original.ws_row,
                original.ws_xpixel,
                original.ws_ypixel
            )
        );
    }

    #[test]
    fn only_missing_pixel_axes_are_completed() {
        let original = Winsize {
            ws_xpixel: 1800,
            ..zero_pixels()
        };
        let after = completed_size(
            original,
            PaneGraphicsInfo {
                cell_width_px: 8,
                cell_height_px: 18,
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!((after.ws_xpixel, after.ws_ypixel), (1800, 972));
    }

    #[test]
    fn zero_or_overflowing_host_sizes_are_rejected() {
        for (width, height) in [(0, 18), (8, 0), (u32::MAX, 18), (400, 18), (8, u32::MAX)] {
            assert!(matches!(
                completed_size(
                    zero_pixels(),
                    PaneGraphicsInfo {
                        cell_width_px: width,
                        cell_height_px: height
                    }
                ),
                Err(GeometryError::InvalidSize)
            ));
        }
    }

    #[test]
    fn an_empty_terminal_grid_cannot_get_invented_pixels() {
        for current in [
            Winsize {
                ws_col: 0,
                ..zero_pixels()
            },
            Winsize {
                ws_row: 0,
                ..zero_pixels()
            },
        ] {
            assert!(
                completed_size(
                    current,
                    PaneGraphicsInfo {
                        cell_width_px: 8,
                        cell_height_px: 18
                    }
                )
                .unwrap()
                .is_none()
            );
        }
    }

    #[tokio::test]
    async fn invalid_or_denied_graphics_responses_preserve_missing_pixels() {
        for response in [
            json!({"error": {"code": "feature_disabled", "message": "disabled"}}),
            json!({"result": {"type": "wrong_type", "cell_width_px": 8, "cell_height_px": 18}}),
            json!({"result": {"type": "pane_graphics_info", "cell_height_px": 18}}),
            json!({"result": {"type": "pane_graphics_info", "cell_width_px": 0, "cell_height_px": 18}}),
            json!({"id": "stale-reply", "result": {"type": "pane_graphics_info", "cell_width_px": 8, "cell_height_px": 18}}),
        ] {
            let (_master, terminal) = pty(zero_pixels());
            let directory = tempfile::tempdir().unwrap();
            let socket = directory.path().join("herdr.sock");
            let listener = UnixListener::bind(&socket).unwrap();
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).await.unwrap();
                let request: serde_json::Value = serde_json::from_str(&line).unwrap();
                let mut response = response;
                if response.get("id").is_none() {
                    response["id"] = request["id"].clone();
                }
                reader
                    .get_mut()
                    .write_all(format!("{response}\n").as_bytes())
                    .await
                    .unwrap();
            });
            assert!(
                supply_missing_pixel_size(
                    &HerdrEnvironment::new(socket, "herdr"),
                    &PaneId::new("w1:p1"),
                    &terminal
                )
                .await
                .is_err()
            );
            let after = tcgetwinsize(&terminal).unwrap();
            assert_eq!((after.ws_xpixel, after.ws_ypixel), (0, 0));
            server.await.unwrap();
        }
    }

    #[tokio::test(start_paused = true)]
    async fn an_unresponsive_host_cannot_hold_up_startup_indefinitely() {
        let (_master, terminal) = pty(zero_pixels());
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("herdr.sock");
        let _listener = UnixListener::bind(&socket).unwrap();
        let start = tokio::time::Instant::now();
        let result = supply_missing_pixel_size(
            &HerdrEnvironment::new(socket, "herdr"),
            &PaneId::new("w1:p1"),
            &terminal,
        )
        .await;
        assert!(matches!(result, Err(GeometryError::Timeout)));
        assert_eq!(start.elapsed(), GEOMETRY_TIMEOUT);
        let after = tcgetwinsize(&terminal).unwrap();
        assert_eq!((after.ws_xpixel, after.ws_ypixel), (0, 0));
    }
}
