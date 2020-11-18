<?php



/**
 * File upload utility.
 *
 */
class FileUpload {

	private $multiple = false;

	public function __construct(){

	}

	public function processUpload(){
		// check if proper upload

		//check if multiple
		//is_array($_FILES[])
		// get all uploaded files

	}

	public function getFileItem($fieldKey){
		/* a nasty hack follows... because of a strange behaviour of the $_FILES array */
		$ar = array();
		$ar['name'] = $_FILES[$fieldKey]['name'];
		$ar['tmp_name'] = $_FILES["$fieldKey"]['tmp_name'];
		$ar['type'] = $_FILES["$fieldKey"]['type'];
		$ar['size'] = $_FILES["$fieldKey"]['size'];
		$ar['error'] = $_FILES["$fieldKey"]['error'];
		return new FileItem($ar);

	}

}
