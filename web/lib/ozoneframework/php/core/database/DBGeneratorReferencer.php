<?php



class DBGeneratorReferencer {
	private $references = array();

	public function addReference($primaryTableName, $primaryKeyName, $referencingTableName, $referencingKeyName, $customFunction=null){
		if($primaryTableName != $referencingTableName){
			$entry = array();
			$entry['primary_table'] = $primaryTableName;
			$entry['primary_key'] = $primaryKeyName;
			$entry['referencing_table'] = $referencingTableName;
			$entry['referencing_key'] = $referencingKeyName;
			$entry['custom_function'] = $customFunction;
			$this->references[] = $entry;
		}
	}

	public function getReferences(){
		return $this->references;
	}

	public function processXMLTable($xmlTable){
		$freferences = $xmlTable->foreignReference;
		foreach ($freferences as $fr){
			$this->addReference($fr['foreignTable'], $fr['foreignKey'], $xmlTable['name'], $fr['localKey'], $fr['customFunction'] );
			OzoneLogger::instance()->debug("found reference: M: ".$fr['foreignTable'].".". $fr['foreignKey'].", S: ". $xmlTable['name'].".".$fr['localKey']. " ". $fr['customFunction']);
		}
	}

}
