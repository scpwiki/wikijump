<?php

namespace Ozone\Framework;





/**
 * Dummy memcache resource.
 *
 */
class DummyMemcache {

	public function get($key){
		return false;
	}

	public function set($parm1, $parm2, $parm3=null, $parm4=null){
		return null;
	}

	public function delete($key){}
}
