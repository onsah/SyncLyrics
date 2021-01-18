# SyncLyrics

A Gtk application to display lyrics of the song on the Spotify

![](assets/screenshot.png)

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