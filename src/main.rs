#![allow(unused)]

use image;
use image::io::Reader as ImageReader;
use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{stdout, BufWriter, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{mem, os::unix::io::IntoRawFd};

const SKIP: u64 = 1;
// 動画のFPS
const FPS: u64 = 30;
// 画像のパス
const IMAGE_PATH: &str = "/home/maillein/Programs/bad_apple/no_no_no_true";
// 画像の横幅
const WIDTH: u64 = 480;
// 画像の縦幅
const HEIGHT: u64 = 270;
// 画像の横方向の分割数
const DIVISION_W: u64 = WIDTH / 5;
// 画像の縦方向の分割数
const DIVISION_H: u64 = HEIGHT / 10;

pub fn terminal_size() -> Option<winsize> {
    // STDOUT_FILENOか/dev/ttyを利用する
    let fd = if let Ok(file) = File::open("/dev/tty") {
        file.into_raw_fd()
    } else {
        STDOUT_FILENO
    };

    // ファイルディスクリプタに対してTIOCGWINSZをシステムコール
    let mut ws: winsize = unsafe { mem::zeroed() };
    if unsafe { ioctl(fd, TIOCGWINSZ, &mut ws) } == -1 {
        None
    } else {
        Some(ws)
    }
}

pub fn true_color(c: &[u8]) -> String {
    format!("\x1b[38;2;{:>03};{:>03};{:>03}m", c[0], c[1], c[2])
}

fn main() {
    // １フレームあたりの時間（ナノ秒）
    let time_per_flame = Duration::from_nanos(1_000_000_000 / FPS * SKIP);
    let (mut width, mut height) = if let Some(ws) = terminal_size() {
        (ws.ws_col as u64, ws.ws_row as u64)
    } else {
        (DIVISION_W, DIVISION_H)
    };
    if width > height * 32 / 9 {
        width = height * 32 / 9;
    } else {
        height = width * 9 / 32;
    }

    let out = stdout();
    // let mut out = BufWriter::new(out.lock());
    let mut buf_size = (width * 25) as usize * (height + 10) as usize;
    let mut out = BufWriter::with_capacity(buf_size, out.lock());
    let mut files: Vec<_> = {
        fs::read_dir(IMAGE_PATH)
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    };
    files.sort_by_key(|dir| dir.path());

    // １回のループにつき１枚処理する
    for (flame, file) in files.iter().enumerate() {
        if flame as u64 % SKIP > 0 {
            continue;
        }
        let start_time = Instant::now();
        let image = {
            ImageReader::open(file.path().as_path())
                .unwrap()
                .decode()
                .unwrap()
                .resize_exact(
                    width as u32,
                    height as u32,
                    image::imageops::FilterType::Nearest,
                )
                .to_rgb8()
        };
        out.write(b"\x1b[H").unwrap();
        out.write(b"\x1b[40m").unwrap();
        image
            .enumerate_pixels()
            .collect::<Vec<(u32, u32, &image::Rgb<u8>)>>()
            .iter()
            .for_each(|(x, y, &pixel)| {
                out.write_fmt(format_args!("{}@", true_color(&pixel.0)));
                if *x == width as u32 - 1 {
                    out.write(b"\n");
                }
            });

        out.flush().unwrap();

        // ループ開始時からの経過時間
        let passed_time = start_time.elapsed();
        if passed_time < time_per_flame {
            // 時間調整
            sleep(time_per_flame - passed_time);
        }
    }
    out.write(b"\x1b[0m");
    out.flush();
}
