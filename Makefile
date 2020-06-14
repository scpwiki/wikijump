
all: db keys config finish

prepare_db:
	bin/prepare_db.php | psql

db:
	bin/bootstrap_db.php files/dump/db/*.sql
	bin/generate_om.php
	bin/bootstrap_pages.php files/dump/sites/*

keys:
	bin/generate_keys.sh

config:
	bin/configure.php

finish:
	@echo
	@echo ============================================
	@echo make complete
	@echo ============================================
	@echo
	@echo Run Wikidot server with ./wikidotctl start
	@echo and navigate to the following URL to finish:
	@echo
	@bin/finish_url.php
	@echo

debs:
	rm -rf build .build
	mkdir -p .build/wikidot-`cat VERSION`
	cp -r * .build/wikidot-`cat VERSION`
	mv .build build
	find build/wikidot-`cat VERSION` -type d -name '.git' -print0 | xargs -0 rm -r
	cd build/wikidot-`cat VERSION`; rm -r lib/zf/bin lib/zf/demos lib/zf/tests lib/zf/documentation
	cd build/wikidot-`cat VERSION`; dpkg-buildpackage -rfakeroot

