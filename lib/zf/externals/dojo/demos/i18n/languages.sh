# Script to generate langues.json, list of languages and countries that use each language
# Steps to use:
#   1. cd util/buildscripts/cldr/ldml
#   2. unzip main.zip (alternately download from http://www.unicode.org/cldr)
#   3. cd main
#   4. run this script from there
#   5. use normalizeJson.html to remove trailing commas so IE doesn't choke

echo "["
for lang in $(ls [a-z][a-z].xml |sed s/.xml//)
do
	echo '{ key: "language:'${lang}'", type: "language", iso: "'${lang}'",'

	# countries that use this language
	echo 'countries: ['
	ls ${lang}_[A-Z][A-Z].xml | sed -e 's/^.*_/{_reference: "/' -e 's/.xml/"},/'
	echo '],'

	# name of this language (in this language)
	grep '<language type="'${lang}'"[ >]' ${lang}.xml |head -1 |sed -e 's/.*<language type=".."[^>]*>/name: "/' -e 's/<\/language>/",/'

	# names of other languages (in this language)
	grep '<language type="..">' ${lang}.xml |sed -e 's/.*<language type=//' -e 's/<\/language>/",/' -e 's/>/: "/' -e 's/-->//'
	echo '},'
done
echo "]"
