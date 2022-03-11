#!/bin/bash
set -e

[[ $(php -r 'echo PHP_INT_SIZE;') == 8 ]]
