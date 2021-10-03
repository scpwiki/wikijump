<?php

namespace Wikidot\Utils;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Wikidot\DB\FtsEntryPeer;
use Wikidot\DB\FtsEntry;
use Wikidot\DB\PageTagPeer;
use Wikidot\DB\PagePeer;
use Wikidot\DB\ForumPostPeer;

/**
 * Full text search handler Class.
 */
class Indexer
{

    private static $instance;

    public static function instance()
    {
        if (self::$instance == null) {
            self::$instance = new Indexer();
        }
        return  self::$instance;
    }

    public function indexPage($page)
    {
        // look for an existing fts_entry
        $ie = FtsEntryPeer::instance()->selectByPageId($page->getPageId());
        if (!$ie) {
            $ie = new FtsEntry();
            $ie->setPageId($page->getPageId());
            $ie->setSiteId($page->getSiteId());
        }
        // set properties (fields)
        $ie->setTitle(htmlspecialchars($page->getTitleOrUnixName()));
        $ie->setUnixName($page->getUnixName());

        $text = $page->getCompiled()->getText();
        $text = strip_tags($text);

        // kill modules
        $d = utf8_encode("\xFE");
        $text = preg_replace("/
            ${d}module\s            # Declare module
            \"([a-zA-Z0-9\/_]+?)\"  # Module definition, wrapped in quotes
            ([^$d]+?)?$d            # Anything else until end of module
            /x", "\n", $text);
        $ie->setText($text);
        $title = db_escape_string(htmlspecialchars($page->getTitleOrUnixName()));
        $unixName =  db_escape_string(htmlspecialchars($page->getUnixName()));

        //get tags
        $tags = PagePeer::getTags($page->getPageId());
        $tags = implode(' ', $tags);

        $db = Database::connection();
        $v = pg_version($db->getLink());
//      if(!preg_match(';^8\.3;', $v['server'])){
//          $db->query("SELECT set_curcfg('default')");
//      }
        $ie->setVector("(setweight( to_tsvector('$title'), 'A') || to_tsvector('".db_escape_string($text)."') || setweight( to_tsvector('$tags'), 'B'))", true);
        $ie->save();
    }

    public function deindexPage($page)
    {
        $ie = FtsEntryPeer::instance()->selectByPageId($page->getPageId());
        FtsEntryPeer::instance()->deleteByPrimaryKey($ie->getFtsId());
    }

    public function indexThread($thread)
    {
        // look for an existing fts_entry
        $ie = FtsEntryPeer::instance()->selectByThreadId($thread->getThreadId());
        if (!$ie) {
            $ie = new FtsEntry();
            $ie->setThreadId($thread->getThreadId());
            $ie->setSiteId($thread->getSiteId());
        }
        $ie->setTitle(htmlspecialchars($thread->getTitle()));
        $ie->setUnixName($thread->getUnixifiedTitle());
        // to create thread text select all posts and extract body

        $c = new Criteria();
        $c->add("thread_id", $thread->getThreadId());
        $c->addOrderAscending("post_id");
        $posts = ForumPostPeer::instance()->select($c);

        $text = '';
        foreach ($posts as $post) {
            $text .= $post->getTitle()."\n";
            $text .= strip_tags($post->getText())."\n\n";
        }
        $ie->setText(htmlspecialchars($thread->getDescription())."\n\n".$text);
        $title = db_escape_string(htmlspecialchars($thread->getTitle()));
        $description = db_escape_string(htmlspecialchars($thread->getDescription()));

        $db = Database::connection();
        $v = pg_version($db->getLink());
        if (!preg_match('/^8\.3/', $v['server'])) {
            // $db->query("SELECT set_curcfg('default')");  # This is related to tsearch2 which is no longer available.
        }

        $ie->setVector("setweight( to_tsvector('$title'), 'C') || setweight( to_tsvector('$description'), 'C') || to_tsvector('".db_escape_string($text)."')", true);

        $ie->save();
    }

    public function deindexThread($thread)
    {
        $ie = FtsEntryPeer::instance()->selectByThreadId($thread->getThreadId());
        FtsEntryPeer::instance()->deleteByPrimaryKey($ie->getFtsId());
    }
}
