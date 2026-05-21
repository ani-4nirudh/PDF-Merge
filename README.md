# Introduction
This is an interactive CLI PDF merge utility. 
![Demo Preview](assets/screenrecording.gif)

# Run tool locally
```
# Clone the repo
cargo build --release
cargo run
```

# Install from `apt`
```
sudo add-apt-repository ppa:ani-4nirudh/pdfmerge
sudo apt update
sudo apt install pdfmerge
``` 

# Build locally
```
# Make desired changes
cargo build --release
cargo-deb
sudo apt install target/debian/pdfmerge*.deb
``` 

# Uninstall 
```
sudo apt remove pdfmerge
```
