.PHONY: dev install install-speech stand clean-target

DAYS ?= 7

dev:
	sh scripts/check.sh

stand:
	sh scripts/stand.sh

clean-target:
	sh scripts/clean-target.sh $(DAYS)

install:
	sh scripts/install-macos.sh

install-speech:
	sh scripts/install-macos.sh with-speech
