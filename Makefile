build:
	cross build --target x86_64-unknown-linux-gnu --all-features

release:
	cross build --target x86_64-unknown-linux-gnu --release --all-features
