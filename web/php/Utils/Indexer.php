<?php

namespace Wikidot\Utils;

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

    public function deindexPage($page)
    {
    }

    public function indexThread($thread)
    {
    }

    public function deindexThread($thread)
    {
    }
}
