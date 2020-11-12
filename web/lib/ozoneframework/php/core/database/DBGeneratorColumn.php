<?php





/**
 * Database column generator.
 *
 */
class DBGeneratorColumn {

	private $name;
	private $type;
	private $defaultValue = null;
	private $canNull = true;
	private $primaryKey = false;
	private $unique = false;

	public function __construct($column_xml){
		$this->name = $column_xml['name'];
		$this->type = $column_xml['type'];
		if($column_xml['null'] == 'no' OR $column_xml['null'] == 'false'){
			$this->canNull = false;
		}
		if($column_xml['primaryKey'] == 'yes' || $column_xml['primaryKey'] == 'true'){
			$this->primaryKey = true;
		}
		if($column_xml['unique'] == 'yes' || $column_xml['unique'] == 'true'){
			$this->unique = true;
		}

		if(isset($column_xml['default'])){
			$this->defaultValue = 	$column_xml['default'];
		}

	}

	public function generateSQLPropertyString(){
		$out = $this->name." ".$this->type." ";
		if($this->canNull === false){
			$out .= " NOT NULL ";
		}
		if($this->defaultValue !== null ){
			$out .= "DEFAULT '" . $this->defaultValue ."' ";
		}
		if($this->primaryKey == true){
			$out .= "PRIMARY KEY";
		}
		if($this->unique == true){
			$out .= " UNIQUE ";
		}
		return $out;
	}

	public function getName(){
		return $this->name;
	}

	public function getType(){
		return $this->type;
	}

	public function getPropertyName(){
		return underscoreToLowerCase($this->name);
	}

	public function getPropertyNameFirstCapitalized(){
		return capitalizeFirstLetter(underscoreToLowerCase($this->name));
	}

	public function isPrimaryKey(){
		return $this->primaryKey;
	}
	public function setPrimaryKey($val){
		$this->primaryKey = $val;
	}
	public function isUnique(){
		return $this->unique;
	}
	public function setUnique(){
		return $this->unique;
	}

	public function getDefaultValue(){
		return $this->defaultValue;
	}

	public function isIntLike(){
		$pos1 = strpos($this->type, 'int');
		$pos2 = strpos($this->type, 'INT');
		if($pos1 !== false || $pos2 !== false  ){
			return true;
		} else {
			return false;
		}
	}

}
