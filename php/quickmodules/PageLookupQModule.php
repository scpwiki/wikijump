<?
class PageLookupQModule extends QuickModule {
	
	public function process($data){
		$search = $_GET['q'];
		$siteId = $_GET['s'];
        if (isset($_GET['parent'])) {
            $parent = WDStringUtils::toUnixName($_GET['parent']);
        } else {
            $parent = null;
        }
        $title = (isset($_GET['title']) && $_GET['title'] == 'yes');

		if (! is_numeric($siteId) || $search == null || strlen($search) == 0) {
            return;
        }

		$search = pg_escape_string(preg_quote(str_replace(' ','-',$search)));
		$siteId = pg_escape_string($siteId);
		
        $orTitle = ($title) ? "OR title ~* '^$search'" : "";

		$query = "SELECT unix_name, COALESCE(title,unix_name) AS title FROM page ";
        $query .= "WHERE site_id ='$siteId' AND (unix_name ~* '^$search' $orTitle)";

        if ($parent) {
            $parent = pg_escape_string($parent);
            $query .= " AND parent_page_id IN (SELECT page_id FROM page WHERE unix_name = '$parent') ";
        }
		
        $query .= "ORDER BY unix_name";

		Database::init();
		return array('pages' => Database::connection()->query($query)->fetchAll());
	}

}
