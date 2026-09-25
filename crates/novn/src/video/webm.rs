use std::io::{Cursor, Read, Seek};
use std::ptr::NonNull;

use lewton::{audio, header};
use matroska_demuxer::{Frame, MatroskaFile, TrackType};
use rav1d::include::dav1d::{
    data::Dav1dData,
    dav1d::{Dav1dContext, Dav1dSettings},
    headers::*,
    picture::Dav1dPicture,
};
use rav1d::src::lib::*;

use super::decode::{Event, Output, Result, VideoFrame, dimensions, error, threads};
use crate::data::assets::Assets;

const AGAIN: i32 = -libc::EAGAIN;

pub(super) fn decode(assets: &Assets, path: &str, output: &Output) -> Result<()> {
    match assets {
        Assets::Dir(root) => decode_reader(std::fs::File::open(root.join(path))?, output),
        Assets::Embedded(_) => decode_reader(Cursor::new(assets.read(path)?), output),
    }
}

fn decode_reader(reader: impl Read + Seek, output: &Output) -> Result<()> {
    let mut file = MatroskaFile::open(reader).map_err(error)?;
    let track = file
        .tracks()
        .iter()
        .find(|t| t.track_type() == TrackType::Video)
        .ok_or_else(|| error("No video track"))?;
    if track.codec_id() != "V_AV1" {
        return Err(error(
            "The video feature supports AV1 video; enable video-ffmpeg for other codecs",
        ));
    }
    let aspect = track.video().map(|v| {
        v.display_width().unwrap_or(v.pixel_width()).get() as f32
            / v.display_height().unwrap_or(v.pixel_height()).get() as f32
    });
    let video_track = track.track_number().get();
    let duration = track
        .default_duration()
        .map_or(1.0 / 30.0, |d| d.get() as f64 / 1e9);
    let mut vorbis = file.tracks().iter().find(|t| t.track_type() == TrackType::Audio).map(|t| {
        if t.codec_id() != "A_VORBIS" {
            return Err(error("The video feature supports Vorbis audio; enable video-ffmpeg for Opus and other codecs"));
        }
        Vorbis::new(t.track_number().get(), t.codec_private().unwrap_or(&[]))
    }).transpose()?;
    output.send(Event::Info {
        sample_rate: vorbis.as_ref().map(|v| v.ident.audio_sample_rate),
    })?;
    let scale = file.info().timestamp_scale().get() as f64 / 1e9;
    let mut decoder = Av1::new(duration, aspect)?;
    let mut frame = Frame::default();
    let mut origin = None;
    while file.next_frame(&mut frame).map_err(error)? {
        if frame.track != video_track && !vorbis.as_ref().is_some_and(|v| v.track == frame.track) {
            continue;
        }
        let timestamp = frame.timestamp as f64 * scale;
        let pts = timestamp - *origin.get_or_insert(timestamp);
        if frame.track == video_track {
            decoder.packet(&frame.data, pts, output)?;
        } else if let Some(vorbis) = &mut vorbis
            && frame.track == vorbis.track
        {
            vorbis.packet(&frame.data, pts, output)?;
        }
    }
    decoder.drain(output)
}

struct Vorbis {
    track: u64,
    ident: header::IdentHeader,
    setup: header::SetupHeader,
    previous: audio::PreviousWindowRight,
    last: Option<(f64, f64)>,
}

fn headers(bytes: &[u8]) -> Result<[&[u8]; 3]> {
    if bytes.first() != Some(&2) {
        return Err(error("Invalid Vorbis headers"));
    }
    let mut at = 1;
    let mut sizes = [0usize; 2];
    for size in &mut sizes {
        loop {
            let byte = *bytes
                .get(at)
                .ok_or_else(|| error("Truncated Vorbis headers"))?;
            at += 1;
            *size = size
                .checked_add(byte as usize)
                .ok_or_else(|| error("Invalid Vorbis header length"))?;
            if byte != 255 {
                break;
            }
        }
    }
    let first = at
        .checked_add(sizes[0])
        .ok_or_else(|| error("Invalid Vorbis header length"))?;
    let second = first
        .checked_add(sizes[1])
        .ok_or_else(|| error("Invalid Vorbis header length"))?;
    if second >= bytes.len() {
        return Err(error("Truncated Vorbis headers"));
    }
    Ok([&bytes[at..first], &bytes[first..second], &bytes[second..]])
}

impl Vorbis {
    fn new(track: u64, bytes: &[u8]) -> Result<Self> {
        let [ident, comment, setup] = headers(bytes)?;
        let ident = header::read_header_ident(ident).map_err(error)?;
        if !(1..=2).contains(&ident.audio_channels)
            || !(8000..=192000).contains(&ident.audio_sample_rate)
        {
            return Err(error(
                "Vorbis playback requires mono/stereo audio at 8–192 kHz",
            ));
        }
        header::read_header_comment(comment).map_err(error)?;
        let setup = header::read_header_setup(
            setup,
            ident.audio_channels,
            (ident.blocksize_0, ident.blocksize_1),
        )
        .map_err(error)?;
        Ok(Self {
            track,
            ident,
            setup,
            previous: audio::PreviousWindowRight::new(),
            last: None,
        })
    }

