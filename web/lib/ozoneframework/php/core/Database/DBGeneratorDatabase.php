<?php

namespace Ozone\Framework\Database;



use Ozone\Framework\DB\IdBrokerPeer;
use Ozone\Framework\IdBroker;

/**
 * Database generator.
 *
 */
class DBGeneratorDatabase {

	private $tables = array ();
	private $views = array ();
	private $referencer;

	private $sql = array();

	private $executeSql = true;

	public function __construct($xml = null) {
		$this->referencer = new DBGeneratorReferencer();
	}

	public function addSchema($xml) {
		foreach ($xml->table as $table) {
			echo "table: ".$table['name']."\n";
			$tname = $table['name'];
			$this->tables["$tname"] = new DBGeneratorTable($table);
			$this->referencer->processXMLTable($table);
		}

    	foreach ($xml->view as $view) {
    		echo "view: ".$view['name']."\n";
    		$vname = $view['name'];
    		$this->views["$vname"] = new DBGeneratorView($view);
    	}
	}

	public function executeSQL() {
		$db = Database::connection();
		global $dropTables;
		foreach ($this->tables as $table) {
			unset($sql);
			if (!$db->tableExists($table->getName())) {
				$sql = $table->generateSQLCreateString();
			} else
				if ($dropTables || $table->getName() == 'ID_BROKER') {
					// note: ID_BROKER should always be dropped
					$sql = "DROP TABLE ".$table->getName() ." CASCADE";
				} else{
					$sql = $table->generateSQLAlterString();
				}
			if($sql){
				$this->sql = array_merge($this->sql, (array)$sql);
				if($this->executeSql){
					$db->query($sql);
				}
			}
		}

		foreach ($this->views as $view) {
			unset($sql);
			if (!$db->tableExists($view->getName())) {
				$sql = $view->generateSQLCreateString()."\n";
			} else	{
				$sql = "DROP VIEW ".$view->getName()	;
				$sql = $view->generateSQLCreateString()."\n";
			}
			if($sql){
				$this->sql = array_merge($this->sql, (array)$sql);
				if($this->executeSql){
					$db->query($sql);
				}
			}
		}
	}

	public function generateClasses(){
		foreach ($this->tables as $table) {
			$table->generateClass();
		}
		foreach ($this->views as $view) {
			$view->generateClass();
		}
	}

	public function setupIdBroker(){
		// for each table with a INT-LIKE primary key let the pk be
		// handled by the IdBroker.

		foreach($this->tables as $table){
			$pkColumn = $table->getPkColumn();

			if($pkColumn != null && ($pkColumn->isIntLike())){
				echo $pkColumn->getName();
				// check if not already there
				$c = new Criteria();
				$c->add('column_name', $pkColumn->getName());
				$c->add('table_name', $table->getName());
				$r = IdBrokerPeer::instance()->selectOne($c);
				if($r == null){
					$idbe = new IdBroker();
					$idbe->setTableName($table->getName());
					$idbe->setColumnName($pkColumn->getName());
					$idbe->save();
				}
			}

		}

		//in case of regeneration - update the indexes:
		$idbp = IdBrokerPeer::instance();
		$idbp->updateIndexes();

	}

	/**
	 * Updates references between tables. It is required before the SQL and
	 * Class genetation.
	 */
	public function updateReferences(){
//
//			// add referencing keys to the "primary table"
//
//
//

		foreach ($this->tables as $table){
			$table->updateReferences($this->referencer);
		}
	}

	public function setExecuteSql($val){
		$this->executeSql = $val;
	}

	public function getSql(){
		return $this->sql;
	}
}
