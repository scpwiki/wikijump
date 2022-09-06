<?php

namespace Ozone\Framework\Template\Services\OnDemand;



use Ozone\Framework\Database\Criteria;
use Ozone\Framework\MessageResolver;
use Ozone\Framework\PathManager;
use Ozone\Framework\TemplateService;

/**
 * List resolver service.
 *
 */
class ListResolver extends TemplateService{
	protected $storage = array();
	protected $pleaseSelectValues = array();

	public function __construct($runData = null){}

	public function loadList($listName){
		$fileName = PathManager::listSpecFile($listName);
			$xml = simplexml_load_file($fileName);
			$optionList = $xml->option;
			$out = array();
			foreach($optionList as $option){
				$optionKey=$option['key'];
				$out["$optionKey"] = $option->text[0].'';
			}
			$this->storage["$listName"] = $out;
			$this->pleaseSelectValues["$listName"] = $xml->pleaseselect[0]->text[0].'';
	}

	public function loadListFromTable($listName){
		$tableName = $listName;
		// table must have columns: key, text and sort_index
		// the row with option_id = null indicates the "please select" value

		$peerName = $peerName = "Wikidot_DB_". capitalizeFirstLetter(underscoreToLowerCase($tableName))."Peer";
		$peer = new $peerName();
		$c = new Criteria();
		$c->add("key", null);

		$pleaseSelectOption = $peer->selectOne($c);

		$c = new Criteria();
		$c->add("key", null, "!=");
		$c->addOrderAscending("sort_index");
		$c->addOrderAscending("text");

		$options = $peer->select($c);

		if($pleaseSelectOption != null){
			$this->pleaseSelectValues["$listName"] = $pleaseSelectOption->getText();
		} else {
			// try SELECT_PLEASE_SELECT from the messages...
			$text = MessageResolver::instance()->message("SELECT_PLEASE_SELECT");
			if($text != null){
				$this->pleaseSelectValues["$listName"] = $text;
			}
		}

		$out = array();
		foreach($options as $option){
			$optionKey=$option->getKey();
			$out["$optionKey"] = $option->getText();
		}
		$this->storage["$listName"] = $out;
	}

	public function getValuesArray($listName){
		if(!isset($this->storage["$listName"])){
			$this->loadList($listName);
		}
		return $this->storage["$listName"];
	}

	public function getValuesArrayFromTable($listName){
		if(!isset($this->storage["$listName"])){
			$this->loadListFromTable($listName);
		}
		return $this->storage["$listName"];
	}

	public function getPleaseSelectValue($listName){
		if(!isset($this->pleaseSelectValues["$listName"])){
			$this->loadList($listName);
		}
		return $this->pleaseSelectValues["$listName"];
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
