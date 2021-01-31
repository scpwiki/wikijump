#!/bin/bash
# vim: set noexpandtab:
# Sets a groundwork for converting a module JS to TS
# TODO Delete after modules conversion

set -e

(
	printf "import Wikijump from \"@/javascript/Wikijump\";\nimport OZONE from \"@/javascript/OZONE\";\nimport { RequestModuleParameters } from \"@/javascript/OZONE/ajax\";\nimport { Wikirequest } from \"wikirequest\";\ndeclare const WIKIREQUEST: Wikirequest;\ndeclare const YAHOO: any;\ndeclare type YahooResponse = any;\ndeclare const fx: any;\n";
	cat "$1"
) > "${1%.js}.ts"

standardx --fix "${1%.js}.ts" || true

$EDITOR -s /dev/stdin <<<-"
	:\" Expect an error until syntax issues are fixed
	:/Wikijump\\.modules\\./s!!//@ts-expect-error\rexport const !

	:\" Fix ManageSite modules naming
	:%s/ManagerSite/ManageSite/g

	:\" No need to absolute-reference the current module
	:%s/Wikijump\\.modules\\.${1%.js}/${1%.js}/g

	:\" Merge individual module sections into a single object definition
	:%s/\\_s*};\\_s*${1%.js}\\.\\(\\S\\+\\) =/\r},\r  \1:/
	:\" Remove an extra closing brace at the start of the module (rebalance later)
	:/export const/s/{\n},/{/

	:\" Replace the jQuery ID wrapper with a builtin
	:%s/\$(\\([^)]\\+\\))/document.getElementById(\1)!/g

	:\" 'p' by itself is shorthand for module request params
	:%s/\\<p\\>/params/g
	:%s/\\<parms\\>/params/g
	:%s/new Object()/{}/g
	:%s/\(var\\|let\\|const\\) params = \\[\\]/\\1 params = {}/
	:%s/\(var\\|let\\|const\\) params =/\\1 params: RequestModuleParameters =/
	:%s/\(var\\|let\\|const\\) params: RequestModuleParameters = null/\\1 params: RequestModuleParameters = {}/

	:\" Recursively move individual parameters into a single object definition
	:%s/^\\(\\s*\\)params\\.\\(\\S\\+\\) = \\(.*\\);/\\1  \\2: \\3,/
	qqqqq:%s/^\\(\\s*\\)\\(.*\\)};\\n\\1\\(  \\S*: .*,\\)/\\1\\2\\r\\1\\3\\r\\1};/@qq@q
	:%s/,\\n\\s*\\n/,\\r/

	:\" 'e' by itself is short for event, but it's usually unused
	:g/function/s/\\<e\\>,/_event: Event | null,/
	:g/function/s/\\<e\\>/_event?: Event | null/
	:%s/\\<e\\>/event/g

	:\" 'r' by itself is short for response
	:g/function/s/\\<r\\>/response: YahooResponse/
	:%s/\\<r\\>/response/g

	:\" The vast majority of functions return void, others can be manually set
	:g/function/s/) {/): void {/

	:\" Wikidot loves passing eval strings to setTimeout
	:%s/setTimeout(\\(['\"]\\)\\(.*\\)\\1/setTimeout(() => \\2/

	:\" I changed requestModule to not accept null but an empty object
	:%s/OZONE\\.ajax\\.requestModule([^,]*, \\zsnull/{}/

	:\" Rebalance the brackets by adding one to the end
	gg?};
	:s/};/}\\r};/

	:\" Remove trailing commas
	:%s/,\\(\\_s*}\\)/\1/

	:\" Autoindent
	G=gg

	:\" Update var to let & const
	:%s/\\<var\\>/let/g
	:\" Queue up (and immediately cancel) a manual let->const replace
	:%s/\\<let\\>/const/c
	" "${1%.js}.ts"
# Exit editor with error (:cq) to block deletion
rm "$1"
