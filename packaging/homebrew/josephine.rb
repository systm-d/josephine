class Josephine < Formula
  desc "Your computer's guardian angel — a local Linux system watcher"
  homepage "https://github.com/systm-d/josephine"
  url "https://github.com/systm-d/josephine/archive/refs/tags/v0.1.0.tar.gz"
  # Placeholder: release.yml replaces this with the real tarball checksum on tag.
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "MIT OR Apache-2.0"
  head "https://github.com/systm-d/josephine.git", branch: "main"

  depends_on "rust" => :build
  depends_on :linux

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "josephine", shell_output("#{bin}/josephine --version")
  end
end