    fn packet(&mut self, bytes: &[u8], pts: f64, output: &Output) -> Result<()> {
        let decoded: Vec<Vec<f32>> =
            audio::read_audio_packet_generic(&self.ident, &self.setup, bytes, &mut self.previous)
                .map_err(error)?;
        let left = &decoded[0];
        let right = decoded.get(1).unwrap_or(left);
        if left.is_empty() {
            return Ok(());
        }
        let pts = laced_pts(
            &mut self.last,
            pts,
            left.len(),
            self.ident.audio_sample_rate,
        );
        let samples = left.iter().zip(right).flat_map(|(&l, &r)| [l, r]).collect();
        output.send(Event::Audio { pts, samples })
    }
}

fn laced_pts(last: &mut Option<(f64, f64)>, pts: f64, frames: usize, rate: u32) -> f64 {
    let start = match *last {
        Some((block, end)) if block == pts => end,
        _ => pts,
    };
    *last = Some((pts, start + frames as f64 / f64::from(rate)));
    start
}

struct Av1 {
    context: Option<Dav1dContext>,
    duration: f64,
    aspect: Option<f32>,
}

impl Av1 {
    fn new(duration: f64, aspect: Option<f32>) -> Result<Self> {
        let mut settings = std::mem::MaybeUninit::<Dav1dSettings>::uninit();
        unsafe {
            dav1d_default_settings(NonNull::from(&mut settings).cast());
        }
        let mut settings = unsafe { settings.assume_init() };
        settings.n_threads = threads() as i32;
        settings.max_frame_delay = 0;
        settings.frame_size_limit = 4096 * 2160;
        let mut decoder = Self {
            context: None,
            duration,
            aspect,
        };
        let result = unsafe {
            dav1d_open(
                Some(NonNull::from(&mut decoder.context)),
                Some(NonNull::from(&mut settings)),
            )
        };
        if result.0 != 0 {
            return Err(error(format!("Could not open AV1 decoder: {}", result.0)));
        }
        Ok(decoder)
    }

    fn packet(&mut self, bytes: &[u8], pts: f64, output: &Output) -> Result<()> {
        if bytes.is_empty() {
            return Err(error("Empty AV1 packet"));
        }
        let mut data = Data(Dav1dData::default());
        let buffer = unsafe { dav1d_data_create(Some(NonNull::from(&mut data.0)), bytes.len()) };
        if buffer.is_null() {
            return Err(error("Could not allocate AV1 packet"));
        }
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len());
        }
        data.0.m.timestamp = (pts * 1e9).round() as i64;
        loop {
            let result = unsafe { dav1d_send_data(self.context, Some(NonNull::from(&mut data.0))) };
            self.drain(output)?;
            if result.0 == AGAIN {
                continue;
            }
            if result.0 != 0 {
                return Err(error(format!("Invalid AV1 packet: {}", result.0)));
            }
            return Ok(());
        }
    }

    fn drain(&mut self, output: &Output) -> Result<()> {
        loop {
            let mut picture = Picture(Dav1dPicture::default());
            let result =
                unsafe { dav1d_get_picture(self.context, Some(NonNull::from(&mut picture.0))) };
            if result.0 == AGAIN {
                return Ok(());
            }
            if result.0 != 0 {
                return Err(error(format!("AV1 decode failed: {}", result.0)));
            }
            let mut frame = picture.frame(self.duration)?;
            if let Some(aspect) = self.aspect {
                frame.aspect = aspect;
            }
            output.send(Event::Video(frame))?;
        }
    }
}

impl Drop for Av1 {
    fn drop(&mut self) {
        unsafe {
            dav1d_close(Some(NonNull::from(&mut self.context)));
        }
    }
}

struct Data(Dav1dData);
impl Drop for Data {
    fn drop(&mut self) {
        unsafe {
            dav1d_data_unref(Some(NonNull::from(&mut self.0)));
        }
    }
}

struct Picture(Dav1dPicture);
impl Drop for Picture {
    fn drop(&mut self) {
        unsafe {
            dav1d_picture_unref(Some(NonNull::from(&mut self.0)));
        }
    }
}

