<?php
	// ensure that we don't try to send "html" down to the client
	header("Content-Type: text/plain");

	if (!$_REQUEST["message"]){
		print "ERROR: message property not found";
	}else{
		print $_REQUEST["message"];
	}
?>
