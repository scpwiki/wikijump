<?
class UserLookupQModule extends QuickModule {
	
	public function process($data){
		// does not use data
		
		$search = $_GET['q'];
		if($search == null || strlen($search) ==0) return;
		
		$search1 = pg_escape_string(preg_quote($search));
		$search2 = pg_escape_string($search);
		
		Database::init();
		$q1 = "SELECT nick_name AS name, user_id FROM ozone_user WHERE " .
				"nick_name ~* '^$search1' AND nick_name != '$search2'";
		$q1 .= "ORDER BY nick_name LIMIT 20";
		$q2 = "SELECT nick_name AS name, user_id FROM ozone_user WHERE " .
				"nick_name = '$search2' ";
		$db = Database::connection();
		
		$result1 = $db->query($q1);
		$result1 = $result1->fetchAll();
		$result2 = $db->query($q2);
		$result2 = $result2->fetchAll();
		
		if($result1 == null && $result2 != null) $result = $result2;
		if($result2 == null && $result1 != null) $result = $result1;
		if($result1 == null && $result2 == null) $result = false; // NOT null since it breakes autocomplete!!!
		if($result1 != null && $result2 != null){
			$result = array_merge($result2, $result1);
		}
		
		return array('users' => $result);
	}
	
}
