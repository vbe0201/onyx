default: run

default_target := "riscv64_qemu"
default_package := "onyx"

run target=default_target:
    cargo xtask run -c {{target}}

build target=default_target package=default_package:
    cargo xtask build -c {{target}} {{package}}

check target=default_target package=default_package:
    cargo xtask check -c {{target}} {{package}}

dist target=default_target:
    cargo xtask dist -c {{target}} --release
