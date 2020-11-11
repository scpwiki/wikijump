standardx --fix $1
(printf "import Wikijump from \"@/javascript/Wikijump\";\nimport OZONE from \"@/javascript/OZONE\";\ndeclare const YAHOO: any;\ndeclare const fx: any;\n"; cat $1) > ${1%.js}.ts
nvim -O ${1%.js}.ts $1
