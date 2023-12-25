build:
	cross build --target x86_64-unknown-linux-gnu --features firewall

release:
	cross build --target x86_64-unknown-linux-gnu --release --features firewall
