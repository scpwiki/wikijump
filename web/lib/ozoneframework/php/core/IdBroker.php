<?php



use DB\IdBrokerPeer;

/**
 * Id broker utility.
 *
 */
class IdBroker{

	public function nextId($tableName){
		$db = Database::connection();
		$idbpeer = IdBrokerPeer::instance();
		$db->begin();
		$t = $idbpeer->selectOneByExplicitQuery("WHERE table_name = '$tableName' FOR UPDATE");
		$index = $t->getNextFreeIndex();
		$t->setNextFreeIndex($index+1);
		$t->save();
		$db->commit();
		return $index;
	}

}
