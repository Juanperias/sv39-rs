run: build
	qemu-system-riscv64 \
		-machine virt \
		-cpu rv64 \
		-bios default \
		-smp 1 \
		-m 128M \
		-nographic \
		-serial mon:stdio \
		--no-reboot \
		-kernel target/riscv64gc-unknown-none-elf/debug/dummy-kernel

debug: build
	qemu-system-riscv64 \
		-machine virt \
		-cpu rv64 \
		-bios default \
		-smp 1 \
		-m 128M \
		-nographic \
		-serial mon:stdio \
		--no-reboot \
		-S -s \
		-kernel target/riscv64gc-unknown-none-elf/debug/dummy-kernel


build:
	RUSTFLAGS="-C link-arg=-Tkernel.ld -C linker=rust-lld" cargo build -p dummy-kernel
