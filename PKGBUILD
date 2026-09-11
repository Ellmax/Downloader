pkgname=dlr
pkgver=0.1.0
pkgrel=1
pkgdesc="Downloader"
arch=('x86_64')
url=https://github.com/Ellmax/Downloader
license=('GPL-3.0-only')
makedepends=('cargo' 'rust')
depends=('glibc' 'libgcc' 'openssl')

build() {
  export CARGO_HOME="$startdir/cargo-home"
  cargo build --release --locked --verbose
}

package() {
  install -Dm755 "$startdir/target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
}