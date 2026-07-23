class Josephine < Formula
  desc "Your computer's guardian angel — a local Linux system watcher"
  homepage "https://github.com/systm-d/josephine"
  url "https://github.com/systm-d/josephine/archive/refs/tags/v0.9.0.tar.gz"
  sha256 "eee82ee61a2e13a46fdbb7d9c7fd621a1a1bf93ba40053457af1baa3c3b91a6a"
  license "MIT OR Apache-2.0"
  head "https://github.com/systm-d/josephine.git", branch: "main"

  depends_on "rust" => :build
  # Joséphine is Linux-native (systemd, /sys/class/thermal, libnotify).
  depends_on :linux

  def install
    # Root Cargo.toml is a virtual workspace, so point cargo at the binary crate.
    system "cargo", "install", *std_cargo_args(path: "crates/josephine")
  end

  test do
    assert_match "josephine", shell_output("#{bin}/josephine --version")
  end
end
