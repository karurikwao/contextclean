# Homebrew Formula Notes

ContextClean ships release archives for macOS Intel and Apple Silicon:

- `contextclean-v0.1.0-x86_64-apple-darwin.tar.gz`
- `contextclean-v0.1.0-aarch64-apple-darwin.tar.gz`

The archives contain:

- `ctxclean`
- `ctxrun`
- `README.md`
- `LICENSE`
- `CHANGELOG.md`

## Direct Install Until A Tap Exists

Until `karurikwao/homebrew-contextclean` is published, users can install from the release archives directly:

```bash
curl -LO https://github.com/karurikwao/contextclean/releases/download/v0.1.0/contextclean-v0.1.0-aarch64-apple-darwin.tar.gz
tar -xzf contextclean-v0.1.0-aarch64-apple-darwin.tar.gz
install -m 0755 contextclean-v0.1.0-aarch64-apple-darwin/ctxclean /usr/local/bin/ctxclean
install -m 0755 contextclean-v0.1.0-aarch64-apple-darwin/ctxrun /usr/local/bin/ctxrun
```

Planned tap install command after `homebrew-contextclean` exists:

```bash
brew tap karurikwao/contextclean
brew install contextclean
```

Planned tap upgrade command:

```bash
brew update
brew upgrade contextclean
```

## Formula Template

```ruby
class Contextclean < Formula
  desc "Local-first context cleaner for AI agents"
  homepage "https://contextclean.pages.dev/"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/karurikwao/contextclean/releases/download/v0.1.0/contextclean-v0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "<aarch64-apple-darwin sha256>"
    else
      url "https://github.com/karurikwao/contextclean/releases/download/v0.1.0/contextclean-v0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "<x86_64-apple-darwin sha256>"
    end
  end

  def install
    bin.install "ctxclean"
    bin.install "ctxrun"
    doc.install "README.md"
    doc.install "CHANGELOG.md"
  end

  test do
    assert_match "ctxclean 0.1.0", shell_output("#{bin}/ctxclean --version")
    assert_match "ctxrun 0.1.0", shell_output("#{bin}/ctxrun --version")
  end
end
```

## Checksum Flow

Download the release checksum files:

```bash
curl -LO https://github.com/karurikwao/contextclean/releases/download/v0.1.0/contextclean-v0.1.0-aarch64-apple-darwin.tar.gz.sha256
curl -LO https://github.com/karurikwao/contextclean/releases/download/v0.1.0/contextclean-v0.1.0-x86_64-apple-darwin.tar.gz.sha256
```

Copy the first field from each `.sha256` file into the formula. Verify locally:

```bash
shasum -a 256 contextclean-v0.1.0-aarch64-apple-darwin.tar.gz
shasum -a 256 contextclean-v0.1.0-x86_64-apple-darwin.tar.gz
```

## Tap Release Flow

1. Create `homebrew-contextclean`.
2. Add `Formula/contextclean.rb` from the template.
3. Replace checksum placeholders with values from the GitHub release.
4. Run `brew audit --strict --online contextclean`.
5. Run `brew install --build-from-source contextclean`.
6. Tag the tap update when a ContextClean release is promoted.

## Upgrade Flow

For each ContextClean release:

1. Update `version`.
2. Update both release URLs.
3. Update both SHA256 values.
4. Run the formula tests on Intel and Apple Silicon runners if available.
5. Publish the tap commit.
