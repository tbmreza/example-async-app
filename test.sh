if [ $(pgrep -f chromedriver) ]
then
	cargo test
else
	chromedriver --port=4444 & cargo test
fi
