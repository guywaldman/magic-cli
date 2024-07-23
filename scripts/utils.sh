#!/bin/bash

if [ "$CI" != "true" ]; then
	bold=$(tput bold)
	normal=$(tput sgr0)
	light_grey=$(tput setaf 250)
	blue=$(tput setaf 4)
	green=$(tput setaf 2)
	yellow=$(tput setaf 3)
	red=$(tput setaf 1)
else
	bold=""
	normal=""
	blue=""
	light_grey=""
	green=""
	yellow=""
	red=""
fi

function info {
	echo "${bold}[INFO] $1 ${normal}"
}

function info-n {
	echo -n "${bold}[INFO] $1 ${normal}"
}

function warn {
	echo "${bold}${yellow}[WARN] $1 ${normal}"
}

function success {
	echo "${bold}${green}[INFO] $1 ${normal}"
}

function error {
	echo >&2 "${red}${bold}[ERROR] $1 ${normal}"
}

function log-to-stderr {
	echo >&2 "${red}${bold}$1 ${normal}"
}

function error-and-exit {
	error "$1"
	if [ "$2" != "" ]; then
		exit $2
	else
		exit 1
	fi
}
