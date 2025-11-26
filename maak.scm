;;; inkaartbrenger

;; Copyright © Josep Bigorra <jjbigorra@gmail.com>

;; inkaartbrenger is free software: you can redistribute it and/or modify
;; it under the terms of the GNU Affero General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.

;; inkaartbrenger is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; GNU Affero General Public License for more details.

;; You should have received a copy of the GNU Affero General Public License
;; along with inkaartbrenger.  If not, see <https://www.gnu.org/licenses/>.

(define-module (maak)
  #:declarative? #t
  #:use-module (maak maak))

(define guix-time-machine
  "guix time-machine --channels=channels.scm")

(define guix-manifest-file
  "manifest.scm")

(define guix-package-file
  "guix.scm")

(define rust-build-vars
  (list "RUST_LOG=\"debug\""))

(define guix-time-machine-shell
  (list guix-time-machine "--" "shell -m" guix-manifest-file "--"))

(define guix-time-machine-program-shell
  (list guix-time-machine "--" "shell -f" guix-package-file "--"))

(define (fmt)
  ($ (append rust-build-vars guix-time-machine-shell
             (list "cargo fmt --verbose --all"))
     #:verbose? #t))

(define (cargo-jointhefreeworld)
  ($ (append rust-build-vars guix-time-machine-shell
             (list "cargo run -- --url" "https://jointhefreeworld.org"))
     #:verbose? #t))

(define (guix-jointhefreeworld)
  ($ (append rust-build-vars guix-time-machine-program-shell
             (list "inkaartbrenger --url" "https://jointhefreeworld.org"))
     #:verbose? #t))

(define (guix-package-build)
  ($ (append rust-build-vars
             (list guix-time-machine "--" "build -f" guix-package-file "-d"))
     #:verbose? #t))

(define (cargo-release)
  ($ (append rust-build-vars guix-time-machine-shell
             (list "cargo build --profile=release"))
     #:verbose? #t))

(define (cargo-install)
  (cargo-release)
  ($ '("sudo cp -rv ./target/release/inkaartbrenger" "~/.local/bin/")
     #:verbose? #t))

(define (verify)
  (fmt)
  (guix-package-build)
  (cargo-jointhefreeworld)
  (guix-jointhefreeworld)
  (cargo-release)
  (cargo-install))
