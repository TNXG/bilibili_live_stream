import importlib.util
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
import zipfile

SPEC = importlib.util.spec_from_file_location("release", Path(__file__).with_name("release.py"))
release = importlib.util.module_from_spec(SPEC)


class ReleaseTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        SPEC.loader.exec_module(release)

    def test_archives_preserve_executable_and_include_only_distribution_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text('[package]\nname = "bili_live"\n', encoding="utf-8")
            for filename in ("README.md", "LICENSE.md", "cookies.json"):
                (root / filename).write_text(filename, encoding="utf-8")
            for target, extension in [("x86_64-unknown-linux-musl", ""), ("aarch64-pc-windows-msvc", ".exe")]:
                binary = root / "target" / target / "release" / ("bili_live" + extension)
                binary.parent.mkdir(parents=True)
                binary.write_bytes(b"distribution binary")
                archive = release.package(root, target, "v1.2.3", root / "artifacts")
                prefix = f"bili_live-v1.2.3-{target}"
                expected = {f"{prefix}/bili_live{extension}", f"{prefix}/README.md", f"{prefix}/LICENSE.md"}
                if extension:
                    with zipfile.ZipFile(archive) as bundle:
                        self.assertEqual(set(bundle.namelist()), expected)
                        self.assertEqual(bundle.read(f"{prefix}/bili_live.exe"), b"distribution binary")
                else:
                    with tarfile.open(archive) as bundle:
                        self.assertEqual(set(bundle.getnames()), expected)
                        self.assertEqual(bundle.getmember(f"{prefix}/bili_live").mode & 0o111, 0o111)
                        self.assertEqual(bundle.extractfile(f"{prefix}/bili_live").read(), b"distribution binary")

    def test_missing_target_prevents_incomplete_release(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "bili_live-v1.0.0-x86_64-unknown-linux-gnu.tar.gz").write_bytes(b"one target")
            targets = [{"target": "x86_64-unknown-linux-gnu", "smoke": "native"}, {"target": "aarch64-unknown-linux-musl", "smoke": "qemu"}]
            with self.assertRaisesRegex(ValueError, "aarch64-unknown-linux-musl"):
                release.assets(root, "bili_live", "v1.0.0", targets)

    def test_checksums_hash_archives_without_hashing_previous_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "binary.zip").write_bytes(b"abc")
            (root / "SHA256SUMS").write_text("old checksum", encoding="utf-8")
            release.checksums(root, [root / "binary.zip"])
            self.assertEqual((root / "SHA256SUMS").read_text(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  binary.zip\n")

    def test_changelog_uses_applicable_previous_tag_and_records_breaking_changes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def git(*args):
                return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()
            git("init", "-q")
            git("config", "user.name", "Release Test")
            git("config", "user.email", "release@example.invalid")
            def commit(subject, body=None):
                args = ["commit", "--allow-empty", "-qm", subject]
                if body:
                    args += ["-m", body]
                git(*args)
            commit("feat: old feature")
            git("tag", "v1.0.0")
            commit("feat(ui): 新增选择界面")
            commit("fix!: 变更配置格式", "BREAKING CHANGE: 旧配置需要迁移")
            commit("docs: 使用说明 | 更新")
            git("tag", "v1.1.0-rc.1")
            previous, sections = release.changes(root, "v1.1.0-rc.1", "owner/repo")
            text = "\n".join(sections)
            self.assertEqual(previous, "v1.0.0")
            self.assertIn("新增选择界面", text)
            self.assertIn("旧配置需要迁移", text)
            self.assertNotIn("old feature", text)
            self.assertIn("https://github.com/owner/repo/commit/", text)
            git("tag", "v1.1.0")
            previous_stable, stable_sections = release.changes(root, "v1.1.0", "owner/repo")
            self.assertEqual(previous_stable, "v1.0.0")
            self.assertIn("新增选择界面", "\n".join(stable_sections))
            self.assertIn("旧配置需要迁移", "\n".join(stable_sections))

    def test_first_release_includes_history_and_no_compare_link(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for args in [("init", "-q"), ("config", "user.name", "Release Test"), ("config", "user.email", "release@example.invalid"), ("commit", "--allow-empty", "-qm", "Initial release"), ("tag", "v0.1.0")]:
                subprocess.run(["git", "-C", str(root), *args], check=True)
            previous, sections = release.changes(root, "v0.1.0", "owner/repo")
            self.assertIsNone(previous)
            self.assertIn("Initial release", "\n".join(sections))


if __name__ == "__main__":
    unittest.main()
