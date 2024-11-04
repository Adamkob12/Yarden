use std::process::Command;

pub fn get_fps(file_name: &'static str) -> usize {
    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=avg_frame_rate",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            file_name,
        ])
        .output()
        .unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    let output = output.trim();
    eprintln!("{}", output);
    assert!(output.ends_with("/1"));
    output
        .split('/')
        .into_iter()
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

pub fn get_width(file_name: &'static str) -> u32 {
    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width",
            "-of",
            "csv=s=x:p=0",
            file_name,
        ])
        .output()
        .unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    eprintln!("{}", output);
    output.trim().parse().unwrap()
}

pub fn get_height(file_name: &'static str) -> u32 {
    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=height",
            "-of",
            "csv=s=x:p=0",
            file_name,
        ])
        .output()
        .unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    eprintln!("{}", output);
    output.trim().parse().unwrap()
}

pub fn get_frame_count(file_name: &'static str) -> usize {
    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-count_frames",
            "-show_entries",
            "stream=nb_read_frames",
            "-of",
            "default=nokey=1:noprint_wrappers=1",
            file_name,
        ])
        .output()
        .unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    eprintln!("{}", output);
    output.trim().parse().unwrap()
}

#[test]
fn test_get_video_metadata() {
    assert_eq!(25, get_fps("client/assets/test1.mp4".into()));
    assert_eq!(1920, get_width("client/assets/test1.mp4".into()));
    assert_eq!(1080, get_height("client/assets/test1.mp4".into()));
    // assert_eq!(6817, get_frame_count("assets/test1.mp4".into()));
}
