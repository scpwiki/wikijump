#!/bin/sh
#
# 	WARNING: experimental. Test with a normal build.sh, and use this after confident. 
#
# 	Run this just as you would build.sh, knowing we're stealing optimize and cssOptimize from you
# 	You can use this as a template for your own custom build script. You NEED to specify a profile
# 	and should pass a action and version value
#
#	eg:
#		./build_mini.sh action=release profile=standard version=1.1.0-mini
#
#	After the build completes, all non-vital files will be removed. This works _great_ with
# 	namespaced code in the vicinity of dojo.js (a sibling, or ../../namespace) .. Though in
# 	the latter you MUST customize your profile.
#
#   WARNING: There are a lot of 'rm -f' commands in here, mostly checked, but know:
#	Use at your own risk! 
#
#	NOTE: This is a fairly difinitive example of what is not needed (except in testing) ...

rm_dojo_files ()
{
	for d in "$@"
	do
		if [ -e "dojo/$d" ]; then
			rm -rf "dojo/$d"
		fi
	done
}

echo "release: mini started"

# FIXME: refs #6616 - could be able to set a global copyright file and null out build_release.txt
mv copyright.txt _copyright.txt
cp copyright_mini.txt copyright.txt
mv build_notice.txt _build_notice.txt
touch build_notice.txt

# run the build, passing all params (our assumed params first)
java -jar ../shrinksafe/custom_rhino.jar build.js optimize=shrinksafe cssImportIgnore=../dijit.css cssOptimize=comments.keepLines "$@"

# did the release work?
if [ -d ../../release ]; then

	cd ../../release/

	# remove dojox tests and demos - they all follow this convetion
	for i in dojo/dojox/* 
	do
	  if [ -d $i ]; then
	    rm -rf $i/tests/
	    rm -rf $i/demos/ 
	  fi
	done
	
	# FIXME: any shell masters out there? this is just a list ...

	# removed dijit tests
	rm_dojo_files "dijit/tests" "dijit/demos" "dijit/bench" "dojo/tests" "dojo/tests.js" "util"

	# noir isn't worth including yet
	if [ -d dojo/dijit/themes/noir ]; then
		rm -rf dojo/dijit/themes/noir/
		
		# so the themes are there, lets assume that, piggyback on noir: FIXME later
		find ./dojo/dijit/themes/ -name *.html -exec rm '{}' ';'
		rm -rf dojo/dijit/themes/themeTesterImages/	

	fi
	# TODO: merge down to a single theme.css for any theme?

	# remove uncompressed .js files
	find . -name *.uncompressed.js -exec rm '{}' ';'

	# WARNING: templates have been inlined into the .js -- if you are using dynamic templates,
	# or other build trickery, these lines might not work!
	rm_dojo_files "dijit/templates" "dijit/form/templates" "dijit/layout/templates"

	# NOTE: we're not doing this in DojoX because the resources/ folder (to me) is deemed
	# ambigious, and should be treated on a per-project basis

	# NOTE: if you aren't using anything in DojoX, uncomment this line:
	# rm -rf dojo/dojox/
	# OR get creative and only populate dojox/ folder with the projects you need, and leave alone.
	# .. assume you didn't, and clean up all the README's (leaving LICENSE, mind you)
	find ./dojo/dojox/ -name README -exec rm '{}' ';'
	
	# WARNING: if you care about _base existing (and not _always_ just dojo.js providing it) then comment this line:
	rm_dojo_files "dojo/_base" "dojo/_base.js"

	# NOTE: we're not doing the above to dijit/_base/ because I secretly use dijit/_base functions
	# when only using dojo.js (place.js and sniff.js in particular), and mini would break stuff ...

	# last but not least
	rm_dojo_files "dojo/build.txt"
	
	cd ../util/buildscripts/

fi

# cleanup from above, refs #6616
mv _copyright.txt copyright.txt
mv _build_notice.txt build_notice.txt

echo "release: mini done!"
