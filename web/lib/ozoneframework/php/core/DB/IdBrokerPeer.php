<?php

namespace Ozone\Framework\DB;



use DB\IdBrokerPeerBase;
use Ozone\Framework\Database\Database;





/**
 * Id broker peer Class.
 *
 */
class IdBrokerPeer extends IdBrokerPeerBase {

	/**
	 * Updates internal data regarding indexes for primary keys.
	 */
	public function updateIndexes(){
		$ents = $this->select();
		foreach ($ents as $ent){
			// get max value of index in the database
			$query = "SELECT MAX(".$ent->getColumnName().") AS m FROM ".$ent->getTableName();
			$db = Database::connection();
			$result = $db->query($query);
			$row = $result->nextRow();
			$maxIdx = $row['m'];
			if($maxIdx == null){
				$ent->setNextFreeIndex(0);
			} else {
				$ent->setNextFreeIndex($maxIdx + 1);
			}

			$ent->save();
		}
	}

}
