<?php



/**
 * List resolver template service.
 *
 */
class ListResolver extends TemplateService{
	protected $storage = array();

	private function loadList($listName){
		$fileName = PathManager::listSpecFile($listName);
			$xml = simplexml_load_file($fileName);

			$optionList = $xml->option;
			$out = array();
			foreach($optionList as $option){
				$out["$option".''] = $option->text[0].'';
			}
			$this->storage["$listName"] = $out;
	}

	public function getValuesArray($listName){
		if(!isset($this->storage["$listName"])){
			$this->loadList($listName);
		}
		return $this->storage["$listName"];
	}

	public function resolveKey($listName, $keyName){
		if(!isset($this->storage["$listName"])){
			$this->loadList($listName);
		}
		return $this->storage["$listName"]["$keyName"];
	}

	public function test(){
		echo "ListResolver tested";
	}

}
