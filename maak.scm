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
  (list "RUST_LOG=\"debug\"" "RUSTFLAGS=\"-C debuginfo=2 -C opt-level=1\""))

(define guix-time-machine-shell
  (list guix-time-machine "--" "shell -m" guix-manifest-file "--"))

(define guix-time-machine-program-shell
  (list guix-time-machine "--" "shell -f" guix-package-file "--"))

(define guix-derivation-file
  ".inkaartbrenger.drv")

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
  ($ (list (~ "rm -rfv ~a" guix-derivation-file)))
  ($ (append rust-build-vars
             (list guix-time-machine "--" "build -f" guix-package-file
                   (~ "-d -v 5 > ~a" guix-derivation-file)))

     #:verbose? #t)
  ($ (list (~ "cat ~a" guix-derivation-file))))

(define (guix-install-to-local-bin)
  (guix-package-build)
  ($ (list "sudo cp -rv"
           (~ "\"$(cat ~a)/bin/inkaartbrenger\"" guix-derivation-file)
           "~/.local/bin/")
     #:verbose? #t))

(define (verify)
  (fmt)
  (guix-package-build)
  (cargo-jointhefreeworld)
  (guix-jointhefreeworld)
  (guix-install-to-local-bin))

;; guix-install:
;;     sudo cp "$(guix time-machine --channels=channels.scm -- build -f guix.scm -q)/bin/inkaartbrenger" ~/.local/bin/

