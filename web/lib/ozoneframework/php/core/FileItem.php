<?php





/**
 * Upload file item.
 *
 */
class FileItem {
	private $name;
	private $tmpName;
	private $size;
	private $type;
	private $error;

	public function __construct($ar){
		$this->name = $ar['name'];
		$this->type = $ar['type'];
		$this->tmpName = $ar['tmp_name'];
		$this->error = $ar['error'];
		$this->size = $ar['size'];
	}

	public function getName(){
		return $this->name;
	}

	public function getTmpName(){
		return $this->tmpName;
	}

	public function getSize(){
		return $this->size;
	}

	public function getType(){
		return $this->type;
	}

	/**
	 * Returns the error code.
	 */
	public function getError(){
		return $this->error;
	}
}
