#!/bin/bash

# Wikidot (Community Edition) - free wiki collaboration software
# 
# 							http://www.wikidot.org
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as
# published by the Free Software Foundation, either version 3 of the
# License, or (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.

# generates the .po file for the wikidot project

# prepare from tpl
./lib/ozone/bin/tsmarty2c.php `find . -name '*.tpl'` > tmp/tmp_locale.c

# generate pot
xgettext -p locale/ --from-code=UTF-8 -n  `find . -name '*.php'` tmp/tmp_locale.c
mv locale/messages.po locale/messages.pot

# merge with existing translations
# if [ -f $file.old ]
# then
#   msgmerge $file.old $file --output-file=$file
#  rm $file.old 
# fi

#rm tmp/tmp_locale.c