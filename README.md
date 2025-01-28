This is a simple simulation of the solar system, written in Rust. It uses Kepler's laws of planetary motion to compute past and future positions of objects from their orbital parameters as of January 1, 2000 (J2000). It does not account for perturbations and so will lose accuracy as it gets farther from J2000. It includes the planets, the Sun, and the Moon, and can calculate their zodiac signs based on their relative positions to the Earth.

The CLI (can be run with `cargo run` to compile, or `solar_system.exe` once compiled) has three subcommands:

`plot` creates a PNG graph of the solar system, showing orbits and locations at the specified time.
Times can be specified by "now", YYYY-MM-DD, or ISO 8601, RFC 2822, or RFC 3339 formats.
They can be graphed relative to the Sun (abs) or relative to the Earth (rel), and use 2 or 3 dimensions.
```
Usage: solar_system.exe plot [OPTIONS]

Options:
  -p, --pixels <PIXELS>  [default: 2048]
  -s, --scale <SCALE>    [default: 200.0]
  -t, --time <TIME>      [default: now]
  -m, --mode <MODE>      [default: abs2d] [possible values: abs2d, rel2d, abs3d, rel3d]
  -h, --help             Print help
```

`sign` calculates the zodiac signs of some or all solar system objects at a chosen time. 
Times can be specified by "now", YYYY-MM-DD, or ISO 8601, RFC 2822, or RFC 3339 formats.
If no value is set for planets, it will show all of them. Multiple planets can be entered with comma separation.
```
Usage: solar_system.exe sign [OPTIONS]

Options:
  -p, --planets <PLANETS>...  [possible values: sun, mercury, venus, moon, mars, jupiter, saturn, uranus, neptune]
  -t, --time <TIME>           [default: now]
  -h, --help                  Print help
```

`next` calculates the next time that objects will be in a certain sign, from a chosen start time. 
Times can be specified by "now", YYYY-MM-DD, or ISO 8601, RFC 2822, or RFC 3339 formats.
If no value is set for planets, it will show all of them. Multiple planets can be entered with comma separation.
```
Usage: solar_system.exe next [OPTIONS] <SIGN>

Arguments:
  <SIGN>  [possible values: aries, taurus, gemini, cancer, leo, virgo, libra, scorpio, sagittarius, capricorn, aquarius, pisces]

Options:
  -p, --planets <PLANETS>...  [possible values: sun, mercury, venus, moon, mars, jupiter, saturn, uranus, neptune]
  -t, --time <TIME>           [default: now]
  -h, --help                  Print help
```