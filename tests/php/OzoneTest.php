<?php


namespace Tests;


class OzoneTest
{
    public function addPage ( $ncat , $unixName , $source , $title ) {

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

    /** Not ready for runtime yet, need to use database transactions. */
    public function new_pages_can_be_created()
    {
        Ozone::init () ;

        $db = Database::connection () ;
        $db->begin () ;

        $od = new Outdater ( ) ;
        $od->recompileWholeSite ( DB_SitePeer::instance ()->selectByPrimaryKey ( 1 ) ) ;

//        $db->commit () ;
//        $db->begin () ;

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

        addPage ( $ncat, "test", "Test page.", "Test page" ) ;

        $od->recompileWholeSite ( DB_SitePeer::instance ()->selectByPrimaryKey ( 1 ) ) ;

//        $db->commit () ;
        $db->rollback();
    }
}