impl Picture {
    fn frame(&self, duration: f64) -> Result<VideoFrame> {
        let p = &self.0;
        let width = u32::try_from(p.p.w).map_err(error)?;
        let height = u32::try_from(p.p.h).map_err(error)?;
        let size = dimensions(width, height)?;
        if p.p.bpc != 8 {
            return Err(error("The Rust video backend requires 8-bit AV1"));
        }
        let (sx, sy) = match p.p.layout {
            DAV1D_PIXEL_LAYOUT_I420 => (1, 1),
            DAV1D_PIXEL_LAYOUT_I422 => (1, 0),
            DAV1D_PIXEL_LAYOUT_I444 => (0, 0),
            _ => return Err(error("Unsupported AV1 pixel layout")),
        };
        let header = unsafe {
            p.seq_hdr
                .ok_or_else(|| error("Missing AV1 sequence header"))?
                .as_ref()
        };
        let (kr, kb) = match header.mtrx {
            DAV1D_MC_BT709 => (0.2126, 0.0722),
            DAV1D_MC_BT601 | DAV1D_MC_BT470BG => (0.299, 0.114),
            DAV1D_MC_UNKNOWN => {
                if height >= 720 {
                    (0.2126, 0.0722)
                } else {
                    (0.299, 0.114)
                }
            }
            _ => {
                return Err(error(
                    "Unsupported AV1 colour matrix; export SDR BT.709 or use video-ffmpeg",
                ));
            }
        };
        let planes: Vec<_> = p
            .data
            .iter()
            .map(|plane| plane.ok_or_else(|| error("Missing AV1 plane")))
            .collect::<Result<_>>()?;
        let matrix = Matrix::new(header.color_range != 0, kr, kb);
        let (width, height) = (width as usize, height as usize);
        let chroma_width = (width + sx) >> sx;
        let row = |plane: usize, y: usize, len: usize| unsafe {
            std::slice::from_raw_parts(
                planes[plane]
                    .as_ptr()
                    .cast::<u8>()
                    .offset(y as isize * p.stride[usize::from(plane != 0)]),
                len,
            )
        };
        let mut rgba = vec![0; size];
        for (y, out) in rgba.chunks_exact_mut(width * 4).enumerate() {
            let luma = row(0, y, width);
            let cb = row(1, y >> sy, chroma_width);
            let cr = row(2, y >> sy, chroma_width);
            for (x, (pixel, &luma)) in out.as_chunks_mut::<4>().0.iter_mut().zip(luma).enumerate() {
                let [r, g, b] = matrix.rgb(luma, cb[x >> sx], cr[x >> sx]);
                *pixel = [r, g, b, 255];
            }
        }
        let (width, height) = (width as u32, height as u32);
        Ok(VideoFrame {
            pts: p.m.timestamp as f64 / 1e9,
            duration,
            width,
            height,
            aspect: width as f32 / height as f32,
            rgba,
        })
    }
}

struct Matrix {
    offset: i32,
    luma: i32,
    red: i32,
    blue: i32,
    green_cr: i32,
    green_cb: i32,
}

impl Matrix {
    fn new(full: bool, kr: f32, kb: f32) -> Self {
        let (offset, luma, chroma) = if full {
            (0, 1.0, 1.0)
        } else {
            (16, 255.0 / 219.0, 255.0 / 224.0)
        };
        let kg = 1.0 - kr - kb;
        let red = 2.0 * (1.0 - kr) * chroma;
        let blue = 2.0 * (1.0 - kb) * chroma;
        let fixed = |v: f32| (v * 65536.0).round() as i32;
        Self {
            offset,
            luma: fixed(luma),
            red: fixed(red),
            blue: fixed(blue),
            green_cr: fixed(red * kr / kg),
            green_cb: fixed(blue * kb / kg),
        }
    }

    fn rgb(&self, y: u8, cb: u8, cr: u8) -> [u8; 3] {
        let y = (i32::from(y) - self.offset) * self.luma + 32768;
        let cb = i32::from(cb) - 128;
        let cr = i32::from(cr) - 128;
        [
            y + self.red * cr,
            y - self.green_cr * cr - self.green_cb * cb,
            y + self.blue * cb,
        ]
        .map(|v| (v >> 16).clamp(0, 255) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vorbis_headers_reject_truncation() {
        for bytes in [&[][..], &[2], &[2, 255], &[2, 9, 2, 1]] {
            assert!(headers(bytes).is_err());
        }
        assert_eq!(
            headers(&[2, 1, 1, 11, 22, 33]).unwrap(),
            [&[11][..], &[22], &[33]]
        );
    }

    #[test]
    fn laced_frames_continue_from_the_previous_packet() {
        let mut last = None;
        assert_eq!(laced_pts(&mut last, 1.0, 500, 1000), 1.0);
        assert_eq!(laced_pts(&mut last, 1.0, 500, 1000), 1.5);
        assert_eq!(laced_pts(&mut last, 1.0, 250, 1000), 2.0);
        assert_eq!(laced_pts(&mut last, 2.25, 100, 1000), 2.25);
    }

    #[test]
    fn colour_range_preserves_black_white_and_red() {
        let limited = Matrix::new(false, 0.2126, 0.0722);
        assert_eq!(limited.rgb(16, 128, 128), [0; 3]);
        assert_eq!(limited.rgb(235, 128, 128), [255; 3]);
        assert_eq!(Matrix::new(true, 0.2126, 0.0722).rgb(0, 128, 128), [0; 3]);
        assert_eq!(
            Matrix::new(true, 0.2126, 0.0722).rgb(255, 128, 128),
            [255; 3]
        );
        let red = Matrix::new(false, 0.299, 0.114).rgb(81, 90, 240);
        assert!(red[0] >= 253 && red[1] <= 1 && red[2] <= 1, "{red:?}");
    }
}
