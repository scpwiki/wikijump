<?php

namespace Ozone\Framework;




use Wikidot\DB\OzoneSessionBase;

/**
 * The Dummy Session Object
 */
class DummySession extends OzoneSessionBase {

	public function __call($m, $a){
		return null;
	}
}
