class Minishelf < Formula
  desc "Rust TUI file explorer with git-aware coloring"
  homepage "https://github.com/rc-code-jp/minishelf"
  version "0.1.10"

  on_macos do
    url "https://github.com/rc-code-jp/minishelf/releases/download/v#{version}/minishelf-#{version}-macos-aarch64.tar.gz"
    sha256 "e7d33544424ff0920bebb09f01f5120ef664eff76fd02cb7c268005141b73048"
  end

  on_linux do
    url "https://github.com/rc-code-jp/minishelf/releases/download/v#{version}/minishelf-#{version}-linux-x86_64.tar.gz"
    sha256 "7fcaab40f473135c2ce8f37c8711b6b8fa82edb1afa3847baa92e42890a6ddba"
  end

  def install
    if OS.mac? && Hardware::CPU.intel?
      odie "Intel macOS binary is not published yet. Please use Apple Silicon or build from source."
    end

    bin.install "minishelf"
  end

  test do
    system "#{bin}/minishelf", "--version"
  end
end
