.PHONY: dev install install-speech stand

dev:
	sh scripts/check.sh

stand:
	sh scripts/stand.sh

install:
	sh scripts/install-macos.sh

install-speech:
	sh scripts/install-macos.sh with-speech
