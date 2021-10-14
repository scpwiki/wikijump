<?php

namespace Tests;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\Ozone;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\Page;
use Wikidot\DB\PageCompiled;
use Wikidot\DB\PageMetadata;
use Wikidot\DB\PageRevision;
use Wikidot\DB\PageSource;
use Wikidot\DB\SitePeer;
use Wikidot\Utils\Outdater;

class OzoneTest
{
    public function addPage($ncat, $unixName, $source, $title)
    {
        $now = new ODate();

        $nsource = new PageSource();
        $nsource->setText($source);
        $nsource->save();

        $nmeta = new PageMetadata();
        $nmeta->setTitle($title);
        $nmeta->setUnixName($unixName);

        $nmeta->setOwnerUserId(1);
        $nmeta->save();

        $nrev = new PageRevision();
        $nrev->setSiteId(1);
        $nrev->setSourceId($nsource->getSourceId());
        $nrev->setMetadataId($nmeta->getMetadataId());
        $nrev->setFlagNew(true);
        $nrev->setDateLastEdited($now);
        $nrev->setUserId(1);
        $nrev->obtainPK();

        $npage = new Page();
        $npage->setSiteId(1);
        $npage->setCategoryId($ncat->getCategoryId());
        $npage->setRevisionId($nrev->getRevisionId());
        $npage->setSourceId($nsource->getSourceId());
        $npage->setMetadataId($nmeta->getMetadataId());
        $npage->setTitle($title);
        $npage->setUnixName($unixName);
        $npage->setDateLastEdited($now);
        $npage->setDateCreated($now);
        $npage->setLastEditUserId(1);
        $npage->setOwnerUserId(1);

        $npage->save();
        $nrev->setPageId($npage->getPageId());
        $nrev->save();

        $ncomp = new PageCompiled();
        $ncomp->setPageId($npage->getPageId());
        $ncomp->setDateCompiled($now);
        $ncomp->save();
    }

    /** Not ready for runtime yet, need to use database transactions. */
    public function new_pages_can_be_created()
    {
        Ozone::init();

        $db = Database::connection();
        $db->begin();

        $od = new Outdater();
        $od->recompileWholeSite(SitePeer::instance()->selectByPrimaryKey(1));

        //        $db->commit () ;
        //        $db->begin () ;

        $c = new Criteria();
        $c->add('name', 'auth');
        $c->add('site_id', 1);

        if (CategoryPeer::instance()->selectOne($c)) {
            die("The auth category already exists!\n\n");
        }

        $ncat = CategoryPeer::instance()->selectByPrimaryKey(1);
        $ncat->setNew(true);
        $ncat->setCategoryId(null);
        $ncat->setName('auth');
        $ncat->save();

        addPage($ncat, 'test', 'Test page.', 'Test page');

        $od->recompileWholeSite(SitePeer::instance()->selectByPrimaryKey(1));

        //        $db->commit () ;
        $db->rollback();
    }
}
