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


(use-modules (guix gexp)
             (guix packages)
             (guix import crate)
             (guix git-download)
             ((guix licenses)
              #:prefix license:)
             (guix build-system cargo)
             (gnu packages)
             (gnu packages tls)
             (gnu packages pkg-config))

(define %source-dir
  (dirname (current-filename)))

(define-public inkaartbrenger
  (package
    (name "inkaartbrenger")
    (version "dev")
    (source
     (local-file %source-dir
                 #:recursive? #t
                 #:select? (git-predicate %source-dir)))
    (build-system cargo-build-system)
    (native-inputs (list pkg-config))
    (inputs (cons* openssl (cargo-inputs-from-lockfile "Cargo.lock")))
    (arguments
      `(#:install-source? #f))
    (home-page "https://codeberg.org/jjba23/inkaartbrenger")
    (synopsis "")
    (description "")
    (license license:agpl3+)))

inkaartbrenger
