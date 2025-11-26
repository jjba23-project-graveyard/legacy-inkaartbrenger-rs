;;; inkaartbrenger

;; Copyright © Josep Jesus Bigorra Algaba

;; inkaartbrenger is free software: you can redistribute it and/or modify
;; it under the terms of the GNU Affero General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.

;; inkaartbrenger is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; GNU Affero General Public License for more details.

;; You should have received a copy of the GNU Affero General Public License
;; along with inkaartbrenger. If not, see <https://www.gnu.org/licenses/>.

(use-modules (guix packages)
             (gnu)
             (gnu packages rust)
             (gnu packages gcc)             
             (gnu packages tls)
             (gnu packages pkg-config))

(packages->manifest (list openssl
                          pkg-config
                          rust
                          gcc
                          (list rust "cargo")
                          (list rust "tools")
                          rust-analyzer))
