.PHONY: dev install install-speech

dev:
	sh scripts/check.sh

install:
	sh scripts/install-macos.sh

install-speech:
	sh scripts/install-macos.sh with-speech
