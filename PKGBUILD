# Maintainer: Stepan Manookian <stepan@manookian.de>
pkgname=markinator
pkgver=0.1.0
pkgrel=1
pkgdesc="Markdown editor and reader for Omarchy"
arch=('x86_64')
url="https://github.com/smanookian/markinator"
license=('MIT')
depends=('webkit2gtk-4.1' 'gtk3' 'fontconfig')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('c48a610b04e143a768bc8266dd0b80525363b13eff2d6e762c61c2c7afca117b')

build() {
  cd "$srcdir/$pkgname-$pkgver/src-tauri"
  cargo build --release --locked
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  install -Dm755 src-tauri/target/release/markinator "$pkgdir/usr/bin/markinator"
  install -Dm644 markinator.desktop "$pkgdir/usr/share/applications/markinator.desktop"
  install -Dm644 icons/markinator.svg "$pkgdir/usr/share/icons/hicolor/scalable/apps/markinator.svg"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/markinator/LICENSE"
}
