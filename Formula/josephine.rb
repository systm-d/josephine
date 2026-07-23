class Josephine < Formula
  desc "Your computer's guardian angel — a local Linux system watcher"
  homepage "https://github.com/systm-d/josephine"
  url "https://github.com/systm-d/josephine/archive/refs/tags/v0.9.1.tar.gz"
  sha256 "54b9178d9e44f5bc88f6dac582a3dc2a7204b05472bf6d9a6a2eec16daccfce9"
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
    system bin/"josephine", "--version"
  end
end
