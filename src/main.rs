use bili_live::{logging, runtime};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about = "B 站推流信息获取工具（RTMP / SRT / WHIP）")]
struct Args {
    #[arg(long, help = "启动时显示完整推流信息；默认隐藏，可在界面按 h 切换")]
    show_full_code: bool,
    #[arg(
        long = "no-request-srt",
        action = clap::ArgAction::SetFalse,
        help = "开播时不索取 SRT 测速节点；默认会索取（服务端仍可能不下发）"
    )]
    request_srt: bool,
}

#[cfg(test)]
mod tests {
    use super::Args;
    use clap::Parser;

    #[test]
    fn srt_is_requested_by_default_and_can_be_disabled() {
        assert!(Args::try_parse_from(["bili_live"]).unwrap().request_srt);
        assert!(
            !Args::try_parse_from(["bili_live", "--no-request-srt"])
                .unwrap()
                .request_srt
        );
    }
}

fn main() -> std::process::ExitCode {
    let args = Args::parse();
    let logs = logging::LogBuffer::new();
    if logging::init(logs.clone()).is_err() {
        eprintln!("无法初始化日志缓冲");
        return std::process::ExitCode::FAILURE;
    }
    match runtime::run(logs, args.show_full_code, args.request_srt) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
