# SyncLyrics

Displays the lyrics of the currently played song on the Spotify. Works for Linux

<p align=center> 
    <img src="assets/screenshot.png">
</p>

## Features
* Automatically retrieve the lyrics of the current song
* Pin the window so it doesn't fall back of other programs
* Change light/dark color modes

## Installation

### Dependenices
Make sure these are installed before proceeding to install

* cargo
* meson
* gtk
* glib

### Building

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

In the first opening, you need to provide an api key from [here](https://happi.dev/)

## TODO
* Cache the lyrics
* Being able to change the api key later
* Add app launcher shortcut
* Create Icon

## Credits

This application uses [happi](https://happi.dev/) api. It is only used for personal interests and I don't have any commercial profit from this application.

This project is inspired from [Lyrics](https://github.com/naaando/lyrics) source code and design.

## License

GPLv3

## Authors
Onur Şahin, sahinonur2000@hotmail.com