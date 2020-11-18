#!/bin/bash
# Sets a groundwork for converting a module JS to TS
# TODO Delete after modules conversion
standardx --fix $1
(printf "import Wikijump from \"@/javascript/Wikijump\";\nimport OZONE from \"@/javascript/OZONE\";\ndeclare const YAHOO: any;\ndeclare const fx: any;\n"; cat $1) > ${1%.js}.ts
$EDITOR -O ${1%.js}.ts $1
