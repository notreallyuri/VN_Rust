use std::io::Write;

use ff::{ChannelLayout, codec, format, frame, media, software};
use ffmpeg_next as ff;

use super::decode::{Event, Output, Result, VideoFrame, dimensions, error, threads};
use crate::data::assets::Assets;

pub(super) fn decode(assets: &Assets, path: &str, output: &Output) -> Result<()> {
    ff::init().map_err(error)?;
    let mut temporary = None;
    let path = match assets {
        Assets::Dir(root) => root.join(path),
        Assets::Embedded(_) => {
            let mut file = tempfile::NamedTempFile::new()?;
            file.write_all(&assets.read(path)?)?;
            file.flush()?;
            let path = file.path().to_path_buf();
            temporary = Some(file);
            path
        }
    };
    let mut options = ff::Dictionary::new();
    options.set("protocol_whitelist", "file");
    let mut input = format::input_with_dictionary(&path, options).map_err(error)?;
    let stream = input
        .streams()
        .best(media::Type::Video)
        .ok_or_else(|| error("No video track"))?;
    check_codec(stream.parameters().id())?;
    let video_index = stream.index();
    let video_time: f64 = stream.time_base().into();
    let fps: f64 = stream.avg_frame_rate().into();
    let duration = if fps.is_finite() && fps > 0.0 {
        1.0 / fps
    } else {
        1.0 / 30.0
    };
    let mut context = open(&stream)?;
    context.set_threading(codec::threading::Config {
        count: threads(),
        ..codec::threading::Config::kind(codec::threading::Type::Frame)
    });
    let mut video = context.decoder().video().map_err(error)?;
    dimensions(video.width(), video.height())?;
    let origin = input
        .streams()
        .filter(|s| {
            matches!(
                s.parameters().medium(),
                media::Type::Video | media::Type::Audio
            )
        })
        .filter(|s| s.start_time() != ff::ffi::AV_NOPTS_VALUE)
        .map(|s| s.start_time() as f64 * f64::from(s.time_base()))
        .reduce(f64::min)
        .unwrap_or(0.0);
    let mut audio = input
        .streams()
        .best(media::Type::Audio)
        .map(|stream| {
            let decoder = open(&stream)?.decoder().audio().map_err(error)?;
            Audio::new(stream.index(), stream.time_base().into(), decoder)
        })
        .transpose()?;
    output.send(Event::Info {
        sample_rate: audio.as_ref().map(|a| a.rate),
    })?;
    let mut scaler = None;
    let mut next_video_pts = 0.0;
    loop {
        let mut packet = ff::Packet::empty();
        match packet.read(&mut input) {
            Ok(()) => {}
            Err(ff::Error::Eof) => break,
            Err(e) => return Err(error(e)),
        }
        if packet.stream() == video_index {
            video.send_packet(&packet).map_err(error)?;
            drain_video(
                &mut video,
                &mut scaler,
                video_time,
                origin,
                duration,
                &mut next_video_pts,
                output,
            )?;
        } else if let Some(audio) = &mut audio
            && packet.stream() == audio.index
        {
            audio.decoder.send_packet(&packet).map_err(error)?;
            audio.drain(origin, output)?;
        }
    }
    video.send_eof().map_err(error)?;
    drain_video(
        &mut video,
        &mut scaler,
        video_time,
        origin,
        duration,
        &mut next_video_pts,
        output,
    )?;
    if let Some(audio) = &mut audio {
        audio.decoder.send_eof().map_err(error)?;
        audio.drain(origin, output)?;
        audio.flush(output)?;
    }
    drop(input);
    drop(temporary);
    Ok(())
}

fn open(stream: &format::stream::Stream) -> Result<codec::context::Context> {
    let mut context =
        codec::context::Context::from_parameters(stream.parameters()).map_err(error)?;
    unsafe {
        (*context.as_mut_ptr()).pkt_timebase = stream.time_base().into();
    }
    Ok(context)
}

fn again(e: ff::Error) -> bool {
    e == ff::Error::Eof || matches!(e, ff::Error::Other { errno } if errno == ff::error::EAGAIN)
}

