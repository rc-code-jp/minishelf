class Pocoshelf < Formula
  desc "Rust TUI file explorer with git-aware coloring"
  homepage "https://github.com/rc-code-jp/pocoshelf"
  version "__VERSION__"

  url "https://github.com/rc-code-jp/pocoshelf/releases/download/v#{version}/pocoshelf-#{version}-aarch64-apple-darwin.tar.gz"
  sha256 "__SHA256_AARCH64_APPLE_DARWIN__"

  def install
    bin.install "pocoshelf"
  end

  test do
    system "#{bin}/pocoshelf", "--version"
  end
end
