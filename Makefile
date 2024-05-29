build:
	cross build --target x86_64-unknown-linux-gnu

release:
	cross build --target x86_64-unknown-linux-gnu --release
