<?
class PageLookupQModule extends QuickModule {
	
	public function process($data){
		// does not use data
		
		$search = $_GET['q'];
		$siteId = $_GET['s'];
		if(!is_numeric($siteId)) return;
		if($search == null || strlen($search) ==0) return;
		$search2 = pg_escape_string(preg_quote(str_replace(' ','-',$search)));
		$search7 = pg_escape_string($search);
		$search = pg_escape_string(preg_quote($search));
		
		$siteId = pg_escape_string($siteId);
		Database::init();
		$q1 = "SELECT unix_name, COALESCE(title,unix_name) AS title FROM page WHERE " .
				"site_id ='$siteId' AND ".
				"("; 
		if($_GET['title'] == 'yes')	{$q1 .= "title ~* '^$search' OR ";}
		$q1 .= " unix_name ~* '^$search2') ";
		//$q1 .= "AND 
		
			$q1 .= "ORDER BY unix_name";
		$db = Database::connection();
		
		$result1 = $db->query($q1);
		
		$result1 = $result1->fetchAll();
		
		return array('pages' => $result1);
	}

}
