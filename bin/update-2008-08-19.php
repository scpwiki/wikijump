<?php

chdir(dirname(__FILE__));
require_once ("../php/setup.php") ;

function addAPage ( $ncat , $unixName , $source , $title ) {
	
	$now = new ODate ( ) ;
	
	$nsource = new DB_PageSource ( ) ;
	$nsource->setText ( $source ) ;
	$nsource->save () ;
	
	$nmeta = new DB_PageMetadata ( ) ;
	$nmeta->setTitle ( $title ) ;
	$nmeta->setUnixName ( $unixName ) ;
	
	$nmeta->setOwnerUserId ( 1 ) ;
	$nmeta->save () ;
	
	$nrev = new DB_PageRevision ( ) ;
	$nrev->setSiteId ( 1 ) ;
	$nrev->setSourceId ( $nsource->getSourceId () ) ;
	$nrev->setMetadataId ( $nmeta->getMetadataId () ) ;
	$nrev->setFlagNew ( true ) ;
	$nrev->setDateLastEdited ( $now ) ;
	$nrev->setUserId ( 1 ) ;
	$nrev->obtainPK () ;
	
	$npage = new DB_Page ( ) ;
	$npage->setSiteId ( 1 ) ;
	$npage->setCategoryId ( $ncat->getCategoryId () ) ;
	$npage->setRevisionId ( $nrev->getRevisionId () ) ;
	$npage->setSourceId ( $nsource->getSourceId () ) ;
	$npage->setMetadataId ( $nmeta->getMetadataId () ) ;
	$npage->setTitle ( $title ) ;
	$npage->setUnixName ( $unixName ) ;
	$npage->setDateLastEdited ( $now ) ;
	$npage->setDateCreated ( $now ) ;
	$npage->setLastEditUserId ( 1 ) ;
	$npage->setOwnerUserId ( 1 ) ;
	
	$npage->save () ;
	$nrev->setPageId ( $npage->getPageId () ) ;
	$nrev->save () ;
	
	$ncomp = new DB_PageCompiled ( ) ;
	$ncomp->setPageId ( $npage->getPageId () ) ;
	$ncomp->setDateCompiled ( $now ) ;
	$ncomp->save () ;

}

Ozone::init () ;

$db = Database::connection () ;
$db->begin () ;

$od = new Outdater ( ) ;
$od->recompileWholeSite ( DB_SitePeer::instance ()->selectByPrimaryKey ( 1 ) ) ;

$db->commit () ;
$db->begin () ;

$c = new Criteria();
$c->add("name", "auth");
$c->add("site_id", 1);

if (DB_CategoryPeer::instance()->selectOne($c)) {
	die("The auth category already exists!\n\n");
}

$ncat = DB_CategoryPeer::instance ()->selectByPrimaryKey ( 1 ) ;
$ncat->setNew ( true ) ;
$ncat->setCategoryId ( null ) ;
$ncat->setName ( "auth" ) ;
$ncat->save () ;

addAPage ( $ncat, "auth:login", "[[module LoginModule]]", "Log in" ) ;
addAPage ( $ncat, "auth:newaccount", "[[module CreateAccount]]", "Create account - step 1" ) ;
addAPage ( $ncat, "auth:newaccount2", "[[module CreateAccount2]]", "Create account - step 2" ) ;
addAPage ( $ncat, "auth:newaccount3", "[[module CreateAccount3]]", "Create account - step 3" ) ;

$od->recompileWholeSite ( DB_SitePeer::instance ()->selectByPrimaryKey ( 1 ) ) ;

$db->commit () ;

echo "Successfully added the new auth pages!\n\n(Don't worry if Segmentation fault occurs BELOW)\n\n";