fn drain_video(
    decoder: &mut codec::decoder::Video,
    scaler: &mut Option<software::scaling::Context>,
    time: f64,
    origin: f64,
    duration: f64,
    next_pts: &mut f64,
    output: &Output,
) -> Result<()> {
    loop {
        let mut decoded = frame::Video::empty();
        match decoder.receive_frame(&mut decoded) {
            Ok(()) => {}
            Err(e) if again(e) => return Ok(()),
            Err(e) => return Err(error(e)),
        }
        let size = dimensions(decoded.width(), decoded.height())?;
        let changed = scaler.as_ref().is_none_or(|s| {
            s.input().format != decoded.format()
                || s.input().width != decoded.width()
                || s.input().height != decoded.height()
        });
        if changed {
            *scaler = Some(
                software::scaling::Context::get(
                    decoded.format(),
                    decoded.width(),
                    decoded.height(),
                    ff::format::Pixel::RGBA,
                    decoded.width(),
                    decoded.height(),
                    software::scaling::flag::Flags::BILINEAR,
                )
                .map_err(error)?,
            );
        }
        let scaler = scaler.as_mut().unwrap();
        let space = match decoded.color_space() {
            ff::color::Space::BT709 => ff::ffi::SWS_CS_ITU709,
            ff::color::Space::BT2020NCL | ff::color::Space::BT2020CL => ff::ffi::SWS_CS_BT2020,
            ff::color::Space::Unspecified if decoded.height() >= 720 => ff::ffi::SWS_CS_ITU709,
            _ => ff::ffi::SWS_CS_ITU601,
        };
        unsafe {
            let coefficients = ff::ffi::sws_getCoefficients(space);
            ff::ffi::sws_setColorspaceDetails(
                scaler.as_mut_ptr(),
                coefficients,
                i32::from(decoded.color_range() == ff::color::Range::JPEG),
                coefficients,
                1,
                0,
                1 << 16,
                1 << 16,
            );
        }
        let mut converted = frame::Video::empty();
        scaler.run(&decoded, &mut converted).map_err(error)?;
        let mut rgba = Vec::with_capacity(size);
        for row in converted
            .data(0)
            .chunks(converted.stride(0))
            .take(decoded.height() as usize)
        {
            rgba.extend_from_slice(&row[..decoded.width() as usize * 4]);
        }
        let pts = decoded
            .timestamp()
            .map_or(*next_pts, |pts| pts as f64 * time - origin);
        *next_pts = pts + duration;
        let sar = f64::from(decoded.aspect_ratio());
        let sar = if sar.is_finite() && sar > 0.0 {
            sar as f32
        } else {
            1.0
        };
        output.send(Event::Video(VideoFrame {
            pts,
            duration,
            width: decoded.width(),
            height: decoded.height(),
            aspect: decoded.width() as f32 * sar / decoded.height() as f32,
            rgba,
        }))?;
    }
}

struct Audio {
    index: usize,
    time: f64,
    rate: u32,
    decoder: codec::decoder::Audio,
    resampler: Option<software::resampling::Context>,
    next_pts: f64,
}

impl Audio {
    fn new(index: usize, time: f64, decoder: codec::decoder::Audio) -> Result<Self> {
        let rate = decoder.rate();
        if !(8000..=192000).contains(&rate) {
            return Err(error("Unsupported audio sample rate"));
        }
        Ok(Self {
            index,
            time,
            rate,
            decoder,
            resampler: None,
            next_pts: 0.0,
        })
    }

    fn drain(&mut self, origin: f64, output: &Output) -> Result<()> {
        loop {
            let mut decoded = frame::Audio::empty();
            match self.decoder.receive_frame(&mut decoded) {
                Ok(()) => {}
                Err(e) if again(e) => return Ok(()),
                Err(e) => return Err(error(e)),
            }
            if decoded.rate() != self.rate {
                return Err(error("Audio sample rate changes within the video"));
            }
            if decoded.channel_layout().is_empty() {
                decoded.set_channel_layout(ChannelLayout::default(i32::from(decoded.channels())));
            }
            if self.resampler.is_none() {
                self.resampler = Some(
                    software::resampling::Context::get(
                        decoded.format(),
                        decoded.channel_layout(),
                        decoded.rate(),
                        ff::format::Sample::F32(ff::format::sample::Type::Packed),
                        ChannelLayout::STEREO,
                        self.rate,
                    )
                    .map_err(error)?,
                );
            }
            let resampler = self.resampler.as_mut().unwrap();
            let delay = resampler
                .delay()
                .map_or(0.0, |d| d.output as f64 / self.rate as f64);
            let pts = decoded
                .timestamp()
                .map_or(self.next_pts, |pts| pts as f64 * self.time - origin - delay);
            let mut converted = frame::Audio::empty();
            resampler.run(&decoded, &mut converted).map_err(error)?;
            self.emit(&converted, pts, output)?;
        }
    }

    fn emit(&mut self, converted: &frame::Audio, pts: f64, output: &Output) -> Result<()> {
        if converted.samples() == 0 {
            return Ok(());
        }
        let samples = converted
            .plane::<(f32, f32)>(0)
            .iter()
            .flat_map(|&(l, r)| [l, r])
            .collect();
        self.next_pts = pts + converted.samples() as f64 / self.rate as f64;
        output.send(Event::Audio { pts, samples })
    }

    fn flush(&mut self, output: &Output) -> Result<()> {
        while self.resampler.as_ref().and_then(|r| r.delay()).is_some() {
            let mut converted = frame::Audio::new(
                ff::format::Sample::F32(ff::format::sample::Type::Packed),
                4096,
                ChannelLayout::STEREO,
            );
            self.resampler
                .as_mut()
                .unwrap()
                .flush(&mut converted)
                .map_err(error)?;
            if converted.samples() == 0 {
                break;
            }
            self.emit(&converted, self.next_pts, output)?;
        }
        Ok(())
    }
}

fn check_codec(codec: codec::Id) -> Result<()> {
    if !cfg!(all(
        target_arch = "wasm32",
        any(target_os = "unknown", target_os = "emscripten")
    )) && matches!(codec, codec::Id::H264 | codec::Id::HEVC)
    {
        return Err(error(
            "H.264/H.265 playback is disabled on desktop; use AV1, VP9 or VP8",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_rejects_h264_and_hevc() {
        assert!(check_codec(codec::Id::H264).is_err());
        assert!(check_codec(codec::Id::HEVC).is_err());
        for codec in [
            codec::Id::AV1,
            codec::Id::VP9,
            codec::Id::VP8,
            codec::Id::THEORA,
        ] {
            assert!(check_codec(codec).is_ok());
        }
    }
}
