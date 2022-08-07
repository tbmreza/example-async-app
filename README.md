# example-async-app

[<img alt="github" src="https://img.shields.io/badge/github-tbmreza/example%E2%80%93async%E2%80%93app-blue?style=for-the-badge&logo=github" height="20">](https://github.com/tbmreza/example-async-app)

This is a REPL program for interacting with ChromeDriver.

The program intends to replace graphical browsers from a web developer's debugging workflow. When
hunting for a JavaScript bug, a common flow is to modify the source code adding log statements,
and then switch to browser to view its console. At developer's preference, tasks where a headless
Chrome would suffice can be performed without context switching to another GUI.

This repo mainly serves as an example of a Selenium WebDriver client ([thirtyfour]), an async
([tokio]), and in general a REPL ([rustyline]) app.

[thirtyfour]: https://github.com/stevepryde/thirtyfour
[tokio]: https://github.com/tokio-rs/tokio
[rustyline]: https://github.com/kkawakam/rustyline

<br>

## Requirements

- `chromedriver` — Download latest from [chromium website](https://chromedriver.chromium.org/home).

[thirtyfour] does support Firefox, which means we can also use geckodriver (which runs marionette).
However, marionette doesn't support non-W3C method `get_log`. Capturing browser log requires a
[geckodriver specific config], which isn't implemented yet.

[geckodriver specific config]: https://github.com/mozilla/geckodriver/issues/284#issuecomment-477677764

<br>

## Testing

The test suite depends on chromedriver already running.

```sh
# cat test.sh
if [ $(pgrep -f chromedriver) ]
then
	cargo test
else
	chromedriver --port=4444 & cargo test
fi
```

`cargo test` intentionally doesn't stop chromedriver process on completion (because restarting the program takes considerable seconds).
Manually kill it using `pkill -f chromedriver`, for example.

### Common Errors

#### "This version of ChromeDriver only supports Chrome version xx"

If chrome updates automatically but chromedriver doesn't, this error will show up quite often.
Either replace chromedriver binary manually from https://chromedriver.chromium.org/downloads,
or invoke your package manager's upgrade command on chromedriver.
```sh
nix-env -iA nixpkgs.chromedriver 
```
<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
