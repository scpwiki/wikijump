#!/bin/bash
# Sets a groundwork for converting a module JS to TS
# TODO Delete after modules conversion

set -e

standardx --fix $1 || true

(printf "import Wikijump from \"@/javascript/Wikijump\";\nimport OZONE from \"@/javascript/OZONE\";\nimport { RequestModuleParameters } from \"@/javascript/OZONE/ajax\";\ndeclare const YAHOO: any;\ndeclare type YahooResponse = any;\ndeclare const fx: any;\n"; cat $1) > ${1%.js}.ts

$EDITOR -s /dev/stdin <<<"
:/Wikijump\\.modules\\./s!!//@ts-expect-error\rexport const !
:%s/Wikijump\\.modules\\.${1%.js}/${1%.js}/g
:%s/\\_s*};\\_s*${1%.js}\\.\\(\\S\\+\\) =/\r},\r  \1:/
:/export const/s/{\n},/{/
:%s/\$(\\([^)]\\+\\))/document.getElementById(\1)!/g
:%s/\\<var\\>/let/g
:%s/\\<p\\>/params/g
:%s/let params =/let params: RequestModuleParameters =/g
:%s/let params: RequestModuleParameters = null/let params: RequestModuleParameters = {}/g
:%s/new Object()/{}/g
:g/function/s/\\<e\\>/_event?: Event | null/
:%s/\\<e\\>/event/g
:g/function/s/\\<r\\>/response: YahooResponse/
:%s/\\<r\\>/response/g
:g/function/s/) {/): void {/
:%s/setTimeout(\\(['\"]\\)\\(.*\\)\1/setTimeout(() => \2/
" ${1%.js}.ts
# Exit editor with error (:cq) to block deletion
rm $1
