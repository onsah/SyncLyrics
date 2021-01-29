# SyncLyrics

Displays the lyrics of the currently played song on the Spotify. Works for Linux

<p align="center">
    <a href="https://github.com/onsah/SyncLyrics/releases">
        <img alt="Get the .deb" src="assets/deb.svg">
    </a>
</p>

<p float="left" align=center> 
    <img src="assets/screenshot-dark.png" width="45%">
    <img src="assets/screenshot-light.png" width="45%">
</p>

## Features
* Automatically retrieve the lyrics of the current song
* Pin the window so it doesn't fall back of other programs
* Change light/dark color modes

## Installation

### From Binary

Download the `.deb` package from [here](https://github.com/onsah/SyncLyrics/releases)

Run
```
sudo dpkg -i package.deb
```

### From Source

#### Dependenices
Make sure these are installed before proceeding to install

* cargo
* rustc
* meson
* libgtk-3-dev
* libglib2.0-dev
* libglib2.0-dev-bin
* libdbus-1-dev
* libssl-dev
* libcairo2-dev
* libpango1.0-dev
* libatk1.0-dev
* libgdk-pixbuf2.0-dev
* build-essential

Sometimes `cargo` may be out of date therefore build may fail. If you want to get latest `cargo` and `rustc` as debian package you can add the ppa from [here](https://launchpad.net/~ubuntu-mozilla-security/+archive/ubuntu/rust-updates)

If you are on an Ubuntu based distro, you can just run the following 
```
sudo apt install meson build-essential libglib2.0-dev libglib2.0-dev-bin libdbus-1-dev libssl-dev libcairo2-dev libpango1.0-dev libatk1.0-dev libgdk-pixbuf2.0-dev libgtk-3-dev cargo rustc
```

#### Build Instructions

To build manually you need to provide an access token from [here](https://genius.com/api-clients). Then put it in a file named `secret` in the project root.

```
meson build --prefix=/usr
cd build
ninja
```

Run with
```
./com.github.onsah.sync-lyrics
```

To install
```
ninja install
```

## TODO
* Cache the lyrics
* App icon

## Credits

This application uses [Genius](https://docs.genius.com/) api. It is only used for personal interests and I don't have any commercial profit from this application.

This project is inspired from [Lyrics](https://github.com/naaando/lyrics) source code and design.

## License

GPLv3

## Authors
Onur Şahin, sahinonur2000@hotmail.com