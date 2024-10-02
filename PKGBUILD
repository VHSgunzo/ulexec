# Maintainer: VHSgunzo <vhsgunzo.github.io>
pkgname='ulexec-bin'
pkgver='0.0.1'
pkgrel='1'
pkgdesc='A tool for loading and executing PE on Windows and ELF on Linux from memory'
arch=("x86_64")
url='https://github.com/VHSgunzo/ulexec'
provides=("${pkgname}")
conflicts=("${pkgname}")
source=(
    "${pkgname}::https://github.com/VHSgunzo/${pkgname}/releases/download/v${pkgver}/${pkgname}"
    "LICENSE::https://raw.githubusercontent.com/VHSgunzo/${pkgname}/refs/heads/main/LICENSE"
)
sha256sums=('SKIP' 'SKIP')

package() {
    install -Dm755 ${pkgname} "$pkgdir/usr/bin/${pkgname}"
    install -Dm755 LICENSE "$pkgdir/usr/share/licenses/ulexec/LICENSE"
}
