"""Distribution packaging and commit-based release notes; Python standard library only."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tarfile
import tomllib
from urllib.parse import quote
import zipfile


def package(root, target, version, output):
    name = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))["package"]["name"]
    executable = name + (".exe" if "windows" in target else "")
    binary = root / "target" / target / "release" / executable
    if not binary.is_file():
        raise FileNotFoundError(binary)
    output.mkdir(parents=True, exist_ok=True)
    prefix = f"{name}-{version}-{target}"
    files = [(binary, executable), (root / "README.md", "README.md"), (root / "LICENSE.md", "LICENSE.md")]
    archive = output / (prefix + (".zip" if "windows" in target else ".tar.gz"))
    if "windows" in target:
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as bundle:
            for source, filename in files:
                bundle.write(source, f"{prefix}/{filename}")
    else:
        with tarfile.open(archive, "w:gz") as bundle:
            for source, filename in files:
                info = bundle.gettarinfo(str(source), f"{prefix}/{filename}")
                info.mode = 0o755 if source == binary else 0o644
                with source.open("rb") as stream:
                    bundle.addfile(info, stream)
    return archive


def assets(directory, name, version, targets):
    result = []
    for target in targets:
        triple = target["target"]
        extension = ".zip" if "windows" in triple else ".tar.gz"
        path = directory / f"{name}-{version}-{triple}{extension}"
        if not path.is_file():
            raise ValueError(f"Missing release asset for {triple}: {path}")
        result.append(path)
    return result


def checksums(directory, archives):
    lines = []
    for archive in sorted(archives):
        with archive.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        lines.append(f"{digest}  {archive.name}\n")
    (directory / "SHA256SUMS").write_text("".join(lines), encoding="utf-8")


def changes(root, tag, repository):
    def git(*args):
        return subprocess.run(["git", "-C", str(root), *args], capture_output=True, text=True, encoding="utf-8", check=True).stdout.strip()

    # Exclude the current tag, including the first-tag-on-the-root-commit case.
    previous_result = subprocess.run(
        ["git", "-C", str(root), "describe", "--tags", "--abbrev=0", "--match", "v[0-9]*", "--exclude", tag, tag],
        capture_output=True, text=True, encoding="utf-8",
    )
    if previous_result.returncode not in (0, 128):
        previous_result.check_returncode()
    previous = previous_result.stdout.strip() or None
    revision = f"{previous}..{tag}" if previous else tag
    records = git("log", "--reverse", "--no-merges", "--format=%H%x00%s%x00%b%x00", revision).split("\0")
    groups = {title: [] for title in ("不兼容变更", "新功能", "问题修复", "性能优化", "文档", "构建与维护", "其他变更")}
    headings = {"feat": "新功能", "fix": "问题修复", "perf": "性能优化", "docs": "文档", "build": "构建与维护", "ci": "构建与维护", "chore": "构建与维护", "test": "构建与维护", "refactor": "构建与维护", "style": "构建与维护"}
    for index in range(0, len(records) - 2, 3):
        commit, subject, body = (value.strip() for value in records[index:index + 3])
        match = re.match(r"([a-z]+)(?:\(([^)]+)\))?(!)?:\s*(.+)", subject)
        title = headings.get(match[1], "其他变更") if match else "其他变更"
        text = ((f"**{match[2]}**: " if match[2] else "") + match[4]) if match else subject
        if (match and match[3]) or re.search(r"(?m)^BREAKING[ -]CHANGE:", body):
            title = "不兼容变更"
            details = re.search(r"(?m)^BREAKING[ -]CHANGE:\s*(.+)", body)
            if details:
                text += f" — {details[1]}"
        text = text.replace("|", "\\|").replace("\n", " ")
        groups[title].append(f"- {text} ([{commit[:7]}](https://github.com/{repository}/commit/{commit}))")
    sections = [f"### {title}\n" + "\n".join(items) for title, items in groups.items() if items]
    return previous, sections


def notes(root, directory, tag, repository, targets, output):
    name = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))["package"]["name"]
    archives = assets(directory, name, tag, targets)
    checksums(directory, archives)
    previous, sections = changes(root, tag, repository)
    lines = [f"## {tag}", "", "## 更新内容", "", "\n\n".join(sections) or "本次发布没有新增的非合并提交。"]
    if previous:
        lines += ["", f"[完整变更](https://github.com/{repository}/compare/{quote(previous, safe='')}...{quote(tag, safe='')})"]
    lines += ["", "## 下载", "", "Windows 下载 `.zip`，其他平台下载 `.tar.gz`。Linux 不确定 libc 类型或需要在旧发行版运行时，优先尝试 musl 版本。", "", "| 平台 | Rust 目标 | 安装包 | 启动验证 |", "|---|---|---|---|"]
    smoke = {"native": "原生 --version / --help", "qemu": "QEMU --version / --help", "compile-only": "仅交叉编译，未运行"}
    for target, archive in zip(targets, archives):
        triple = target["target"]
        platform = "Windows" if "windows" in triple else "macOS" if "apple" in triple else "FreeBSD" if "freebsd" in triple else "Linux"
        url = f"https://github.com/{repository}/releases/download/{quote(tag, safe='')}/{quote(archive.name, safe='')}"
        lines.append(f"| {platform} | `{triple}` | [{archive.name}]({url}) | {smoke[target['smoke']]} |")
    lines += ["", "## 安装与校验", "", "解压后运行 `bili_live`（Windows 为 `bili_live.exe`）；Unix 安装包保留执行权限。", "", "下载所选安装包和 `SHA256SUMS` 到同一目录，核对文件名及 SHA256：", "", "```sh", "# Linux：只检查已下载的安装包", "sha256sum --ignore-missing -c SHA256SUMS", "# macOS：计算后与 SHA256SUMS 中对应行比较", "shasum -a 256 bili_live-*.tar.gz", "```", "", "```powershell", "# Windows：与 SHA256SUMS 中对应行比较", "Get-FileHash .\\bili_live-*.zip -Algorithm SHA256", "```", "", "GNU 版本依赖构建工具链对应的 glibc；musl 版本静态链接 libc，但仍需系统 CA 证书。FreeBSD 包需匹配系统 ABI。macOS / Windows 包未做代码签名。", "", "启动验证仅检查 CLI 能否启动，不代表扫码登录、剪贴板和所有终端图形协议均已实机验证。", ""]
    output.write_text("\n".join(lines), encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    package_parser = subparsers.add_parser("package")
    package_parser.add_argument("--target", required=True)
    package_parser.add_argument("--version", required=True)
    package_parser.add_argument("--output", type=Path, default=Path("artifacts"))
    notes_parser = subparsers.add_parser("notes")
    notes_parser.add_argument("--tag", required=True)
    notes_parser.add_argument("--repository", required=True)
    notes_parser.add_argument("--artifacts", type=Path, default=Path("artifacts"))
    notes_parser.add_argument("--output", type=Path, default=Path("release-notes.md"))
    args = parser.parse_args()
    root = Path.cwd()
    if args.command == "package":
        print(package(root, args.target, args.version, args.output))
    else:
        targets = json.loads((root / ".github" / "targets.json").read_text(encoding="utf-8"))
        notes(root, args.artifacts, args.tag, args.repository, targets, args.output)


if __name__ == "__main__":
    main()
