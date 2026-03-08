class Minishelf < Formula
  desc "Rust TUI file explorer with git-aware coloring"
  homepage "https://github.com/yourorg/minishelf"
  version "__VERSION__"

  url "https://github.com/yourorg/minishelf/releases/download/v#{version}/minishelf-#{version}-aarch64-apple-darwin.tar.gz"
  sha256 "__SHA256_AARCH64_APPLE_DARWIN__"

  def install
    bin.install "minishelf"
  end

  test do
    system "#{bin}/minishelf", "--version"
  end
end
