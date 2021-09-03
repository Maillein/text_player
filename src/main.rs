#![allow(unused)]

use image;
use image::imageops::FilterType;
use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{fs::File, mem, os::unix::io::IntoRawFd};

use image::io::Reader as ImageReader;

// 動画のFPS
const FPS: u64 = 30;
// 画像のパス
const IMAGE_PATH: &str = "/home/maillein/Programs/bad_apple/images";
// 画像の横幅
const WIDTH: u64 = 480;
// 画像の縦幅
const HEIGHT: u64 = 360;
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

fn main() {
    // １フレームあたりの時間（ナノ秒）
    let time_per_flame = Duration::from_nanos(1_000_000_000 / FPS);
    let (mut width, mut height) = if let Some(ws) = terminal_size() {
        (ws.ws_col as u64, ws.ws_row as u64)
    } else {
        (DIVISION_W, DIVISION_H)
    };
    if width > height * 8 / 3 {
        width = height * 8 / 3;
    } else {
        height = width * 3 / 8;
    }

    let out = stdout();
    let mut out = out.lock();

    // １回のループにつき１枚処理する
    'running: for image_number in 1..=6572 {
        let start_time = Instant::now();
        //         Command::new("clear").spawn().unwrap();
        let mut colors = vec![vec![0u32; width as usize]; height as usize];
        let mut bufs = vec![String::with_capacity(width as usize); height as usize];
        let image = {
            ImageReader::open(format!("{}/{:>04}.png", IMAGE_PATH, image_number).as_str())
                .unwrap()
                .decode()
                .unwrap()
                .resize_exact(width as u32, height as u32, FilterType::Nearest)
                .to_luma8()
        };
        image
            .enumerate_pixels()
            .collect::<Vec<(u32, u32, &image::Luma<u8>)>>()
            .iter()
            .for_each(|(x, y, pixel)| {
                colors[*y as usize][*x as usize] += (pixel.0)[0] as u32;
            });

        write!(out, "\x1b[2J");
        // write!(out, "\x1b[{}F", height).unwrap();
        for h in 0..height as usize {
            for w in 0..width as usize {
                if colors[h][w] > 128 {
                    bufs[h] += "@";
                } else {
                    bufs[h] += " ";
                }
            }
        }
        for s in bufs {
            write!(out, "{}\n", s).unwrap();
        }

        // ループ開始時からの経過時間
        let passed_time = start_time.elapsed();
        if passed_time < time_per_flame {
            // 時間調整
            sleep(time_per_flame - passed_time);
        }
    }
}
