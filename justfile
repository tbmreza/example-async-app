alias sc := spin-chrome
alias sg := spin-gecko

# run repl
run:
	cargo +nightly r --bin repl

# run chromedriver on port 4444
spin-chrome:
	chromedriver --port=4444

# run geckodriver on port 4444
spin-gecko:
	geckodriver --port 4444
