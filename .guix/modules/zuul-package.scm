(define-module (zuul-package)
  #:use-module ((guix licenses) #:prefix license:)
  #:use-module (gnu packages freedesktop)
  #:use-module (gnu packages pkg-config)
  #:use-module (gnu packages rust-apps)
  #:use-module (gnu packages xdisorg)
  #:use-module (gnu packages xorg)
  #:use-module (guix build-system cargo)
  #:use-module (guix git)
  #:use-module (guix git-download)
  #:use-module (guix packages)
  #:use-module (guix)
  #:use-module (rusty rust))

(define source-checkout
  (let ((vcs-file? (or (git-predicate
                        (string-append (current-source-directory)
                                       "/../.."))
                       (const #t))))
    (local-file "../.." "zuul-checkout"
                #:recursive? #t
                #:select? vcs-file?)))

(define %version "0.1.0")

(define-public zuul
  (package
   (version (string-append %version "-git"))
   (name "zuul")
   (source source-checkout)
   (build-system cargo-build-system)
   (native-inputs
    (list rust-next))
   (synopsis "a pinentry software for wayland written in rust and iced")
   (description "a pinentry software for wayland written in rust and iced")
   (home-page "https://github.com/ph/zuul")
   (license license:expat)))

zuul
