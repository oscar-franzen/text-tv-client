## An extremely simple teletext client
An extremely simple Swedish Television Text TV Client [1,2] for the
Linux command line. Written as a toy project to learn Rust. The actual
protocol is https since SVT's teletext [3] can also be accessed online.

## Compile instructions
A Linux 64 bit executable binary is also provided in the repo root.

```bash
git clone https://github.com/oscar-franzen/text-tv-client

cd text-tv-client

cargo r --release
```

## Usage
Just run the executable without arguments:

```bash
./text_tv_cli
```

The `-u` flag can be used to change the user agent, for example:

```bash
./text_tv_cli -u "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:53.0) Gecko/20100101 Firefox/53.0"
```

## Feedback:
- OF; <p.oscar.franzen@gmail.com>

## References
1. https://sv.wikipedia.org/wiki/Text-TV
2. https://en.wikipedia.org/wiki/Teletext
3. https://www.svt.se/svttext/web/pages/100.html
