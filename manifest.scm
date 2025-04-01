;;; SPDX-FileCopyrightText: 2025 Pier-Hugues Pellerin <ph@heykimo.com>
;;;
;;; SPDX-License-Identifier: MIT

(use-modules (guix)
	     (guix packages)
	     (gnu packages freedesktop)
	     (gnu packages wm)
	     (gnu packages xdisorg)
	     (gnu packages pkg-config)
	     (gnu packages xorg)
	     (rusty rust))

(packages->manifest (list
		     rust-next
		     rust-analyzer-next
		     `(,rust-next "cargo")
		     ;; `(,rust-next "rust-src")
		     ;; `(,rust-next "tools")
		     wayland-protocols
		     pkg-config
		     wayland
		     wlroots
		     libinput-minimal
		     libxkbcommon
		     libevdev))
