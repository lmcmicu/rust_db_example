MAKEFLAGS += --warn-undefined-variables
SHELL := bash
.SHELLFLAGS := -eu -o pipefail -c
.DEFAULT_GOAL := run
.DELETE_ON_ERROR:
.SUFFIXES:

.PHONY: run
run:
	cargo $@ sqlite
	cargo $@ postgres
