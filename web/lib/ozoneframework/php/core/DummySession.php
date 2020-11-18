<?php




use DB\OzoneSessionBase;

/**
 * The Dummy Session Object
 */
class DummySession extends OzoneSessionBase {

	public function __call($m, $a){
		return null;
	}
}
